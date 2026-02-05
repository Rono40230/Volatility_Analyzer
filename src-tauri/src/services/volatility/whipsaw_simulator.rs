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
    let offset_points = offset_pips * pip_value;
    let sl_points = sl_pips * pip_value;
    let tp_points = tp_pips * pip_value;

    let buy_stop = entry_price + offset_points;
    let sell_stop = entry_price - offset_points;

    let mut buy_triggered = false;
    let mut sell_triggered = false;
    let mut buy_closed = false;
    let mut sell_closed = false;

    // Parcourir la fenêtre de 60 minutes
    for candle in test_window.iter() {
        // ===== BUY Position =====
        if !buy_triggered && candle.high >= buy_stop {
            buy_triggered = true;
            let buy_sl = buy_stop - sl_points;
            let buy_tp = buy_stop + tp_points;

            // Vérifier SL et TP sur la même candle
            if candle.low <= buy_sl {
                buy_closed = true;
                // Loss
            } else if candle.high >= buy_tp {
                // Win
                return TradeResult::Win;
            }
        }

        // ===== SELL Position =====
        if !sell_triggered && candle.low <= sell_stop {
            sell_triggered = true;
            let sell_sl = sell_stop + sl_points;
            let sell_tp = sell_stop - tp_points;

            // Vérifier SL et TP sur la même candle
            if candle.high >= sell_sl {
                sell_closed = true;
                // Loss
            } else if candle.low <= sell_tp {
                // Win
                return TradeResult::Win;
            }
        }

        // ===== Double Trigger = Whipsaw =====
        if buy_triggered && sell_triggered && (buy_closed || sell_closed) {
            return TradeResult::Loss; // Au moins 1 position SL'd = perte
        }
    }

    // Après 60 candles
    if buy_triggered && sell_triggered {
        // Les 2 se sont déclenchés mais pas de SL hit = timeout (sortie partielle)
        return TradeResult::Timeout;
    }

    // Un seul s'est déclenché = pas de whipsaw, sortie partielle
    TradeResult::Timeout
}
