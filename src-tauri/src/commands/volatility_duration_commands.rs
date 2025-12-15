use crate::models::{Stats15Min, VolatilityDuration};
use crate::services::VolatilityDurationAnalyzer;

/// Analyzes volatility duration characteristics for a given time period
///
/// Takes a Stats15Min and calculates:
/// - Peak duration (how long volatility stays above 80% of peak)
/// - Volatility half-life (time for volatility to decay 50%)
/// - Recommended trade expiration (max of peak_duration and half_life × 2)
/// - Confidence score (50-100% based on sample size)
///
/// # Arguments
/// * `stats_15min` - Stats15Min containing volatility data
///
/// # Returns
/// * `Result<VolatilityDuration, String>` - Analysis result or error message
#[tauri::command]
pub fn analyze_volatility_duration(stats: Stats15Min) -> Result<VolatilityDuration, String> {
    VolatilityDurationAnalyzer::analyser(&stats)
        .map_err(|e| format!("Failed to analyze volatility duration: {}", e))
}
