mod helpers;

pub use helpers::{analyze_multiple_events, OptimalTimingResult, TimingAnalysis};

use crate::models::{Candle, Result, VolatilityError};
use crate::services::win_rate_calculator::{TradeOutcome, WinRateCalculator};
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// Optimiseur de timing d'entrée
pub struct EntryTimingOptimizer<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
}

impl<'a> EntryTimingOptimizer<'a> {
    /// Crée un nouvel optimiseur
    pub fn new(candles: &'a [Candle], event_time: DateTime<Utc>) -> Self {
        Self {
            candles,
            event_time,
        }
    }

    /// Analyse tous les timings possibles et trouve l'optimal
    pub fn find_optimal_timing(
        &self,
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<OptimalTimingResult> {
        info!(
            "Finding optimal entry timing for event at {}",
            self.event_time
        );

        let timings_to_test = vec![60, 45, 30, 15, 5, 1];
        let mut results = Vec::new();

        for minutes_before in timings_to_test {
            debug!("Testing entry at -{} minutes", minutes_before);

            let analysis = self.analyze_timing(
                minutes_before,
                atr_multiplier_sl,
                atr_multiplier_tp,
                max_duration_minutes,
            )?;

            results.push(analysis);
        }

        let best = results
            .iter()
            .max_by(|a, b| {
                a.win_rate
                    .partial_cmp(&b.win_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| VolatilityError::InsufficientData("No timing results".to_string()))?;

        let worst = results
            .iter()
            .min_by(|a, b| {
                a.win_rate
                    .partial_cmp(&b.win_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| VolatilityError::InsufficientData("No timing results".to_string()))?;

        info!(
            "Optimal timing: -{}min ({}% win rate), Worst: -{}min ({}% win rate)",
            best.minutes_before,
            (best.win_rate * 100.0).round(),
            worst.minutes_before,
            (worst.win_rate * 100.0).round()
        );

        Ok(OptimalTimingResult {
            best_entry_minutes_before: best.minutes_before,
            best_win_rate: best.win_rate,
            worst_entry_minutes_before: worst.minutes_before,
            worst_win_rate: worst.win_rate,
            all_timings: results,
        })
    }

    /// Analyse un timing spécifique
    fn analyze_timing(
        &self,
        minutes_before: i32,
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<TimingAnalysis> {
        let calculator = WinRateCalculator::new(self.candles, self.event_time);

        let outcome = calculator.simulate_trade(
            minutes_before,
            atr_multiplier_sl,
            atr_multiplier_tp,
            max_duration_minutes,
        )?;

        let (wins, losses, whipsaws) = match outcome {
            TradeOutcome::Win => (1, 0, 0),
            TradeOutcome::Loss => (0, 1, 0),
            TradeOutcome::Whipsaw => (0, 0, 1),
        };

        let total = wins + losses + whipsaws;
        let win_rate = if total > 0 {
            wins as f64 / total as f64
        } else {
            0.0
        };

        Ok(TimingAnalysis {
            minutes_before,
            win_count: wins,
            loss_count: losses,
            whipsaw_count: whipsaws,
            win_rate,
        })
    }

    #[allow(dead_code)]
    pub fn analyze_multiple_events(
        events: &[(Vec<Candle>, DateTime<Utc>)],
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<OptimalTimingResult> {
        analyze_multiple_events(
            events,
            atr_multiplier_sl,
            atr_multiplier_tp,
            max_duration_minutes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candle(minutes_offset: i64, price: f64, range: f64) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: DateTime::from_timestamp(1609459200 + (minutes_offset * 60), 0)
                .expect("Invalid timestamp"),
            open: price,
            high: price + range,
            low: price - range,
            close: price + range / 2.0,
            volume: 100.0,
        }
    }

    #[test]
    fn test_find_optimal_timing() {
        let mut candles = Vec::new();

        for i in 0..80 {
            candles.push(create_test_candle(-(80 - i), 1.1000, 0.0010));
        }

        for i in 0..120 {
            let price = 1.1000 + (i as f64 * 0.00005);
            candles.push(create_test_candle(i, price, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp");
        let optimizer = EntryTimingOptimizer::new(&candles, event_time);

        let result = optimizer
            .find_optimal_timing(2.0, 3.0, 120)
            .expect("Failed to find timing");

        assert_eq!(result.all_timings.len(), 6);
        assert!(result.best_win_rate >= result.worst_win_rate);
        assert!(result.best_entry_minutes_before > 0);
    }

    #[test]
    fn test_timing_analysis() {
        let mut candles = Vec::new();

        for i in 0..80 {
            candles.push(create_test_candle(-(80 - i), 1.1000, 0.0010));
        }

        for i in 0..120 {
            let price = 1.1000 + (i as f64 * 0.0001);
            candles.push(create_test_candle(i, price, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp");
        let optimizer = EntryTimingOptimizer::new(&candles, event_time);

        let analysis = optimizer
            .analyze_timing(15, 2.0, 3.0, 60)
            .expect("Failed to analyze");

        assert_eq!(analysis.minutes_before, 15);
        assert!(analysis.win_rate >= 0.0 && analysis.win_rate <= 1.0);
    }

    #[test]
    fn test_analyze_timing_different_minutes() {
        let mut candles = Vec::new();
        for i in 0..80 {
            candles.push(create_test_candle(-(80 - i), 1.1000, 0.0010));
        }
        for i in 0..120 {
            candles.push(create_test_candle(i, 1.1000 + (i as f64 * 0.00001), 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp");
        let optimizer = EntryTimingOptimizer::new(&candles, event_time);

        for minutes in &[1, 5, 15, 30, 45, 60] {
            let analysis = optimizer.analyze_timing(*minutes, 2.0, 3.0, 60);
            assert!(analysis.is_ok());
        }
    }

    #[test]
    fn test_find_optimal_timing_consistency() {
        let mut candles = Vec::new();
        for i in 0..80 {
            candles.push(create_test_candle(-(80 - i), 1.1000, 0.0010));
        }
        for i in 0..180 {
            candles.push(create_test_candle(i, 1.1000, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp");
        let optimizer = EntryTimingOptimizer::new(&candles, event_time);

        let result1 = optimizer
            .find_optimal_timing(2.0, 3.0, 120)
            .expect("should find timing");
        let result2 = optimizer
            .find_optimal_timing(2.0, 3.0, 120)
            .expect("should find timing");

        assert_eq!(
            result1.best_entry_minutes_before,
            result2.best_entry_minutes_before
        );
        assert_eq!(result1.best_win_rate, result2.best_win_rate);
    }

    #[test]
    fn test_entry_timing_optimizer_empty_candles() {
        let candles = Vec::new();
        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp");
        let optimizer = EntryTimingOptimizer::new(&candles, event_time);

        let result = optimizer.find_optimal_timing(2.0, 3.0, 120);
        assert!(result.is_err());
    }
}
