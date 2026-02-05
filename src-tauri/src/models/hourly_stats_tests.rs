// tests/hourly_stats_tests.rs - Tests pour HourlyStats
#[cfg(test)]
mod tests {
    use crate::models::HourlyStats;

    #[test]
    fn test_quality_score_calculation() {
        let stats = HourlyStats {
            hour: 13,
            candle_count: 500,
            atr_mean: 0.0015,
            atr_max: 0.003,
            max_true_range: 0.004,
            volatility_mean: 0.10,
            range_mean: 0.002,
            body_range_mean: 55.0,
            shadow_ratio_mean: 1.2,
            volume_imbalance_mean: 0.15,
            noise_ratio_mean: 1.8,
            breakout_percentage: 25.0,
            events: Vec::new(),
        };

        let score = stats.quality_score();
        assert!((0.0..=100.0).contains(&score));
        assert!(score > 50.0);
    }

    #[test]
    fn test_quality_score_empty() {
        let stats = HourlyStats {
            hour: 0,
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
        };

        assert_eq!(stats.quality_score(), 0.0);
    }

    #[test]
    fn test_quality_rating() {
        let excellent = HourlyStats {
            hour: 13,
            candle_count: 500,
            atr_mean: 15.0, // 15 pips
            atr_max: 20.0,
            max_true_range: 25.0,
            volatility_mean: 0.12,
            range_mean: 18.0,
            body_range_mean: 60.0,
            shadow_ratio_mean: 1.5,
            volume_imbalance_mean: 0.2,
            noise_ratio_mean: 1.5,
            breakout_percentage: 30.0,
            events: Vec::new(),
        };

        assert_eq!(excellent.quality_rating(), "Excellent");
    }
}
