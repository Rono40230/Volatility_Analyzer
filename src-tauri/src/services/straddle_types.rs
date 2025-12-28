use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhipsawDetail {
    pub entry_index: usize,
    pub entry_price: f64,
    pub buy_stop: f64,
    pub sell_stop: f64,
    pub buy_trigger_index: usize,
    pub sell_trigger_index: usize,
    pub net_loss_pips: f64, // Perte réelle incluant spread/slippage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraddleSimulationResult {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate_percentage: f64,
    pub whipsaw_frequency_percentage: f64,
    pub offset_optimal_pips: f64,
    pub percentile_95_wicks: f64,
    pub risk_level: String,
    pub risk_color: String,
    // Valeurs pondérées par le whipsaw (Option B - affichage direct)
    pub win_rate_adjusted: f64,        // Win Rate pondéré par whipsaw
    pub sl_adjusted_pips: f64,         // SL ajusté par whipsaw
    pub trailing_stop_adjusted: f64,   // Trailing Stop réduit
    pub timeout_adjusted_minutes: i32, // Timeout réduit
    pub whipsaw_details: Vec<WhipsawDetail>, // Détails de chaque whipsaw
    
    // Nouvelles métriques financières (Réalité News Trading)
    pub total_pnl_net_pips: f64,       // P&L Net cumulé
    pub avg_trade_cost_pips: f64,      // Coût moyen par trade (Spread + Slippage)
    pub is_profitable_net: bool,       // Si P&L Net > 0

    // Indicateurs de Confiance (Priority 4)
    pub confidence_score: f64,         // Score 0-100%
    pub sample_size_warning: bool,     // Alerte si < 5 événements
}
