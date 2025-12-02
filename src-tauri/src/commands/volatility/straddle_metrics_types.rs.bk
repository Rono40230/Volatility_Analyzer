// commands/volatility/straddle_metrics_types.rs - Types pour TÂCHE 5
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WhipsawDetailResponse {
    pub entry_candle_index: usize,
    pub trigger_minute: i32,
    pub entry_price: f64,
    pub buy_stop: f64,
    pub sell_stop: f64,
}

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
    /// Stop Loss ajusté par whipsaw: SL × (1 + whipsaw_frequency × 0.3)
    pub sl_adjusted_pips: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WinRateData {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate_percentage: f64,
    /// Win Rate ajusté par la fréquence whipsaw: WR × (1 - whipsaw_frequency)
    pub win_rate_adjusted: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhipsawData {
    pub total_trades: usize,
    pub whipsaw_count: usize,
    pub whipsaw_frequency_percentage: f64,
    pub risk_level: String,
    pub risk_color: String,
    /// Stop Loss ajusté par whipsaw
    pub sl_adjusted_pips: f64,
    /// Win Rate ajusté par whipsaw
    pub win_rate_adjusted: f64,
    /// Trailing Stop ajusté par whipsaw: 1.59 × (1 - whipsaw_frequency / 2)
    pub trailing_stop_adjusted: f64,
    /// Timeout ajusté par whipsaw: 32 min × (1 - whipsaw_frequency × 0.5)
    pub timeout_adjusted_minutes: i32,
    /// Meilleur moment d'entrée (minutes après début du quarter) basé sur analyse whipsaw
    pub optimal_entry_minutes: i32,
    /// Détails de chaque whipsaw détecté
    pub whipsaw_details: Vec<WhipsawDetailResponse>,
}


