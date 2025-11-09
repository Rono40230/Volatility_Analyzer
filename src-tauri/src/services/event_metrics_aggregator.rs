// services/event_metrics_aggregator.rs - Agrégateur de toutes les métriques d'événement
// Conforme .clinerules : < 300L, pas d'unwrap()

use crate::models::{Candle, EventMetrics, Result, VolatilityError};
use crate::services::{
    contextual_atr_analyzer::{ContextualAtrAnalyzer, VolatilityLevel},
    event_duration_analyzer::EventDurationAnalyzer,
    entry_timing_optimizer::EntryTimingOptimizer,
    win_rate_calculator::WinRateCalculator,
};
use chrono::{DateTime, Utc};
use tracing::{info, warn};

/// Agrégateur pour calculer toutes les métriques d'un événement
pub struct EventMetricsAggregator<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
    event_name: String,
}

/// Configuration pour le calcul des métriques
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub atr_period: usize,
    pub atr_multiplier_sl: f64,
    pub atr_multiplier_tp: f64,
    pub max_trade_duration_minutes: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            atr_period: 14,
            atr_multiplier_sl: 2.0,
            atr_multiplier_tp: 3.0,
            max_trade_duration_minutes: 120,
        }
    }
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
    pub fn calculate_all_metrics(&self, config: MetricsConfig) -> Result<EventMetrics> {
        info!(
            "Calculating all metrics for event '{}' at {}",
            self.event_name, self.event_time
        );

        // 1. Analyse de durée
        let duration_analyzer = EventDurationAnalyzer::new(self.candles, self.event_time);
        let duration_metrics = duration_analyzer.analyze()?;

        info!(
            "Duration: peak={}min, return={}min",
            duration_metrics.peak_duration_minutes, duration_metrics.return_to_normal_minutes
        );

        // 2. Calcul du win rate
        let win_rate_calculator = WinRateCalculator::new(self.candles, self.event_time);
        let win_rate_metrics = win_rate_calculator.calculate_win_rate(
            15, // Entry à -15min par défaut
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

        // 3. Optimisation du timing d'entrée
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

        // 4. Analyse ATR contextuel
        let atr_analyzer = ContextualAtrAnalyzer::new(self.candles, self.event_time);
        let atr_metrics = atr_analyzer.analyze(config.atr_period)?;

        info!(
            "ATR: before={:.5}, after={:.5}, ratio={:.2}x",
            atr_metrics.atr_before_event, atr_metrics.atr_after_event, atr_metrics.atr_ratio
        );

        // Assemblage final
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
            sample_size: 1, // Pour un événement unique
            created_at: Utc::now(),
        };

        info!("All metrics calculated successfully");
        Ok(metrics)
    }

    /// Calcule les métriques agrégées pour plusieurs occurrences du même événement
    /// NOTE: Cette fonction est conservée pour usage futur (agrégation multi-événements)
    #[allow(dead_code)]
    pub fn calculate_aggregated_metrics(
        events: Vec<(Vec<Candle>, DateTime<Utc>)>,
        event_name: String,
        config: MetricsConfig,
    ) -> Result<EventMetrics> {
        if events.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "No events provided for aggregation".to_string(),
            ));
        }

        info!(
            "Calculating aggregated metrics for {} occurrences of '{}'",
            events.len(),
            event_name
        );

        // Calculer métriques individuelles
        let mut all_metrics = Vec::new();

        for (candles, event_time) in &events {
            let aggregator = EventMetricsAggregator::new(candles, *event_time, event_name.clone());

            match aggregator.calculate_all_metrics(config.clone()) {
                Ok(metrics) => all_metrics.push(metrics),
                Err(e) => {
                    warn!("Failed to calculate metrics for event at {}: {}", event_time, e);
                    continue;
                }
            }
        }

        if all_metrics.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "Could not calculate metrics for any event".to_string(),
            ));
        }

        // Agréger les résultats (moyenne)
        let n = all_metrics.len() as f64;

        let aggregated = EventMetrics {
            id: None,
            event_name: event_name.clone(),
            event_time: events[0].1, // Premier événement comme référence
            symbol: all_metrics[0].symbol.clone(),
            peak_duration_minutes: (all_metrics
                .iter()
                .map(|m| m.peak_duration_minutes as f64)
                .sum::<f64>()
                / n) as i32,
            return_to_normal_minutes: (all_metrics
                .iter()
                .map(|m| m.return_to_normal_minutes as f64)
                .sum::<f64>()
                / n) as i32,
            peak_time_minutes: (all_metrics
                .iter()
                .map(|m| m.peak_time_minutes as f64)
                .sum::<f64>()
                / n) as i64,
            baseline_atr: all_metrics.iter().map(|m| m.baseline_atr).sum::<f64>() / n,
            win_rate: all_metrics.iter().map(|m| m.win_rate).sum::<f64>() / n,
            loss_rate: all_metrics.iter().map(|m| m.loss_rate).sum::<f64>() / n,
            whipsaw_rate: all_metrics.iter().map(|m| m.whipsaw_rate).sum::<f64>() / n,
            risk_reward_ratio: all_metrics.iter().map(|m| m.risk_reward_ratio).sum::<f64>() / n,
            best_entry_minutes_before: (all_metrics
                .iter()
                .map(|m| m.best_entry_minutes_before as f64)
                .sum::<f64>()
                / n) as i32,
            best_entry_win_rate: all_metrics
                .iter()
                .map(|m| m.best_entry_win_rate)
                .sum::<f64>()
                / n,
            worst_entry_minutes_before: (all_metrics
                .iter()
                .map(|m| m.worst_entry_minutes_before as f64)
                .sum::<f64>()
                / n) as i32,
            worst_entry_win_rate: all_metrics
                .iter()
                .map(|m| m.worst_entry_win_rate)
                .sum::<f64>()
                / n,
            atr_before_event: all_metrics
                .iter()
                .map(|m| m.atr_before_event)
                .sum::<f64>()
                / n,
            atr_after_event: all_metrics.iter().map(|m| m.atr_after_event).sum::<f64>() / n,
            atr_ratio: all_metrics.iter().map(|m| m.atr_ratio).sum::<f64>() / n,
            max_atr_spike: all_metrics.iter().map(|m| m.max_atr_spike).sum::<f64>() / n,
            recommended_sl_multiplier: all_metrics
                .iter()
                .map(|m| m.recommended_sl_multiplier)
                .sum::<f64>()
                / n,
            recommended_tp_multiplier: all_metrics
                .iter()
                .map(|m| m.recommended_tp_multiplier)
                .sum::<f64>()
                / n,
            baseline_volatility: all_metrics[0].baseline_volatility.clone(),
            sample_size: all_metrics.len() as i32,
            created_at: Utc::now(),
        };

        info!(
            "Aggregated metrics calculated from {} samples",
            all_metrics.len()
        );
        Ok(aggregated)
    }
}

