// services/volatility/metrics.rs - Calcul des métriques globales et scores
// Conforme .clinerules : < 300L, pas d'unwrap()

use super::best_hours_finder::BestHoursFinder;
use super::confidence_scorer::ConfidenceScorer;
use crate::models::{GlobalMetrics, HourlyStats};

/// Calculateur de métriques globales
pub(super) struct MetricsAggregator;

impl MetricsAggregator {
    /// Calcule les métriques globales agrégées
    pub(super) fn calculate_global_metrics(
        hourly_stats: &[HourlyStats],
        total_candles: usize,
    ) -> GlobalMetrics {
        let stats_with_data: Vec<&HourlyStats> =
            hourly_stats.iter().filter(|h| h.candle_count > 0).collect();

        if stats_with_data.is_empty() {
            return GlobalMetrics {
                mean_atr: 0.0,
                mean_volatility: 0.0,
                mean_body_range: 0.0,
                mean_tick_quality: 0.0,
                mean_noise_ratio: 0.0,
                mean_volume_imbalance: 0.0,
                mean_breakout_percentage: 0.0,
                total_candles: 0,
            };
        }

        let count = stats_with_data.len() as f64;

        GlobalMetrics {
            mean_atr: stats_with_data.iter().map(|h| h.atr_mean).sum::<f64>() / count,
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
            mean_tick_quality: stats_with_data
                .iter()
                .map(|h| h.tick_quality_mean)
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
            total_candles,
        }
    }

    /// Calcule le score de confiance - DÉLÉGUÉ au ConfidenceScorer
    pub(super) fn calculate_confidence_score(metrics: &GlobalMetrics) -> f64 {
        ConfidenceScorer::calculate_confidence_score(metrics)
    }

    /// Trouve les meilleures heures - DÉLÉGUÉ au BestHoursFinder
    pub(super) fn find_best_hours(hourly_stats: &[HourlyStats]) -> Vec<u8> {
        BestHoursFinder::find_best_hours(hourly_stats)
    }
}
