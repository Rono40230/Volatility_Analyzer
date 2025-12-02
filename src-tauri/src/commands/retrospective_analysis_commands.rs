use crate::models::Candle;
use crate::services::VolatilityDurationAnalyzer;
use serde::{Deserialize, Serialize};

/// Peak delay analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakDelayResult {
    pub peak_delay_minutes: i16,
    pub peak_atr: f64,
    pub event_minute: u8,
    pub confidence: f64,
    pub event_count: usize,
    pub event_type: String,
}

/// Decay profile analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayProfileResult {
    pub peak_atr: f64,
    pub decay_rate_pips_per_minute: f64,
    pub decay_speed: String,
    pub recommended_timeout_minutes: i16,
    pub event_count: usize,
    pub event_type: String,
}

/// Available event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeList {
    pub types: Vec<String>,
}



#[tauri::command]
pub async fn analyze_peak_delay(candles: Vec<Candle>, event_type: String) -> Result<PeakDelayResult, String> {
    if candles.is_empty() { return Err("No candle data provided".to_string()) }
    let atr_values: Vec<f64> = candles.iter().map(|c| (c.high - c.low).max((c.high - c.close.abs()).max(c.close - c.low.abs()))).collect();
    let event_minute = 0u8;
    let peak_delay = VolatilityDurationAnalyzer::calculate_peak_delay(&atr_values, event_minute).unwrap_or(0);
    let peak_atr = atr_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let confidence = if peak_atr > 1.0 { 0.95 } else if peak_atr > 0.5 { 0.75 } else { 0.5 };
    Ok(PeakDelayResult { peak_delay_minutes: peak_delay, peak_atr, event_minute, confidence, event_count: candles.len(), event_type })
}

#[tauri::command]
pub async fn analyze_decay_profile(candles: Vec<Candle>, event_type: String) -> Result<DecayProfileResult, String> {
    if candles.is_empty() { return Err("No candle data provided".to_string()) }
    let atr_values: Vec<f64> = candles.iter().map(|c| (c.high - c.low).max((c.high - c.close.abs()).max(c.close - c.low.abs()))).collect();
    let peak_atr = atr_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (decay_rate, decay_speed) = VolatilityDurationAnalyzer::calculate_decay_profile(&atr_values)
        .map_err(|e| format!("Decay calculation failed: {}", e))?;
    let timeout = if decay_rate > 3.0 { 18 } else if decay_rate > 1.5 { 25 } else { 32 };
    Ok(DecayProfileResult { peak_atr, decay_rate_pips_per_minute: decay_rate, decay_speed, recommended_timeout_minutes: timeout, event_count: candles.len(), event_type })
}

#[tauri::command]
pub async fn get_event_types() -> Result<EventTypeList, String> {
    Ok(EventTypeList {
        types: vec!["NFP".to_string(), "Inflation".to_string(), "Unemployment".to_string(), "GDP".to_string(), "Retail Sales".to_string()],
    })
}

