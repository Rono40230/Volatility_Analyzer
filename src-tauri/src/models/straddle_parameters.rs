use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraddleParameters {
    pub offset_pips: f64,
    pub stop_loss_pips: f64,
    pub trailing_stop_pips: f64,
    pub timeout_minutes: i32,
    pub sl_recovery_pips: f64,
    pub hard_tp_pips: f64, // Nouveau: Take Profit fixe
    pub risk_reward_ratio: f64,
    pub spread_safety_margin_pips: f64,
}
