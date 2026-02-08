use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub stop_loss_pips: f64,
    pub tp_rr: f64,
    pub trailing_atr_coef: f64,
    pub atr_period: i32,
    pub trailing_refresh_seconds: i32,
    pub timeout_minutes: i32,
    pub sl_recovery_pips: Option<f64>, // Uniquement pour le mode Simultané
    pub spread_pips: f64,              // Spread simulé (ex: 1.0 pip)
    pub slippage_pips: f64,            // Slippage simulé (ex: 0.5 pip)
    pub point_value: f64,              // Valeur du point (ex: 0.00001)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TradeOutcome {
    TakeProfit,
    StopLoss,
    Timeout,
    NoEntry, // L'ordre n'a jamais été déclenché
    Whipsaw, // Déclenché puis SL touché rapidement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub event_date: String, // ISO 8601
    pub entry_time: String, // Heure d'entrée
    pub exit_time: String,  // Heure de sortie
    pub duration_minutes: i32,
    pub pips_net: f64,      // Résultat net (après spread)
    pub outcome: TradeOutcome,
    pub max_favorable_excursion: f64, // MFE (en pips)
    pub max_adverse_excursion: f64,   // MAE (en pips)
    pub logs: Vec<String>, // Journal d'exécution pour comprendre le trade
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub symbol: String,
    pub event_name: String,
    pub unit: String, // Unité d'affichage (pips, points, $)
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub no_entries: usize,
    pub win_rate_percent: f64,
    pub total_pips: f64,
    pub average_pips_per_trade: f64,
    pub max_drawdown_pips: f64,
    pub profit_factor: f64,
    pub trades: Vec<TradeResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction { Long, Short }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub direction: Direction,
    pub entry_price: f64,
    pub entry_time: chrono::DateTime<chrono::Utc>,
    pub stop_loss: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub mfe: f64,
    pub mae: f64,
}
