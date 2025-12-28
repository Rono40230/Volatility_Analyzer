// services/volatility/analyzer.rs - Analyseur de volatilité principal
// Conforme .clinerules : < 300L, structure claire, pas d'unwrap()

use super::best_quarter_finder::BestQuarterFinder;
use super::event_loader::EventLoader;
use super::hourly_stats::HourlyStatsCalculator;
use super::metrics::MetricsAggregator;
use super::quarterly_aggregator::QuarterlyAggregator;
use super::stats_15min::Stats15MinCalculator;
use super::volatility_heuristics::VolatilityHeuristics;
use crate::db::DbPool;
use crate::models::{
    AnalysisResult, AssetProperties, Candle, Result, RiskLevel, TradingRecommendation,
    VolatilityError,
};
use chrono::Datelike;
use tracing::info;

/// Analyseur de volatilité principal
pub struct VolatilityAnalyzer {
    candles: Vec<Candle>,
}

impl VolatilityAnalyzer {
    /// Crée un nouvel analyseur avec les bougies fournies
    pub fn new(candles: Vec<Candle>) -> Self {
        Self { candles }
    }

    /// Effectue l'analyse complète et retourne le résultat
    pub fn analyze(&self, symbol: &str, pool: Option<DbPool>) -> Result<AnalysisResult> {
        info!("Starting volatility analysis for {}", symbol);

        if self.candles.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "No candles provided for analysis".to_string(),
            ));
        }

        // Calculer période et timeframe avec format français
        let period_start = self
            .candles
            .first()
            .map(|c| {
                let day = c.datetime.day();
                let month = match c.datetime.month() {
                    1 => "janvier",
                    2 => "février",
                    3 => "mars",
                    4 => "avril",
                    5 => "mai",
                    6 => "juin",
                    7 => "juillet",
                    8 => "août",
                    9 => "septembre",
                    10 => "octobre",
                    11 => "novembre",
                    12 => "décembre",
                    _ => "?",
                };
                let year = c.datetime.year();
                format!("{} {} {}", day, month, year)
            })
            .unwrap_or_else(|| "N/A".to_string());

        let period_end = self
            .candles
            .last()
            .map(|c| {
                let day = c.datetime.day();
                let month = match c.datetime.month() {
                    1 => "janvier",
                    2 => "février",
                    3 => "mars",
                    4 => "avril",
                    5 => "mai",
                    6 => "juin",
                    7 => "juillet",
                    8 => "août",
                    9 => "septembre",
                    10 => "octobre",
                    11 => "novembre",
                    12 => "décembre",
                    _ => "?",
                };
                let year = c.datetime.year();
                format!("{} {} {}", day, month, year)
            })
            .unwrap_or_else(|| "N/A".to_string());

        // Déterminer le timeframe en calculant l'intervalle moyen entre bougies
        // APPROCHE STRICTE : valider l'interval et logger si inconnu (vs fallback silencieux)
        let timeframe = if self.candles.len() > 1 {
            let first_ts = self.candles[0].datetime.timestamp();
            let second_ts = self.candles[1].datetime.timestamp();
            let interval_seconds = (second_ts - first_ts).abs();

            let recognized_timeframe = match interval_seconds {
                60 => Some("M1"),
                300 => Some("M5"),
                900 => Some("M15"),
                1800 => Some("M30"),
                3600 => Some("H1"),
                14400 => Some("H4"),
                86400 => Some("D1"),
                _ => None,
            };

            match recognized_timeframe {
                Some(tf) => tf.to_string(),
                None => {
                    // Log warning pour interval inconnu mais continue (soft fail)
                    tracing::warn!(
                        "Unexpected candle interval: {} seconds for symbol {}. Using M1 as fallback.",
                        interval_seconds,
                        symbol
                    );
                    format!("M1_INFERRED({}s)", interval_seconds)
                }
            }
        } else {
            // Pas assez de bougies pour calculer l'interval
            tracing::warn!(
                "Only {} candles loaded for symbol {} - cannot determine timeframe",
                self.candles.len(),
                symbol
            );
            "M1_DEFAULT".to_string()
        };

        // 1. Calcule les statistiques par heure
        let calculator = HourlyStatsCalculator::new(&self.candles, symbol.to_string());
        let mut hourly_stats = calculator.calculer()?;

        // 1.5 Calcule les statistiques par tranche de 15 minutes (pour scalping)
        let calculator_15min = Stats15MinCalculator::new(&self.candles);
        let mut stats_15min = calculator_15min.calculer()?;

        // Détection des propriétés de l'actif (Unités, Pips)
        let asset_props = AssetProperties::from_symbol(symbol);
        let point_value = asset_props.pip_value;
        let unit = asset_props.unit;

        // 1.6 Agrège les stats par quarter pour obtenir les moyennes historiques
        stats_15min = QuarterlyAggregator::aggregate(&stats_15min, point_value, symbol);

        // 1b. Charge les événements économiques et les associe aux heures
        if let Err(e) = EventLoader::load_and_associate_events(
            &self.candles,
            symbol,
            &mut hourly_stats,
            pool.clone(),
        ) {
            tracing::warn!("Failed to load events for {}: {}", symbol, e);
            // Continue quand même, les événements ne sont pas critiques
        }

        // 1c. Charge les événements pour les tranches de 15 minutes également
        if let Err(e) = EventLoader::load_and_associate_events_15min(
            &self.candles,
            symbol,
            &mut stats_15min,
            pool.clone(),
        ) {
            tracing::warn!("Failed to load 15min events for {}: {}", symbol, e);
            // Continue quand même
        }

        // Log pour vérifier les événements des quarters
        let total_15min_events: usize = stats_15min.iter().map(|s| s.events.len()).sum();
        tracing::info!(
            "📋 Stats15Min: {} slices, {} total events across all quarters",
            stats_15min.len(),
            total_15min_events
        );

        for (_idx, slice) in stats_15min.iter().enumerate().take(10) {
            if !slice.events.is_empty() {
                tracing::info!(
                    "   Slice {}:{}  has {} events: {:?}",
                    slice.hour,
                    slice.quarter,
                    slice.events.len(),
                    slice
                        .events
                        .iter()
                        .map(|e| &e.event_name)
                        .collect::<Vec<_>>()
                );
            }
        }

        // 2. Trouve le meilleur quarter
        let best_quarter = BestQuarterFinder::find_best_quarter(&stats_15min).unwrap_or((0, 0));

        // 2. Calcule les métriques globales
        let global_metrics =
            MetricsAggregator::calculer_metriques_globales(&hourly_stats, self.candles.len());

        // 4. Calcule le score de confiance
        let confidence_score = MetricsAggregator::calculer_score_confiance(&global_metrics);

        // 5. Génère la recommandation
        let recommendation = TradingRecommendation::from_confidence(confidence_score);

        // 6. Détermine le niveau de risque
        let risk_level = RiskLevel::from_volatility(global_metrics.mean_volatility);

        // 6b. VALIDE LA COHÉRENCE RECOMMENDATION × RISK
        // Si incohérent, ajuste la recommandation pour matcher le risque
        let mut recommendation = recommendation.validate_with_risk(&risk_level);

        // LOGIC-01: Détection Whipsaw (Doji Géant) sur le meilleur quarter
        // Si détecté, on force la recommandation à RISKY pour protéger le capital
        // Cette vérification doit se faire APRÈS la validation de risque pour avoir le dernier mot
        if let Some(best_stats) = stats_15min
            .iter()
            .find(|s| s.hour == best_quarter.0 && s.quarter == best_quarter.1)
        {
            if VolatilityHeuristics::is_giant_doji(best_stats) {
                tracing::warn!("⚠️ Whipsaw detected on best quarter! Downgrading recommendation to RISKY.");
                recommendation = TradingRecommendation::StraddleRisky;
            }
        }

        // NOTE: Corrélation événements économiques gérée séparément via EventCorrelationService
        // (voir EventCorrelationView.vue pour affichage)

        info!(
            "Analysis complete: confidence={:.1}, recommendation={:?}, risk={:?}",
            confidence_score, recommendation, risk_level
        );

        Ok(AnalysisResult {
            symbol: symbol.to_string(),
            period_start,
            period_end,
            timeframe,
            hourly_stats,
            stats_15min,
            best_quarter,
            confidence_score,
            recommendation,
            risk_level,
            global_metrics,
            point_value,
            unit,
        })
    }
}