/// Convertit VolatilityLevel en String pour la DB
fn volatility_level_to_string(level: &VolatilityLevel) -> String {
    match level {
        VolatilityLevel::Low => "Low".to_string(),
        VolatilityLevel::Medium => "Medium".to_string(),
        VolatilityLevel::High => "High".to_string(),
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
                .expect("Invalid timestamp")
                .into(),
            open: price,
            high: price + range,
            low: price - range,
            close: price + range / 2.0,
            volume: 100.0,
        }
    }

    #[test]
    fn test_calculate_all_metrics() {
        let mut candles = Vec::new();

        // 60min avant
        for i in 0..60 {
            candles.push(create_test_candle(-(60 - i), 1.1000, 0.0010));
        }

        // 120min après
        for i in 0..120 {
            let increased_range = 0.0030;
            candles.push(create_test_candle(i, 1.1000, increased_range));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let aggregator =
            EventMetricsAggregator::new(&candles, event_time, "Test Event".to_string());

        let config = MetricsConfig::default();
        let result = aggregator.calculate_all_metrics(config);

        assert!(result.is_ok());
        let metrics = result.unwrap();

        assert_eq!(metrics.event_name, "Test Event");
        assert_eq!(metrics.symbol, "EURUSD");
        assert!(metrics.win_rate >= 0.0 && metrics.win_rate <= 1.0);
        assert!(metrics.atr_ratio >= 0.0);
    }
}
