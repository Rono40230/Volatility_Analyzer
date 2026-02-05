// tests/volatility_duration_tests.rs - Tests pour VolatilityDuration
#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::models::volatility_duration::VolatilityDuration;

    #[test]
    fn test_volatility_duration_creation() {
        let vd = VolatilityDuration::new(14, 2, 150, 60, 85);
        assert_eq!(vd.hour, 14);
        assert_eq!(vd.quarter, 2);
        assert_eq!(vd.peak_duration_minutes, 150);
        assert_eq!(vd.volatility_half_life_minutes, 60);
        assert_eq!(vd.recommended_trade_expiration_minutes, 150);
        assert_eq!(vd.sample_size, 85);
        assert!(vd.is_valid());
    }

    #[test]
    fn test_trade_expiration_max_logic() {
        let vd = VolatilityDuration::new(10, 0, 100, 80, 50);
        assert_eq!(vd.recommended_trade_expiration_minutes, 160);
    }

    #[test]
    fn test_confidence_score_by_sample_size() {
        assert_eq!(
            VolatilityDuration::new(14, 0, 150, 60, 150).confidence_score,
            100
        );
        assert_eq!(
            VolatilityDuration::new(14, 0, 150, 60, 85).confidence_score,
            90
        );
        assert_eq!(
            VolatilityDuration::new(14, 0, 150, 60, 40).confidence_score,
            75
        );
        assert_eq!(
            VolatilityDuration::new(14, 0, 150, 60, 20).confidence_score,
            60
        );
        assert_eq!(
            VolatilityDuration::new(14, 0, 150, 60, 10).confidence_score,
            50
        );
    }

    #[test]
    fn test_time_label_formatting() {
        let vd = VolatilityDuration::new(14, 2, 150, 60, 50);
        assert_eq!(vd.time_label(), "14:30-14:45");

        let vd2 = VolatilityDuration::new(9, 0, 150, 60, 50);
        assert_eq!(vd2.time_label(), "09:00-09:15");
    }

    #[test]
    fn test_validation() {
        let valid = VolatilityDuration::new(14, 2, 150, 60, 50);
        assert!(valid.is_valid());

        let mut invalid = valid.clone();
        invalid.peak_duration_minutes = 0;
        assert!(!invalid.is_valid());

        let mut invalid = valid.clone();
        invalid.volatility_half_life_minutes = 150;
        assert!(!invalid.is_valid());
    }
}
