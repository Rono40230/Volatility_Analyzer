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
                mean_breakout_percentage: 0.0,
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
            mean_breakout_percentage: stats_with_data.iter().map(|h| h.breakout_percentage).sum::<f64>() / count,
            total_candles,
        }
    }

    /// Calcule le score de confiance global (0-100)
    /// Adapté aux réalités du Forex scalping sur M1
    pub(super) fn calculate_confidence_score(metrics: &GlobalMetrics) -> f64 {
        let mut score: f64 = 0.0;

        // 1. Score ATR (30 points max) - Seuils adaptés au Forex M1
        // ATR Forex M1 typique : 0.00010 - 0.00030 (10-30 pips)
        if metrics.mean_atr > 0.00025 {
            score += 30.0;  // Excellent : >25 pips
        } else if metrics.mean_atr > 0.00015 {
            score += 25.0;  // Très bon : 15-25 pips
        } else if metrics.mean_atr > 0.00010 {
            score += 20.0;  // Bon : 10-15 pips
        } else if metrics.mean_atr > 0.00005 {
            score += 10.0;  // Acceptable : 5-10 pips
        }

        // 2. Score Body Range (25 points max) - Seuils réalistes
        // Body Range Forex : 25-45% est normal, >45% est excellent
        if metrics.mean_body_range > 45.0 {
            score += 25.0;  // Excellent : mouvements directionnels forts
        } else if metrics.mean_body_range > 35.0 {
            score += 20.0;  // Très bon
        } else if metrics.mean_body_range > 25.0 {
            score += 15.0;  // Bon
        } else if metrics.mean_body_range > 15.0 {
            score += 8.0;   // Acceptable
        }

        // 3. Score Volatilité (25 points max) - BONUS si volatile
        // Plus c'est volatil, MIEUX c'est pour le scalping !
        if metrics.mean_volatility > 0.30 {
            score += 25.0;  // Excellent : cryptos, exotiques
        } else if metrics.mean_volatility > 0.20 {
            score += 20.0;  // Très bon : paires majeures volatiles
        } else if metrics.mean_volatility > 0.10 {
            score += 15.0;  // Bon : volatilité correcte
        } else if metrics.mean_volatility > 0.05 {
            score += 8.0;   // Acceptable
        }

        // 4. Score Noise Ratio (10 points max) - Signal/Bruit
        if metrics.mean_noise_ratio < 2.0 {
            score += 10.0;  // Excellent : signal propre
        } else if metrics.mean_noise_ratio < 3.0 {
            score += 7.0;   // Bon
        } else if metrics.mean_noise_ratio < 4.0 {
            score += 4.0;   // Acceptable
        }

        // 5. Score Breakout % (10 points max) - CRITIQUE pour Straddle
        // % de bougies qui cassent significativement (>P80 ATR)
        if metrics.mean_breakout_percentage > 15.0 {
            score += 10.0;  // Excellent : mouvements forts fréquents
        } else if metrics.mean_breakout_percentage > 10.0 {
            score += 7.0;   // Très bon
        } else if metrics.mean_breakout_percentage > 5.0 {
            score += 4.0;   // Acceptable
        }

        // 6. Bonus données suffisantes (5 points max)
        if metrics.total_candles > 100000 {
            score += 5.0;   // Données suffisantes pour fiabilité
        } else if metrics.total_candles > 50000 {
            score += 3.0;
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
