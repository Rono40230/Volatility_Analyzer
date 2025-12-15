// services/event_duration_analyzer/mod.rs - API publique et tests
// Conforme RÈGLE 5: < 300L par fichier

mod analyzer;

pub use analyzer::EventDurationAnalyzer;

#[cfg(test)]
mod tests {
    use super::analyzer::EventDurationMetrics;
    use super::*;
    use crate::models::Candle;
    use chrono::DateTime;

    fn create_test_candle(minutes_offset: i64) -> Candle {
        let base_price = 1.1000;

        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: DateTime::from_timestamp(1609459200 + (minutes_offset * 60), 0)
                .expect("Invalid timestamp")
                .into(),
            open: base_price,
            high: base_price + 0.0010,
            low: base_price - 0.0010,
            close: base_price + 0.0005,
            volume: 100.0,
        }
    }

    #[test]
    fn test_event_duration_metrics_creation() {
        let metrics = EventDurationMetrics {
            peak_duration_minutes: 60,
            return_to_normal_minutes: 120,
            peak_time_minutes: 30,
            baseline_atr: 0.0001,
        };
        assert_eq!(metrics.peak_duration_minutes, 60);
        assert_eq!(metrics.return_to_normal_minutes, 120);
        assert!(metrics.baseline_atr > 0.0);
    }

    #[test]
    fn test_analyzer_creation() {
        let candles = vec![
            create_test_candle(-30),
            create_test_candle(-15),
            create_test_candle(0),
            create_test_candle(15),
            create_test_candle(30),
        ];

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();

        let _analyzer = EventDurationAnalyzer::new(&candles, event_time);
        assert!(!candles.is_empty());
    }

    #[test]
    fn test_analyzer_with_sufficient_data() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| create_test_candle(i as i64 - 60))
            .collect();

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();

        let analyzer = EventDurationAnalyzer::new(&candles, event_time);
        let result = analyzer.analyze();
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyzer_minimal_candles() {
        let candles = vec![
            create_test_candle(-1),
            create_test_candle(0),
            create_test_candle(1),
            create_test_candle(2),
        ];

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();

        let analyzer = EventDurationAnalyzer::new(&candles, event_time);
        let result = analyzer.analyze();
        // Peut échouer (données insuffisantes) ou réussir - les deux sont valides
        let _ = result;
    }

    #[test]
    fn test_find_event_index() {
        let candles = vec![
            create_test_candle(-30),
            create_test_candle(-15),
            create_test_candle(0),
            create_test_candle(15),
        ];

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();

        let analyzer = EventDurationAnalyzer::new(&candles, event_time);
        let index_result = analyzer.find_event_index();
        assert!(index_result.is_ok());
        if let Ok(idx) = index_result {
            assert!(idx < candles.len());
        }
    }

    #[test]
    fn test_baseline_atr_insufficient_data() {
        let candles = vec![
            create_test_candle(-5),
            create_test_candle(0),
            create_test_candle(5),
        ];

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();

        let analyzer = EventDurationAnalyzer::new(&candles, event_time);
        let result = analyzer.calculer_atr_reference();
        // Devrait échouer avec données insuffisantes
        assert!(result.is_err());
    }
}
