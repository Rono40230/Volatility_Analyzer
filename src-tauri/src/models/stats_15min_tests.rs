// tests/stats_15min_tests.rs - Tests pour Stats15Min
#[cfg(test)]
mod tests {
    use crate::models::Stats15Min;

    #[test]
    fn test_time_label() {
        let stats = Stats15Min {
            hour: 14,
            quarter: 0,
            candle_count: 0,
            atr_mean: 0.0,
            atr_max: 0.0,
            volatility_mean: 0.0,
            range_mean: 0.0,
            body_range_mean: 0.0,
            shadow_ratio_mean: 0.0,
            volume_imbalance_mean: 0.0,
            noise_ratio_mean: 0.0,
            breakout_percentage: 0.0,
            events: Vec::new(),
            peak_duration_minutes: None,
            volatility_half_life_minutes: None,
            recommended_trade_expiration_minutes: None,
            peak_duration_mean: None,
            volatility_half_life_mean: None,
            recommended_trade_expiration_mean: None,
            max_true_range: 0.0,
            p95_wick: 0.0,
            straddle_parameters: None,
            volatility_profile: None,
            optimal_entry_minute: None,
        };
        assert_eq!(stats.time_label(), "14:00-14:15");

        let stats = Stats15Min {
            hour: 14,
            quarter: 2,
            ..stats
        };
        assert_eq!(stats.time_label(), "14:30-14:45");
    }

    #[test]
    fn test_quality_score() {
        let stats = Stats15Min {
            hour: 14,
            quarter: 0,
            candle_count: 100,
            atr_mean: 3.0, // 3 pips
            atr_max: 5.0,
            volatility_mean: 0.15,
            range_mean: 8.0,
            body_range_mean: 50.0,
            shadow_ratio_mean: 1.2,
            volume_imbalance_mean: 0.15,
            noise_ratio_mean: 1.8,
            breakout_percentage: 25.0,
            events: Vec::new(),
            peak_duration_minutes: None,
            volatility_half_life_minutes: None,
            recommended_trade_expiration_minutes: None,
            peak_duration_mean: None,
            volatility_half_life_mean: None,
            recommended_trade_expiration_mean: None,
            max_true_range: 0.0,
            p95_wick: 0.0,
            straddle_parameters: None,
            volatility_profile: None,
            optimal_entry_minute: None,
        };

        let score = stats.quality_score();
        assert!((0.0..=100.0).contains(&score));
        assert!(score > 50.0);
    }

    #[test]
    fn test_quality_score_empty() {
        let stats = Stats15Min {
            hour: 0,
            quarter: 0,
            candle_count: 0,
            atr_mean: 0.0,
            atr_max: 0.0,
            volatility_mean: 0.0,
            range_mean: 0.0,
            body_range_mean: 0.0,
            shadow_ratio_mean: 0.0,
            volume_imbalance_mean: 0.0,
            noise_ratio_mean: 0.0,
            breakout_percentage: 0.0,
            events: Vec::new(),
            peak_duration_minutes: None,
            volatility_half_life_minutes: None,
            recommended_trade_expiration_minutes: None,
            peak_duration_mean: None,
            volatility_half_life_mean: None,
            recommended_trade_expiration_mean: None,
            max_true_range: 0.0,
            p95_wick: 0.0,
            straddle_parameters: None,
            volatility_profile: None,
            optimal_entry_minute: None,
        };

        assert_eq!(stats.quality_score(), 0.0);
    }
}
