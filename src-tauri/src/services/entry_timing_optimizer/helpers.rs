use crate::models::{Candle, Result, VolatilityError};
use crate::services::win_rate_calculator::{TradeOutcome, WinRateCalculator};
use chrono::{DateTime, Utc};
use tracing::info;

/// Résultat d'analyse pour un timing spécifique
#[derive(Debug, Clone)]
pub struct TimingAnalysis {
    pub minutes_before: i32,
    #[allow(dead_code)]
    pub win_count: usize,
    #[allow(dead_code)]
    pub loss_count: usize,
    #[allow(dead_code)]
    pub whipsaw_count: usize,
    pub win_rate: f64,
}

/// Résultat complet d'optimisation
#[derive(Debug, Clone)]
pub struct OptimalTimingResult {
    pub best_entry_minutes_before: i32,
    pub best_win_rate: f64,
    pub worst_entry_minutes_before: i32,
    pub worst_win_rate: f64,
    #[allow(dead_code)]
    pub all_timings: Vec<TimingAnalysis>,
}

/// Analyse multiple événements pour statistiques robustes
#[allow(dead_code)]
pub fn analyze_multiple_events(
    events: &[(Vec<Candle>, DateTime<Utc>)],
    atr_multiplier_sl: f64,
    atr_multiplier_tp: f64,
    max_duration_minutes: usize,
) -> Result<OptimalTimingResult> {
    info!("Analyzing {} events for optimal timing", events.len());

    if events.is_empty() {
        return Err(VolatilityError::InsufficientData(
            "No events provided".to_string(),
        ));
    }

    let timings_to_test = vec![60, 45, 30, 15, 5, 1];
    let mut aggregated_results: Vec<TimingAnalysis> = Vec::new();

    for minutes_before in timings_to_test {
        let mut total_wins = 0;
        let mut total_losses = 0;
        let mut total_whipsaws = 0;

        for (candles, event_time) in events {
            let calculator = WinRateCalculator::new(candles, *event_time);

            if let Ok(outcome) = calculator.simulate_trade(
                minutes_before,
                atr_multiplier_sl,
                atr_multiplier_tp,
                max_duration_minutes,
            ) {
                match outcome {
                    TradeOutcome::Win => total_wins += 1,
                    TradeOutcome::Loss => total_losses += 1,
                    TradeOutcome::Whipsaw => total_whipsaws += 1,
                }
            }
        }

        let total = total_wins + total_losses + total_whipsaws;
        let win_rate = if total > 0 {
            total_wins as f64 / total as f64
        } else {
            0.0
        };

        aggregated_results.push(TimingAnalysis {
            minutes_before,
            win_count: total_wins,
            loss_count: total_losses,
            whipsaw_count: total_whipsaws,
            win_rate,
        });
    }

    let best = aggregated_results
        .iter()
        .max_by(|a, b| {
            a.win_rate
                .partial_cmp(&b.win_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or_else(|| VolatilityError::InsufficientData("No results".to_string()))?;

    let worst = aggregated_results
        .iter()
        .min_by(|a, b| {
            a.win_rate
                .partial_cmp(&b.win_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or_else(|| VolatilityError::InsufficientData("No results".to_string()))?;

    Ok(OptimalTimingResult {
        best_entry_minutes_before: best.minutes_before,
        best_win_rate: best.win_rate,
        worst_entry_minutes_before: worst.minutes_before,
        worst_win_rate: worst.win_rate,
        all_timings: aggregated_results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candle(minutes_offset: i32, base_price: f64, range_pips: f64) -> Candle {
        let dt = DateTime::<Utc>::from_timestamp(1609459200 + (minutes_offset as i64 * 60), 0)
            .expect("Invalid timestamp");
        Candle {
            id: None,
            symbol: "TEST".to_string(),
            datetime: dt,
            open: base_price,
            high: base_price + range_pips,
            low: base_price - range_pips,
            close: base_price + (range_pips / 2.0),
            volume: 1000.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_timing_analysis_structure() {
        let ta = TimingAnalysis {
            minutes_before: 15,
            win_count: 10,
            loss_count: 5,
            whipsaw_count: 2,
            win_rate: 0.625,
        };
        assert_eq!(ta.minutes_before, 15);
        assert!(ta.win_rate > 0.0 && ta.win_rate <= 1.0);
    }

    #[test]
    fn test_optimal_timing_result_structure() {
        let result = OptimalTimingResult {
            best_entry_minutes_before: 15,
            best_win_rate: 0.8,
            worst_entry_minutes_before: 60,
            worst_win_rate: 0.4,
            all_timings: vec![],
        };
        assert!(result.best_win_rate >= result.worst_win_rate);
    }

    #[test]
    fn test_analyze_multiple_events_empty() {
        let result = analyze_multiple_events(&[], 2.0, 3.0, 120);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_multiple_events_single() {
        let candles = vec![
            create_test_candle(-60, 1.1000, 0.0010),
            create_test_candle(-30, 1.1000, 0.0010),
            create_test_candle(0, 1.1010, 0.0010),
            create_test_candle(30, 1.1020, 0.0010),
            create_test_candle(60, 1.1000, 0.0010),
        ];
        let event_time = DateTime::<Utc>::from_timestamp(1609459200, 0).expect("Invalid timestamp");

        let result = analyze_multiple_events(&[(candles, event_time)], 2.0, 3.0, 120);
        assert!(result.is_ok());
        let opt = result.expect("Failed");
        assert!(opt.best_win_rate >= 0.0);
        assert!(opt.worst_win_rate >= 0.0);
    }
}
