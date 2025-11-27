// commands/volatility/straddle_metrics_types.rs - Types pour TÂCHE 5
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StraddleMetricsResponse {
    pub symbol: String,
    pub hour: u8,
    pub candle_count: usize,
    pub offset_optimal: OptimalOffsetData,
    pub win_rate: WinRateData,
    pub whipsaw: WhipsawData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimalOffsetData {
    pub offset_pips: f64,
    pub percentile_95_wicks: f64,
    pub with_margin: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WinRateData {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhipsawData {
    pub total_trades: usize,
    pub whipsaw_count: usize,
    pub whipsaw_frequency_percentage: f64,
    pub risk_level: String,
    pub risk_color: String,
}

/// Calcule le risque et la couleur basé sur la fréquence whipsaw
#[allow(dead_code)]
pub fn calculate_risk_level(whipsaw_freq_pct: f64) -> (String, String) {
    let risk_level = match whipsaw_freq_pct {
        x if x < 5.0 => "Très Bas",
        x if x < 10.0 => "Bas",
        x if x < 20.0 => "Modéré",
        x if x < 30.0 => "Élevé",
        _ => "Très Élevé",
    };

    let risk_color = match risk_level {
        "Très Bas" => "#22c55e",
        "Bas" => "#84cc16",
        "Modéré" => "#f59e0b",
        "Élevé" => "#ef4444",
        "Très Élevé" => "#7f1d1d",
        _ => "#6b7280",
    };

    (risk_level.to_string(), risk_color.to_string())
}
