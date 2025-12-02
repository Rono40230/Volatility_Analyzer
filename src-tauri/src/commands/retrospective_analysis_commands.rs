use crate::models::Candle;
use crate::services::VolatilityDurationAnalyzer;
use crate::services::retrospective_helpers::*;
use serde::{Deserialize, Serialize};

/// Peak delay analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakDelayResult {
    pub event_type: String,
    pub peak_delay_minutes: i16,
    pub sample_count: usize,
    pub consistency_percent: f64,
}

/// Decay profile analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayProfileResult {
    pub peak_atr: f64,
    pub decay_rate_pips_per_minute: f64,
    pub decay_speed: String,
    pub recommended_timeout_minutes: i16,
}

/// Entry timing profitability result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryTimingResult {
    pub entry_offset_minutes: i16,
    pub win_rate: f64,
    pub whipsaw_rate: f64,
    pub avg_profit_pips: f64,
    pub sample_size: usize,
    pub quality_score: f64,
    pub is_best: bool,
}

/// Directional bias analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalBiasResult {
    pub up_wins_count: usize,
    pub down_wins_count: usize,
    pub whipsaw_count: usize,
    pub bias_value: f64,
    pub asymmetry_percent: f64,
    pub classification: String,
    pub confidence_level: String,
}

/// Whipsaw root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhipsawRootCauseResult {
    pub early_count: usize,
    pub early_percentage: f64,
    pub early_avg_loss_pips: f64,
    pub late_count: usize,
    pub late_percentage: f64,
    pub late_avg_loss_pips: f64,
    pub total_whipsaws: usize,
    pub dominant_type: String,
}

#[tauri::command]
pub async fn analyze_peak_delay(
    candles: Vec<Candle>,
    event_minute: usize,
) -> Result<DecayProfileResult, String> {
    if candles.is_empty() {
        return Err("No candle data provided".to_string());
    }

    let atr_values: Vec<f64> = candles
        .iter()
        .map(|c| (c.high - c.low).max((c.high - c.close.abs()).max(c.close - c.low.abs())))
        .collect();

    let peak_delay = VolatilityDurationAnalyzer::calculate_peak_delay(&atr_values, event_minute as u8)
        .unwrap_or(0);
    let (decay_rate, decay_speed) = VolatilityDurationAnalyzer::calculate_decay_profile(&atr_values)
        .map_err(|e| format!("Decay calculation failed: {}", e))?;

    let peak_atr = atr_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let timeout = (peak_delay as i32 + 25) as i16;

    Ok(DecayProfileResult { peak_atr, decay_rate_pips_per_minute: decay_rate, decay_speed, recommended_timeout_minutes: timeout })
}

#[tauri::command]
pub async fn analyze_decay_profile(candles: Vec<Candle>) -> Result<DecayProfileResult, String> {
    if candles.is_empty() {
        return Err("No candle data provided".to_string());
    }

    let atr_values: Vec<f64> = candles
        .iter()
        .map(|c| (c.high - c.low).max((c.high - c.close.abs()).max(c.close - c.low.abs())))
        .collect();

    let peak_atr = atr_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (decay_rate, decay_speed) = VolatilityDurationAnalyzer::calculate_decay_profile(&atr_values)
        .map_err(|e| format!("Decay calculation failed: {}", e))?;

    let timeout = if decay_rate > 3.0 { 18 } else if decay_rate > 1.5 { 25 } else { 32 };

    Ok(DecayProfileResult { peak_atr, decay_rate_pips_per_minute: decay_rate, decay_speed, recommended_timeout_minutes: timeout })
}

#[tauri::command]
pub async fn analyze_entry_timing(results: Vec<(i16, f64, bool)>) -> Result<Vec<EntryTimingResult>, String> {
    if results.is_empty() {
        return Err("No backtest results provided".to_string());
    }

    let metrics = calculate_entry_timing_metrics(&results);
    let mut timing_results = Vec::new();

    for offset in &[-10i16, -5, 0, 3] {
        if let Some(&(win_rate, avg_profit, whipsaw_rate, size)) = metrics.get(offset) {
            let quality_score = calculate_quality_score(win_rate, avg_profit, whipsaw_rate);
            timing_results.push(EntryTimingResult {
                entry_offset_minutes: *offset, win_rate, whipsaw_rate, avg_profit_pips: avg_profit,
                sample_size: size, quality_score, is_best: false,
            });
        }
    }

    if let Some(best) = timing_results.iter_mut().max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap_or(std::cmp::Ordering::Equal)) {
        best.is_best = true;
    }

    Ok(timing_results)
}

#[tauri::command]
pub async fn analyze_directional_bias(up_wins: usize, down_wins: usize, whipsaws: usize) -> Result<DirectionalBiasResult, String> {
    let total_events = up_wins + down_wins + whipsaws;
    if up_wins + down_wins == 0 {
        return Err("No win data provided".to_string());
    }

    let (bias_value, asymmetry_percent, classification) = calculate_bias_metrics(up_wins, down_wins, total_events);
    let confidence_level = get_confidence_level(total_events);

    Ok(DirectionalBiasResult {
        up_wins_count: up_wins, down_wins_count: down_wins, whipsaw_count: whipsaws,
        bias_value, asymmetry_percent, classification: classification.to_string(),
        confidence_level: confidence_level.to_string(),
    })
}

#[tauri::command]
pub async fn analyze_whipsaw_root_cause(early_whipsaws: usize, early_avg_loss: f64, late_whipsaws: usize, late_avg_loss: f64) -> Result<WhipsawRootCauseResult, String> {
    let total = early_whipsaws + late_whipsaws;
    if total == 0 {
        return Err("No whipsaw data provided".to_string());
    }

    let early_pct = (early_whipsaws as f64 / total as f64) * 100.0;
    let late_pct = (late_whipsaws as f64 / total as f64) * 100.0;
    let dominant_type = classify_whipsaw_type(early_whipsaws, late_whipsaws);

    Ok(WhipsawRootCauseResult {
        early_count: early_whipsaws, early_percentage: early_pct, early_avg_loss_pips: early_avg_loss,
        late_count: late_whipsaws, late_percentage: late_pct, late_avg_loss_pips: late_avg_loss,
        total_whipsaws: total, dominant_type: dominant_type.to_string(),
    })
}

