use super::CandlesState;
use crate::models::{Candle, EventMetrics};
use crate::services::event_metrics_aggregator::{EventMetricsAggregator, MetricsConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalculateMetricsParams {
    pub event_name: String,
    pub event_time: String,
    pub symbol: String,
    pub atr_period: Option<usize>,
    pub atr_multiplier_sl: Option<f64>,
    pub atr_multiplier_tp: Option<f64>,
    pub max_trade_duration_minutes: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsCommandResult {
    pub success: bool,
    pub metrics: Option<EventMetrics>,
    pub error: Option<String>,
}

pub fn get_unique_symbols(candles: &[Candle]) -> Vec<String> {
    let mut symbols: Vec<String> = candles
        .iter()
        .map(|c| c.symbol.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    symbols.sort();
    symbols
}

#[tauri::command]
pub async fn calculer_metriques_evenement(
    params: CalculateMetricsParams,
    candles_state: State<'_, CandlesState>,
) -> Result<MetricsCommandResult, String> {
    info!("Calculating metrics for event: {}", params.event_name);

    let event_time = DateTime::parse_from_rfc3339(&params.event_time)
        .map_err(|e| format!("Invalid event time format: {}", e))?
        .with_timezone(&Utc);

    let candles_guard = candles_state
        .candles
        .lock()
        .map_err(|e| format!("Failed to lock candles: {}", e))?;

    if candles_guard.is_empty() {
        return Ok(MetricsCommandResult {
            success: false,
            metrics: None,
            error: Some("No candles loaded. Please import data first.".to_string()),
        });
    }

    let symbol_candles: Vec<Candle> = candles_guard
        .iter()
        .filter(|c| c.symbol == params.symbol)
        .cloned()
        .collect();

    if symbol_candles.is_empty() {
        return Ok(MetricsCommandResult {
            success: false,
            metrics: None,
            error: Some(format!(
                "No candles found for symbol {}. Available symbols: {:?}",
                params.symbol,
                get_unique_symbols(&candles_guard)
            )),
        });
    }

    let config = MetricsConfig {
        atr_period: params.atr_period.unwrap_or(14),
        atr_multiplier_sl: params.atr_multiplier_sl.unwrap_or(2.0),
        atr_multiplier_tp: params.atr_multiplier_tp.unwrap_or(3.0),
        max_trade_duration_minutes: params.max_trade_duration_minutes.unwrap_or(120),
    };

    let aggregator =
        EventMetricsAggregator::new(&symbol_candles, event_time, params.event_name.clone());

    match aggregator.calculate_all_metrics(config) {
        Ok(metrics) => {
            info!("Metrics calculated successfully for {}", params.event_name);
            Ok(MetricsCommandResult {
                success: true,
                metrics: Some(metrics),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to calculate metrics: {}", e);
            Ok(MetricsCommandResult {
                success: false,
                metrics: None,
                error: Some(format!("Calculation error: {}", e)),
            })
        }
    }
}
