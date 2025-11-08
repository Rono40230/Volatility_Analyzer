// services/volatility/metrics.rs - Calcul des métriques globales et scores
// Conforme .clinerules : < 100L, pas d'unwrap()

use crate::models::{GlobalMetrics, HourlyStats};

/// Calculateur de métriques globales
pub(super) struct MetricsAggregator;

impl MetricsAggregator {
    /// Calcule les métriques globales agrégées
    pub(super) fn calculate_global_metrics(hourly_stats: &[HourlyStats], total_candles: usize) -> GlobalMetrics {
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
                total_candles: 0,
            };
        }

        let count = stats_with_data.len() as f64;

        GlobalMetrics {
            mean_atr: stats_with_data.iter().map(|h| h.atr_mean).sum::<f64>() / count,
            mean_volatility: stats_with_data.iter().map(|h| h.volatility_mean).sum::<f64>() / count,
            mean_body_range: stats_with_data.iter().map(|h| h.body_range_mean).sum::<f64>() / count,
            mean_tick_quality: stats_with_data.iter().map(|h| h.tick_quality_mean).sum::<f64>() / count,
            mean_noise_ratio: stats_with_data.iter().map(|h| h.noise_ratio_mean).sum::<f64>() / count,
            mean_volume_imbalance: stats_with_data.iter().map(|h| h.volume_imbalance_mean).sum::<f64>() / count,
            total_candles,
        }
    }

    /// Calcule le score de confiance global (0-100)
    pub(super) fn calculate_confidence_score(metrics: &GlobalMetrics) -> f64 {
        let mut score: f64 = 0.0;

        // 1. Score ATR (25 points max)
        if metrics.mean_atr > 0.001 {
            score += 25.0;
        } else if metrics.mean_atr > 0.0005 {
            score += 15.0;
        } else if metrics.mean_atr > 0.0001 {
            score += 5.0;
        }

        // 2. Score Body Range (25 points max)
        if metrics.mean_body_range > 50.0 {
            score += 25.0;
        } else if metrics.mean_body_range > 30.0 {
            score += 15.0;
        } else if metrics.mean_body_range > 10.0 {
            score += 5.0;
        }

        // 3. Score Tick Quality (20 points max)
        if metrics.mean_tick_quality > 0.001 {
            score += 20.0;
        } else if metrics.mean_tick_quality > 0.0005 {
            score += 10.0;
        }

        // 4. Score Noise Ratio (20 points max) - inverse (moins de bruit = mieux)
        if metrics.mean_noise_ratio < 2.0 {
            score += 20.0;
        } else if metrics.mean_noise_ratio < 3.0 {
            score += 10.0;
        }

        // 5. Pénalité volatilité excessive (10 points)
        if metrics.mean_volatility < 0.15 {
            score += 10.0;
        }

        score.min(100.0)
    }

    /// Trouve les 3 meilleures heures pour trader
    pub(super) fn find_best_hours(hourly_stats: &[HourlyStats]) -> Vec<u8> {
        let mut scored_hours: Vec<(u8, f64)> = hourly_stats
            .iter()
            .filter(|h| h.candle_count > 0)
            .map(|h| (h.hour, h.quality_score()))
            .collect();

        // Trie par score décroissant
        scored_hours.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Retourne les 3 meilleures heures
        scored_hours.iter().take(3).map(|(hour, _)| *hour).collect()
    }
}
