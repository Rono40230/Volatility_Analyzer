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
}

impl<'a> HourlyStatsCalculator<'a> {
    pub(super) fn new(candles: &'a [Candle]) -> Self {
        Self { candles }
    }

    /// Calcule les statistiques pour chaque heure (en heure de Paris, UTC+1)
    /// Les candles sont en UTC, on les groupe par heure de Paris pour une meilleure analyse pour les traders français
    pub(super) fn calculate(&self) -> Result<Vec<HourlyStats>> {
        debug!("Calculating hourly statistics (Paris time)");

        const PARIS_OFFSET_HOURS: i32 = 1; // UTC+1 (hiver) - TODO: gérer DST pour l'été

        // Groupe les bougies par heure de Paris
        let mut hourly_groups: HashMap<u8, Vec<&Candle>> = HashMap::new();

        for candle in self.candles {
            let utc_hour = candle.hour_utc() as i32;
            let paris_hour = (utc_hour + PARIS_OFFSET_HOURS) % 24;
            let paris_hour_u8 = paris_hour as u8;
            hourly_groups.entry(paris_hour_u8).or_default().push(candle);
        }

        // Calcule les stats pour chaque heure de Paris
        let mut stats = Vec::new();

        for hour in 0..24 {
            if let Some(candles) = hourly_groups.get(&hour) {
                let hour_stats = self.calculate_for_hour(hour, candles)?;
                stats.push(hour_stats);
            } else {
                // Heure sans données : stats vides
                stats.push(HourlyStats {
                    hour,
                    candle_count: 0,
                    atr_mean: 0.0,
                    atr_max: 0.0,
                    volatility_mean: 0.0,
                    range_mean: 0.0,
                    body_range_mean: 0.0,
                    shadow_ratio_mean: 0.0,
                    tick_quality_mean: 0.0,
                    volume_imbalance_mean: 0.0,
                    noise_ratio_mean: 0.0,
                    breakout_percentage: 0.0,
                    events: Vec::new(),
                });
            }
        }

        debug!("Calculated stats for {} hours (Paris time)", stats.len());
        Ok(stats)
    }

    /// Calcule les statistiques pour une heure spécifique
    fn calculate_for_hour(&self, hour: u8, candles: &[&Candle]) -> Result<HourlyStats> {
        let candle_count = candles.len();

        // Crée un vecteur owned pour le calculateur
        let owned_candles: Vec<Candle> = candles.iter().map(|&c| c.clone()).collect();
        let calc = MetricsCalculator::new(&owned_candles);

        // Calcule les métriques (avec gestion d'erreur si pas assez de données)
        let atr_values = calc.calculate_atr(14).unwrap_or_default();
        let volatility_values = calc.calculate_volatility(20).unwrap_or_default();
        let body_ranges = calc.calculate_body_ranges();
        let shadow_ratios = calc.calculate_shadow_ratios();
        let tick_qualities = calc.calculate_tick_quality();
        let volume_imbalances = calc.calculate_volume_imbalance(14).unwrap_or_default();
        let noise_ratios = calc.calculate_noise_ratio();
        let tr_dist = calc.calculate_true_range_distribution()?;

        // Calcule les moyennes
        let atr_mean = mean(&atr_values);
        let atr_max = max(&atr_values);
        let volatility_mean = mean(&volatility_values);
        let range_mean =
            owned_candles.iter().map(|c| c.high - c.low).sum::<f64>() / owned_candles.len() as f64;
        let body_range_mean = mean(&body_ranges);
        let shadow_ratio_mean = mean(&shadow_ratios);
        let tick_quality_mean = mean(&tick_qualities);
        let volume_imbalance_mean = mean(&volume_imbalances);
        let noise_ratio_mean = mean(&noise_ratios);

        let breakout_count = tr_dist.is_breakout.iter().filter(|&&b| b).count();
        let breakout_percentage =
            (breakout_count as f64 / tr_dist.is_breakout.len() as f64) * 100.0;

        Ok(HourlyStats {
            hour,
            candle_count,
            atr_mean,
            atr_max,
            volatility_mean,
            range_mean,
            body_range_mean,
            shadow_ratio_mean,
            tick_quality_mean,
            volume_imbalance_mean,
            noise_ratio_mean,
            breakout_percentage,
            events: Vec::new(), // Sera rempli après par l'analyseur
        })
    }
}
