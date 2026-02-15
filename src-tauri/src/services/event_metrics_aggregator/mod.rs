mod aggregation;
mod config;

pub use aggregation::calculer_metriques_agregees;
pub use config::{volatility_level_to_string, MetricsConfig};

use crate::models::{Candle, EventMetrics, Result};
use crate::services::{
    contextual_atr_analyzer::ContextualAtrAnalyzer, entry_timing_optimizer::EntryTimingOptimizer,
    event_duration_analyzer::EventDurationAnalyzer, win_rate_calculator::WinRateCalculator,
};
use chrono::{DateTime, Utc};
use tracing::info;

/// Agrégateur pour calculer toutes les métriques d'un événement
pub struct EventMetricsAggregator<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
    event_name: String,
}

impl<'a> EventMetricsAggregator<'a> {
    /// Crée un nouvel agrégateur
    pub fn new(candles: &'a [Candle], event_time: DateTime<Utc>, event_name: String) -> Self {
        Self {
            candles,
            event_time,
            event_name,
        }
    }

    /// Calcule toutes les métriques de l'événement
    pub fn calculer_toutes_metriques(&self, config: MetricsConfig) -> Result<EventMetrics> {
        info!(
            "Calculating all metrics for event '{}' at {}",
            self.event_name, self.event_time
        );

        let duration_analyzer = EventDurationAnalyzer::new(self.candles, self.event_time);
        let duration_metrics = duration_analyzer.analyze()?;

        info!(
            "Duration: peak={}min, return={}min",
            duration_metrics.peak_duration_minutes, duration_metrics.return_to_normal_minutes
        );

        let win_rate_calculator = WinRateCalculator::new(self.candles, self.event_time);
        let win_rate_metrics = win_rate_calculator.calculer_taux_reussite(
            15,
            config.atr_multiplier_sl,
            config.atr_multiplier_tp,
            config.max_trade_duration_minutes,
        )?;

        info!(
            "Win rate: {:.1}% ({} wins, {} losses, {} whipsaws)",
            win_rate_metrics.win_rate * 100.0,
            win_rate_metrics.wins,
            win_rate_metrics.losses,
            win_rate_metrics.whipsaws
        );

        let timing_optimizer = EntryTimingOptimizer::new(self.candles, self.event_time);
        let timing_result = timing_optimizer.find_optimal_timing(
            config.atr_multiplier_sl,
            config.atr_multiplier_tp,
            config.max_trade_duration_minutes,
        )?;

        info!(
            "Optimal entry: -{}min ({:.1}% win rate)",
            timing_result.best_entry_minutes_before,
            timing_result.best_win_rate * 100.0
        );

        let atr_analyzer = ContextualAtrAnalyzer::new(self.candles, self.event_time);
        let atr_metrics = atr_analyzer.analyze(config.atr_period)?;

        info!(
            "ATR: before={:.5}, after={:.5}, ratio={:.2}x",
            atr_metrics.atr_before_event, atr_metrics.atr_after_event, atr_metrics.atr_ratio
        );

        let metrics = EventMetrics {
            id: None,
            event_name: self.event_name.clone(),
            event_time: self.event_time,
            symbol: self
                .candles
                .first()
                .map(|c| c.symbol.clone())
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            peak_duration_minutes: duration_metrics.peak_duration_minutes,
            return_to_normal_minutes: duration_metrics.return_to_normal_minutes,
            peak_time_minutes: duration_metrics.peak_time_minutes,
            baseline_atr: duration_metrics.baseline_atr,
            win_rate: win_rate_metrics.win_rate,
            loss_rate: win_rate_metrics.losses as f64
                / (win_rate_metrics.wins + win_rate_metrics.losses + win_rate_metrics.whipsaws)
                    as f64,
            whipsaw_rate: win_rate_metrics.whipsaw_rate,
            risk_reward_ratio: win_rate_metrics.risk_reward_ratio,
            best_entry_minutes_before: timing_result.best_entry_minutes_before,
            best_entry_win_rate: timing_result.best_win_rate,
            worst_entry_minutes_before: timing_result.worst_entry_minutes_before,
            worst_entry_win_rate: timing_result.worst_win_rate,
            atr_before_event: atr_metrics.atr_before_event,
            atr_after_event: atr_metrics.atr_after_event,
            atr_ratio: atr_metrics.atr_ratio,
            max_atr_spike: atr_metrics.max_atr_spike,
            recommended_sl_multiplier: atr_metrics.recommended_sl_multiplier,
            recommended_tp_multiplier: atr_metrics.recommended_tp_multiplier,
            baseline_volatility: volatility_level_to_string(&atr_metrics.baseline_volatility_level),
            sample_size: 1,
            created_at: Utc::now(),
        };

        info!("All metrics calculated successfully");
        Ok(metrics)
    }

    #[allow(dead_code)]
    pub fn calculer_metriques_agregees(
        events: Vec<(Vec<Candle>, DateTime<Utc>)>,
        event_name: String,
        config: MetricsConfig,
    ) -> Result<EventMetrics> {
        calculer_metriques_agregees(events, event_name, config)
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
            ..Default::default()
        }
    }

    #[test]
    fn test_calculer_toutes_metriques() {
        let mut candles = Vec::new();

        for i in 0..80 {
            candles.push(create_test_candle(-(80 - i), 1.1000, 0.0010));
        }

        for i in 0..120 {
            let increased_range = 0.0030;
            candles.push(create_test_candle(i, 1.1000, increased_range));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp");
        let aggregator =
            EventMetricsAggregator::new(&candles, event_time, "Test Event".to_string());

        let config = MetricsConfig::default();
        let result = aggregator.calculer_toutes_metriques(config);

        assert!(result.is_ok());
        let metrics = result.expect("Failed to calculate metrics");

        assert_eq!(metrics.event_name, "Test Event");
        assert_eq!(metrics.symbol, "EURUSD");
        assert!(metrics.win_rate >= 0.0 && metrics.win_rate <= 1.0);
        assert!(metrics.atr_ratio >= 0.0);
    }
}
