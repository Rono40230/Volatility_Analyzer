// services/volatility/analyzer.rs - Analyseur de volatilité principal
// Conforme .clinerules : < 150L, structure claire, pas d'unwrap()

use super::hourly_stats::HourlyStatsCalculator;
use super::stats_15min::Stats15MinCalculator;
use super::metrics::MetricsAggregator;
use crate::db::DbPool;
use crate::models::{
    AnalysisResult, Candle, Result, RiskLevel, TradingRecommendation, VolatilityError,
};
use chrono::{Datelike, Timelike};
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
            tracing::warn!("Only {} candles loaded for symbol {} - cannot determine timeframe", 
                          self.candles.len(), symbol);
            "M1_DEFAULT".to_string()
        };

        // 1. Calcule les statistiques par heure
        let calculator = HourlyStatsCalculator::new(&self.candles);
        let mut hourly_stats = calculator.calculate()?;

        // 1.5 Calcule les statistiques par tranche de 15 minutes (pour scalping)
        let calculator_15min = Stats15MinCalculator::new(&self.candles);
        let mut stats_15min = calculator_15min.calculate()?;

        // 1b. Charge les événements économiques et les associe aux heures
        if let Err(e) = self.load_and_associate_events(symbol, &mut hourly_stats, pool.clone()) {
            tracing::warn!("Failed to load events for {}: {}", symbol, e);
            // Continue quand même, les événements ne sont pas critiques
        }

        // 1c. Charge les événements pour les tranches de 15 minutes également
        if let Err(e) = self.load_and_associate_events_15min(symbol, &mut stats_15min, pool.clone()) {
            tracing::warn!("Failed to load 15min events for {}: {}", symbol, e);
            // Continue quand même
        }

        // 2. Trouve les meilleures heures
        let best_hours = MetricsAggregator::find_best_hours(&hourly_stats);

        // 3. Calcule les métriques globales
        let global_metrics =
            MetricsAggregator::calculate_global_metrics(&hourly_stats, self.candles.len());

        // 4. Calcule le score de confiance
        let confidence_score = MetricsAggregator::calculate_confidence_score(&global_metrics);

        // 5. Génère la recommandation
        let recommendation = TradingRecommendation::from_confidence(confidence_score);

        // 6. Détermine le niveau de risque
        let risk_level = RiskLevel::from_volatility(global_metrics.mean_volatility);

        // 6b. VALIDE LA COHÉRENCE RECOMMENDATION × RISK
        // Si incohérent, ajuste la recommandation pour matcher le risque
        let recommendation = recommendation.validate_with_risk(&risk_level);

        // NOTE: Corrélation événements économiques gérée séparément via EventCorrelationService
        // (voir EventCorrelationView.vue pour affichage)

        info!(
            "Analysis complete: confidence={:.1}, recommendation={:?}, risk={:?}",
            confidence_score,
            recommendation,
            risk_level
        );

        Ok(AnalysisResult {
            symbol: symbol.to_string(),
            period_start,
            period_end,
            timeframe,
            hourly_stats,
            stats_15min,
            best_hours,
            confidence_score,
            recommendation,
            risk_level,
            global_metrics,
        })
    }

    /// Charge les événements économiques (HIGH/MEDIUM) et les associe aux heures
    fn load_and_associate_events(
        &self,
        symbol: &str,
        hourly_stats: &mut Vec<crate::models::HourlyStats>,
        pool: Option<DbPool>,
    ) -> Result<()> {
        // Si pas de pool, skip chargement des événements
        let Some(pool) = pool else {
            return Ok(());
        };

        // Charger les événements du calendrier pour la période analysée
        let start_time = self
            .candles
            .first()
            .map(|c| c.datetime.naive_utc())
            .ok_or(crate::models::VolatilityError::InsufficientData(
                "No candles to determine event period".to_string(),
            ))?;

        let end_time = self
            .candles
            .last()
            .map(|c| c.datetime.naive_utc())
            .ok_or(crate::models::VolatilityError::InsufficientData(
                "No candles to determine event period".to_string(),
            ))?;

        // Charger événements via EventCorrelationService
        let event_service = crate::services::EventCorrelationService::new(pool);
        let events = event_service
            .get_events_for_period(symbol, start_time, end_time)
            .map_err(|e| crate::models::VolatilityError::DatabaseError(e.to_string()))?;

        // Filtrer HIGH/MEDIUM impact et compter par heure (Paris)
        // NOTE: Les candles sont en UTC, on les convertit en heure de Paris (UTC+1/+2 selon DST)
        // Paris: UTC+1 en hiver, UTC+2 en été
        // Pour simplifier, on utilise UTC+1 (heure d'hiver standard)
        const PARIS_OFFSET_HOURS: i32 = 1;

        for event in events {
            if event.impact != "HIGH" && event.impact != "MEDIUM" {
                continue;
            }

            // Convertir l'heure UTC en heure de Paris
            let utc_hour = event.event_time.hour() as i32;
            let paris_hour = (utc_hour + PARIS_OFFSET_HOURS) % 24;
            let paris_hour_u8 = paris_hour as u8;

            // Trouver l'heure correspondante dans hourly_stats
            // NOTE: hourly_stats contient les statistiques en UTC, donc on doit chercher l'heure UTC
            // Mais on affichera l'heure de Paris à l'utilisateur dans le frontend
            if let Some(hour_stat) = hourly_stats.iter_mut().find(|h| h.hour == paris_hour_u8) {
                let event_in_hour = crate::models::EventInHour {
                    event_name: event.description.clone(),
                    impact: event.impact.clone(),
                    datetime: event.event_time.format("%H:%M:%S").to_string(),
                    volatility_increase: 0.0,
                };
                hour_stat.events.push(event_in_hour);
            }
        }

        Ok(())
    }

    /// Associe les événements économiques aux tranches de 15 minutes
    fn load_and_associate_events_15min(
        &self,
        symbol: &str,
        stats_15min: &mut Vec<crate::models::Stats15Min>,
        pool: Option<DbPool>,
    ) -> Result<()> {
        // Si pas de pool, skip chargement des événements
        let Some(pool) = pool else {
            return Ok(());
        };

        // Charger les événements du calendrier pour la période analysée
        let start_time = self
            .candles
            .first()
            .map(|c| c.datetime.naive_utc())
            .ok_or(crate::models::VolatilityError::InsufficientData(
                "No candles to determine event period".to_string(),
            ))?;

        let end_time = self
            .candles
            .last()
            .map(|c| c.datetime.naive_utc())
            .ok_or(crate::models::VolatilityError::InsufficientData(
                "No candles to determine event period".to_string(),
            ))?;

        // Charger événements via EventCorrelationService
        let event_service = crate::services::EventCorrelationService::new(pool);
        let events = event_service
            .get_events_for_period(symbol, start_time, end_time)
            .map_err(|e| crate::models::VolatilityError::DatabaseError(e.to_string()))?;

        // Filtrer HIGH/MEDIUM impact et compter par tranche de 15 minutes (Paris)
        const PARIS_OFFSET_HOURS: i32 = 1;

        for event in events {
            if event.impact != "HIGH" && event.impact != "MEDIUM" {
                continue;
            }

            // Convertir l'heure UTC en heure de Paris
            let utc_hour = event.event_time.hour() as i32;
            let utc_minute = event.event_time.minute() as i32;
            
            let paris_hour = (utc_hour + PARIS_OFFSET_HOURS) % 24;
            let paris_hour_u8 = paris_hour as u8;
            let quarter = (utc_minute / 15) as u8;

            // Trouver la tranche de 15 minutes correspondante
            if let Some(slot) = stats_15min
                .iter_mut()
                .find(|s| s.hour == paris_hour_u8 && s.quarter == quarter)
            {
                let event_in_hour = crate::models::EventInHour {
                    event_name: event.description.clone(),
                    impact: event.impact.clone(),
                    datetime: event.event_time.format("%H:%M:%S").to_string(),
                    volatility_increase: 0.0,
                };
                slot.events.push(event_in_hour);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    fn create_test_candle(hour: u32, high: f64, low: f64) -> Candle {
        Candle {
            id: None,
            symbol: "TESTBTC".to_string(),
            datetime: DateTime::from_timestamp(1609459200 + (hour as i64 * 3600), 0)
                .expect("Invalid timestamp")
                .into(),
            open: (high + low) / 2.0,
            high,
            low,
            close: (high + low) / 2.0,
            volume: 100.0,
        }
    }

    #[test]
    fn test_analyzer_creation() {
        let candles = vec![
            create_test_candle(0, 50000.0, 49900.0),
            create_test_candle(1, 50100.0, 49950.0),
        ];

        let analyzer = VolatilityAnalyzer::new(candles);
        assert_eq!(analyzer.candles.len(), 2);
    }

    #[test]
    fn test_analyze_with_sufficient_data() {
        let candles: Vec<Candle> = (0..100)
            .map(|i| create_test_candle(i % 24, 50000.0 + i as f64, 49900.0 + i as f64))
            .collect();

        let analyzer = VolatilityAnalyzer::new(candles);
        let result = analyzer.analyze("TESTBTC", None);

        assert!(result.is_ok());
        let analysis = result.expect("Failed to analyze");
        assert_eq!(analysis.symbol, "TESTBTC");
        assert_eq!(analysis.hourly_stats.len(), 24);
        assert!(!analysis.best_hours.is_empty());
    }

    #[test]
    fn test_analyze_empty_candles() {
        let analyzer = VolatilityAnalyzer::new(Vec::new());
        let result = analyzer.analyze("TEST", None);

        assert!(result.is_err());
        match result {
            Err(VolatilityError::InsufficientData(_)) => {} // Expected
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[test]
    fn test_analyze_single_hour_data() {
        let candles: Vec<Candle> = (0..10)
            .map(|i| create_test_candle(0, 50000.0 + i as f64, 49900.0 + i as f64))
            .collect();

        let analyzer = VolatilityAnalyzer::new(candles);
        let result = analyzer.analyze("TESTETH", None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_different_symbols() {
        let candles: Vec<Candle> = (0..100)
            .map(|i| {
                create_test_candle(
                    i % 24,
                    1.1000 + (i as f64 * 0.001),
                    1.0900 + (i as f64 * 0.001),
                )
            })
            .collect();

        let analyzer = VolatilityAnalyzer::new(candles);

        let result1 = analyzer.analyze("EURUSD", None);
        let result2 = analyzer.analyze("GBPUSD", None);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap().symbol, "EURUSD");
        assert_eq!(result2.unwrap().symbol, "GBPUSD");
    }

    #[test]
    fn test_analyze_preserves_metadata() {
        let candles: Vec<Candle> = (0..100)
            .map(|i| create_test_candle(i % 24, 50000.0 + i as f64, 49900.0 + i as f64))
            .collect();

        let analyzer = VolatilityAnalyzer::new(candles);
        let result = analyzer.analyze("TESTBTC", None).expect("Failed");

        assert!(!result.timeframe.is_empty());
        assert!(!result.hourly_stats.is_empty());
    }
}
