// services/volatility/metrics.rs - Calcul des métriques globales et scores
// Conforme .clinerules : < 300L, pas d'unwrap()

use super::confidence_scorer::ConfidenceScorer;
use crate::models::{GlobalMetrics, HourlyStats};

/// Calculateur de métriques globales
pub(super) struct MetricsAggregator;

impl MetricsAggregator {
    /// Calcule les métriques globales agrégées
    pub(super) fn calculer_metriques_globales(
        hourly_stats: &[HourlyStats],
        total_candles: usize,
    ) -> GlobalMetrics {
        let stats_with_data: Vec<&HourlyStats> =
            hourly_stats.iter().filter(|h| h.candle_count > 0).collect();

        if stats_with_data.is_empty() {
            return GlobalMetrics {
                mean_atr: 0.0,
                mean_max_true_range: 0.0,
                mean_volatility: 0.0,
                mean_body_range: 0.0,
                mean_noise_ratio: 0.0,
                mean_volume_imbalance: 0.0,
                mean_breakout_percentage: 0.0,
                mean_range: 0.0,
                total_candles: 0,
            };
        }

        let count = stats_with_data.len() as f64;

        // FIX 2.1: Unit Declarations for GlobalMetrics
        // ⬇️ ALL UNITS EXPLICITLY DOCUMENTED ⬇️
        // mean_atr: [Pips/Points] - normalized via symbol_properties.rs
        // mean_max_true_range: [Pips/Points] - 95th percentile of TR
        // mean_volatility: [%] - (ATR/Close) × 100, range 0-100
        // mean_body_range: [%] - body/range ratio, range 0-100
        // mean_noise_ratio: [Ratio] - wicks/body ratio, range 1-10
        // mean_volume_imbalance: [Ratio, 0-1] - FIX 2.1: NOW (body%/100) × (breakout%/100), clean math
        // mean_breakout_percentage: [%] - frequency of ATR breakouts, range 0-100
        // mean_range: [Pips/Points] - normalized high-low average
        
        GlobalMetrics {
            mean_atr: stats_with_data.iter().map(|h| h.atr_mean).sum::<f64>() / count,
            mean_max_true_range: stats_with_data
                .iter()
                .map(|h| h.max_true_range)
                .sum::<f64>()
                / count,
            mean_volatility: stats_with_data
                .iter()
                .map(|h| h.volatility_mean)
                .sum::<f64>()
                / count,
            mean_body_range: stats_with_data
                .iter()
                .map(|h| h.body_range_mean)
                .sum::<f64>()
                / count,
            mean_noise_ratio: stats_with_data
                .iter()
                .map(|h| h.noise_ratio_mean)
                .sum::<f64>()
                / count,
            mean_volume_imbalance: stats_with_data
                .iter()
                .map(|h| h.volume_imbalance_mean)
                .sum::<f64>()
                / count,
            mean_breakout_percentage: stats_with_data
                .iter()
                .map(|h| h.breakout_percentage)
                .sum::<f64>()
                / count,
            mean_range: stats_with_data.iter().map(|h| h.range_mean).sum::<f64>() / count,
            total_candles,
        }
    }

    /// Calcule le score de confiance - DÉLÉGUÉ au ConfidenceScorer
    pub(super) fn calculer_score_confiance(metrics: &GlobalMetrics) -> f64 {
        ConfidenceScorer::calculer_score_confiance(metrics)
    }
}
