// services/volatility/hourly_stats.rs - Calcul des statistiques horaires
// Conforme .clinerules : < 150L, pas d'unwrap()

use super::utils::{max, mean};
use crate::models::{Candle, HourlyStats, Result};
use crate::services::MetricsCalculator;
use std::collections::HashMap;
use tracing::debug;

/// Calculateur de statistiques horaires
pub(super) struct HourlyStatsCalculator<'a> {
    candles: &'a [Candle],
    symbol: String,
}

impl<'a> HourlyStatsCalculator<'a> {
    pub(super) fn new(candles: &'a [Candle], symbol: String) -> Self {
        Self { candles, symbol }
    }

    /// Calcule les statistiques pour chaque heure (en UTC)
    /// Les candles sont en UTC, on les groupe par heure UTC
    pub(super) fn calculer(&self) -> Result<Vec<HourlyStats>> {
        debug!("Calculating hourly statistics (UTC)");

        // Groupe les bougies par heure UTC
        let mut hourly_groups: HashMap<u8, Vec<&Candle>> = HashMap::new();

        for candle in self.candles {
            let utc_hour = candle.hour_utc() as u8;
            hourly_groups.entry(utc_hour).or_default().push(candle);
        }

        // Calcule les stats pour chaque heure UTC
        let mut stats = Vec::new();

        for hour in 0..24 {
            if let Some(candles) = hourly_groups.get(&hour) {
                let hour_stats = self.calculer_pour_heure(hour, candles)?;
                stats.push(hour_stats);
            } else {
                // Heure sans données : stats vides
                stats.push(HourlyStats {
                    hour,
                    candle_count: 0,
                    atr_mean: 0.0,
                    atr_max: 0.0,
                    max_true_range: 0.0,
                    volatility_mean: 0.0,
                    range_mean: 0.0,
                    body_range_mean: 0.0,
                    shadow_ratio_mean: 0.0,
                    volume_imbalance_mean: 0.0,
                    noise_ratio_mean: 0.0,
                    breakout_percentage: 0.0,
                    events: Vec::new(),
                });
            }
        }

        debug!("Calculated stats for {} hours (UTC)", stats.len());
        Ok(stats)
    }

    /// Calcule les statistiques pour une heure spécifique
    fn calculer_pour_heure(&self, hour: u8, candles: &[&Candle]) -> Result<HourlyStats> {
        let candle_count = candles.len();

        // Crée un vecteur owned pour le calculateur
        let owned_candles: Vec<Candle> = candles.iter().map(|&c| c.clone()).collect();
        let calc = MetricsCalculator::new(&owned_candles);

        // Calcule les métriques (avec gestion d'erreur si pas assez de données)
        let atr_values = calc.calculer_atr(14).unwrap_or_default();
        let volatility_values = calc.calculer_volatilite(20).unwrap_or_default();
        let body_ranges = calc.calculer_ranges_corps();
        let shadow_ratios = calc.calculer_ratios_ombres();
        let noise_ratios = calc.calculer_ratio_bruit();
        let tr_dist = calc.calculer_distribution_true_range()?;

        // Normalisation des valeurs (Pips/Points) — DB override en priorité
        let asset_props = crate::services::pair_data::symbol_properties::get_asset_properties(&self.symbol);

        // Calcule les moyennes
        let raw_atr_mean = mean(&atr_values); // FIX-01: Moyenne au lieu de last()
        let raw_atr_max = max(&atr_values);
        let raw_range_mean = mean(&tr_dist.true_ranges);
        let raw_max_true_range = tr_dist.percentile_95; // FIX-01: Max Spike stabilisé (95e percentile)

        let atr_mean = asset_props.normalize(raw_atr_mean);
        let atr_max = asset_props.normalize(raw_atr_max);
        let max_true_range = asset_props.normalize(raw_max_true_range);
        let volatility_mean = mean(&volatility_values); // En % (ne pas normaliser)
        let range_mean = asset_props.normalize(raw_range_mean);
        
        let body_range_mean = mean(&body_ranges);
        let shadow_ratio_mean = mean(&shadow_ratios);
        let noise_ratio_mean = mean(&noise_ratios);

        // Calculate breakout percentage first
        let breakout_count = tr_dist.is_breakout.iter().filter(|&&b| b).count();
        let breakout_percentage =
            (breakout_count as f64 / tr_dist.is_breakout.len() as f64) * 100.0;

        // Direction Strength: Force directionnelle [Ratio, 0-1]
        // = (Body Range % / 100) × (Breakout % / 100)
        // Exemple: 45% body × 18% breakout = 0.45 × 0.18 = 0.081 [ratio]
        // Sémantique: Score combiné de pureté directionnelle (0=aucun, 1=parfait)
        // Frontend affiche en % en multipliant par 100: 0.081 → 8.1%
        let direction_strength = (body_range_mean / 100.0) * (breakout_percentage / 100.0);

        Ok(HourlyStats {
            hour,
            candle_count,
            atr_mean,
            atr_max,
            max_true_range,
            volatility_mean,
            range_mean,
            body_range_mean,
            shadow_ratio_mean,
            volume_imbalance_mean: direction_strength, // Remplacé par direction_strength
            noise_ratio_mean,
            breakout_percentage,
            events: Vec::new(), // Sera rempli après par l'analyseur
        })
    }
}
