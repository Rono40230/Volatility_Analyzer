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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
        };
        assert!(candle.hour_utc() < 24);
    }

    #[test]
    fn test_true_range_exact_value() {
        // TR = max(H-L, |H-prevC|, |L-prevC|)
        // = max(1.0980-1.0940, |1.0980-1.0960|, |1.0940-1.0960|)
        // = max(0.004, 0.002, 0.002) = 0.004
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0950,
            close: 1.0970,
            high: 1.0980,
            low: 1.0940,
            volume: 1200.0,
            ..Default::default()
        };
        let tr = candle.true_range(Some(1.0960));
        assert!((tr - 0.004).abs() < 1e-10);
    }

    #[test]
    fn test_true_range_with_gap_down_exact() {
        // prev_close = 1.1100, H = 1.1050, L = 1.1020
        // TR = max(0.003, |1.1050-1.1100|=0.005, |1.1020-1.1100|=0.008) = 0.008
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.1040,
            high: 1.1050,
            low: 1.1020,
            close: 1.1030,
            volume: 100.0,
            ..Default::default()
        };
        let tr = candle.true_range(Some(1.1100));
        assert!((tr - 0.008).abs() < 1e-10);
    }

    #[test]
    fn test_true_range_nan_prev_close_does_not_panic() {
        // Si prev_close est NaN, la fonction ne doit pas paniquer.
        // En Rust, f64::max() utilise IEEE total ordering : max(x, NaN) = x
        // Donc le résultat est au moins H-L (le range brut survit).
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.1000,
            high: 1.1050,
            low: 1.1000,
            close: 1.1020,
            volume: 100.0,
            ..Default::default()
        };
        let tr = candle.true_range(Some(f64::NAN));
        // f64::max en Rust: max(0.005, NaN) = 0.005 (NaN perd toujours)
        // Le résultat est le range H-L = 0.005
        assert!((tr - 0.005).abs() < 1e-10, "Should fallback to H-L range, got {}", tr);
    }

    #[test]
    fn test_shadow_ratio_doji() {
        // Doji : open == close == high == low → upper=0, lower=0 → retourne 1.0
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.1000,
            high: 1.1000,
            low: 1.1000,
            close: 1.1000,
            volume: 100.0,
            ..Default::default()
        };
        assert!((candle.shadow_ratio() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_shadow_ratio_hammer_no_upper_wick() {
        // Hammer : open=close=high, lower wick exists → ratio = 0.0
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.1050,
            high: 1.1050,
            low: 1.1000,
            close: 1.1050,
            volume: 100.0,
            ..Default::default()
        };
        assert!((candle.shadow_ratio() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_shadow_ratio_no_lower_wick_returns_infinity() {
        // Pas de mèche basse (close=low), mèche haute existe
        // upper_wick = 1.1050 - 1.1020 = 0.003, lower_wick = 1.1000 - 1.1000 = 0
        // → f64::INFINITY
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.1000,
            high: 1.1050,
            low: 1.1000,
            close: 1.1020,
            volume: 100.0,
            ..Default::default()
        };
        // upper = 1.1050 - max(1.1000,1.1020) = 1.1050-1.1020 = 0.003
        // lower = min(1.1000,1.1020) - 1.1000 = 0
        assert!(candle.shadow_ratio().is_infinite());
    }

    #[test]
    fn test_shadow_ratio_normal() {
        // upper_wick = 1.1080 - max(1.1050, 1.1060) = 1.1080 - 1.1060 = 0.002
        // lower_wick = min(1.1050, 1.1060) - 1.1040 = 1.1050 - 1.1040 = 0.001
        // ratio = 0.002 / 0.001 = 2.0
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.1050,
            high: 1.1080,
            low: 1.1040,
            close: 1.1060,
            volume: 100.0,
            ..Default::default()
        };
        assert!((candle.shadow_ratio() - 2.0).abs() < 1e-10);
    }
}
