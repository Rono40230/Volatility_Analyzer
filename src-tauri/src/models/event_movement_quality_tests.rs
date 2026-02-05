// tests/event_movement_quality_tests.rs - Tests pour EventMovementQuality
#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::models::event_movement_quality::{EventMovementQuality, MovementRecommendation};

    #[test]
    fn test_event_movement_quality_creation() {
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.75,
            0.15,
            45.2,
            0.78,
            7.5,
            32,
        );
        assert_eq!(quality.symbol, "EURUSD");
        assert_eq!(quality.event_type, "NFP");
        assert!(quality.directional_move_rate > 0.0);
        assert!(quality.whipsaw_rate < 1.0);
    }

    #[test]
    fn test_quality_score_high_recommendation() {
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.85,
            0.10,
            50.0,
            0.82,
            8.5,
            45,
        );
        assert_eq!(
            quality.recommendation(),
            MovementRecommendation::HighQuality
        );
        assert!(quality.is_tradable());
    }

    #[test]
    fn test_quality_score_low_recommendation() {
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "FOMC".to_string(),
            0.45,
            0.55,
            20.0,
            0.35,
            2.5,
            15,
        );
        assert_eq!(quality.recommendation(), MovementRecommendation::AvoidEvent);
        assert!(!quality.is_tradable());
    }

    #[test]
    fn test_movement_recommendation_display() {
        assert_eq!(MovementRecommendation::HighQuality.to_string(), "TRADE");
        assert_eq!(MovementRecommendation::AvoidEvent.to_string(), "AVOID");
    }

    #[test]
    fn test_is_tradable_conditions() {
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.80,
            0.10,
            48.0,
            0.55,
            7.5,
            30,
        );
        assert!(!quality.is_tradable());

        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.65,
            0.35,
            35.0,
            0.70,
            4.5,
            25,
        );
        assert!(!quality.is_tradable());

        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.80,
            0.10,
            48.0,
            0.75,
            7.5,
            30,
        );
        assert!(quality.is_tradable());
    }
}
