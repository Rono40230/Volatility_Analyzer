// services/volatility/whipsaw_simulator.rs - Simulation de Straddle pour whipsaw detection
use crate::models::Candle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeResult {
    Win,
    Loss,
    Timeout,
}

/// Simule un trade Straddle réaliste avec TP/SL dynamiques
/// Retourne Win/Loss/Timeout
pub fn simulate_straddle_trade(
    entry_price: f64,
    offset_pips: f64,
    sl_pips: f64,
    tp_pips: f64,
    test_window: &[&Candle],
    pip_value: f64,
) -> TradeResult {
    crate::services::straddle::simulate_straddle_trade(
        entry_price,
        offset_pips,
        sl_pips,
        tp_pips,
        test_window,
        pip_value,
    )
}
