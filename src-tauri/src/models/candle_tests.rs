// tests/candle_tests.rs - Tests pour Candle
#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::models::candle::Candle;
    use chrono::Utc;

    #[test]
    fn test_candle_creation() {
        let candle = Candle::new(
            "EURUSD".to_string(),
            Utc::now(),
            1.0950,
            1.0980,
            1.0940,
            1.0970,
            1200.0,
        );
        assert!(candle.is_ok());
    }

    #[test]
    fn test_body_range() {
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0950,
            close: 1.0970,
            high: 1.0980,
            low: 1.0940,
            volume: 1200.0,
        };
        let body_range = candle.body_range();
        assert!((body_range - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_body_range_bearish() {
        // Test pour vérifier que Body Range n'est jamais négatif pour une bougie baissière
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0970,
            close: 1.0950,
            high: 1.0980,
            low: 1.0940,
            volume: 1200.0,
        };
        let body_range = candle.body_range();
        // Body Range doit toujours être positif: |close - open| / range * 100
        assert!(body_range >= 0.0);
        assert!((body_range - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_true_range() {
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0950,
            close: 1.0970,
            high: 1.0980,
            low: 1.0940,
            volume: 1200.0,
        };
        let tr = candle.true_range(Some(1.0960));
        assert!(tr > 0.0);
    }

    #[test]
    fn test_shadow_ratio() {
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0950,
            close: 1.0970,
            high: 1.0980,
            low: 1.0940,
            volume: 1200.0,
        };
        let ratio = candle.shadow_ratio();
        assert!(ratio >= 0.0);
    }

    #[test]
    fn test_invalid_candle_high_low() {
        let result = Candle::new(
            "EURUSD".to_string(),
            Utc::now(),
            1.0950,
            1.0940,
            1.0980,
            1.0970,
            1200.0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_hour_utc() {
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0,
            high: 1.1,
            low: 0.9,
            close: 1.05,
            volume: 1000.0,
        };
        assert!(candle.hour_utc() < 24);
    }
}
