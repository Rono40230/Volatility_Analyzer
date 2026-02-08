// services/straddle/trade_simulation.rs
// Types et fonctions de simulation de trades straddle extraits de implementation.rs

use crate::models::Candle;

// --- TradeOutcome: résultat d'une simulation de trade straddle ---
#[derive(Debug, Clone)]
pub struct TradeOutcome {
    pub result: String,
    pub buy_trigger_idx: usize,
    pub sell_trigger_idx: usize,
}

// --- TradeResult: résultat simplifié pour le simulateur whipsaw ---
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeResult {
    Win,
    Loss,
    Timeout,
}

/// Simule le résultat d'un trade straddle à partir d'un index de départ.
/// Renvoie None si aucun côté n'est déclenché avant max_duration.
pub fn simulate_trade_outcome(
    candles: &[Candle],
    start_idx: usize,
    buy_stop: f64,
    sell_stop: f64,
    tp_distance: f64,
    sl_distance: f64,
    max_duration: usize,
) -> Option<TradeOutcome> {
    let end_idx = candles.len().min(start_idx + max_duration + 1);
    let mut triggered_side: Option<&str> = None;
    let mut buy_trigger_idx = 0usize;
    let mut sell_trigger_idx = 0usize;

    for (j, current) in candles.iter().enumerate().take(end_idx).skip(start_idx + 1) {
        if triggered_side.is_none() {
            if current.high >= buy_stop && current.low <= sell_stop {
                return Some(TradeOutcome { result: "WHIPSAW".to_string(), buy_trigger_idx: j, sell_trigger_idx: j });
            } else if current.high >= buy_stop {
                triggered_side = Some("BUY");
                buy_trigger_idx = j;
                if current.low <= sell_stop {
                    return Some(TradeOutcome { result: "WHIPSAW".to_string(), buy_trigger_idx: j, sell_trigger_idx: j });
                }
                if current.low <= buy_stop - sl_distance {
                    return Some(TradeOutcome { result: "LOSS".to_string(), buy_trigger_idx: j, sell_trigger_idx: 0 });
                }
                if current.high >= buy_stop + tp_distance {
                    return Some(TradeOutcome { result: "WIN".to_string(), buy_trigger_idx: j, sell_trigger_idx: 0 });
                }
            } else if current.low <= sell_stop {
                triggered_side = Some("SELL");
                sell_trigger_idx = j;
                if current.high >= buy_stop {
                    return Some(TradeOutcome { result: "WHIPSAW".to_string(), buy_trigger_idx: j, sell_trigger_idx: j });
                }
                if current.high >= sell_stop + sl_distance {
                    return Some(TradeOutcome { result: "LOSS".to_string(), buy_trigger_idx: 0, sell_trigger_idx: j });
                }
                if current.low <= sell_stop - tp_distance {
                    return Some(TradeOutcome { result: "WIN".to_string(), buy_trigger_idx: 0, sell_trigger_idx: j });
                }
            }
        } else {
            match triggered_side {
                Some("BUY") => {
                    if current.low <= sell_stop {
                        return Some(TradeOutcome { result: "WHIPSAW".to_string(), buy_trigger_idx, sell_trigger_idx: j });
                    }
                    if current.low <= buy_stop - sl_distance {
                        return Some(TradeOutcome { result: "LOSS".to_string(), buy_trigger_idx, sell_trigger_idx: 0 });
                    }
                    if current.high >= buy_stop + tp_distance {
                        return Some(TradeOutcome { result: "WIN".to_string(), buy_trigger_idx, sell_trigger_idx: 0 });
                    }
                }
                Some("SELL") => {
                    if current.high >= buy_stop {
                        return Some(TradeOutcome { result: "WHIPSAW".to_string(), buy_trigger_idx: j, sell_trigger_idx });
                    }
                    if current.high >= sell_stop + sl_distance {
                        return Some(TradeOutcome { result: "LOSS".to_string(), buy_trigger_idx: 0, sell_trigger_idx });
                    }
                    if current.low <= sell_stop - tp_distance {
                        return Some(TradeOutcome { result: "WIN".to_string(), buy_trigger_idx: 0, sell_trigger_idx });
                    }
                }
                _ => {}
            }
        }
    }

    if triggered_side.is_some() {
        Some(TradeOutcome { result: "TIMEOUT".to_string(), buy_trigger_idx, sell_trigger_idx })
    } else {
        None
    }
}

/// Simule un trade straddle simplifié pour le détecteur whipsaw.
/// Utilise des pips et une fenêtre de bougies de test.
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

    for candle in test_window.iter() {
        if !buy_triggered && candle.high >= buy_stop {
            buy_triggered = true;
            let buy_sl = buy_stop - sl_points;
            let buy_tp = buy_stop + tp_points;
            if candle.low <= buy_sl {
                buy_closed = true;
            } else if candle.high >= buy_tp {
                return TradeResult::Win;
            }
        }

        if !sell_triggered && candle.low <= sell_stop {
            sell_triggered = true;
            let sell_sl = sell_stop + sl_points;
            let sell_tp = sell_stop - tp_points;
            if candle.high >= sell_sl {
                sell_closed = true;
            } else if candle.low <= sell_tp {
                return TradeResult::Win;
            }
        }

        if buy_triggered && sell_triggered && (buy_closed || sell_closed) {
            return TradeResult::Loss;
        }
    }

    if buy_triggered && sell_triggered { return TradeResult::Timeout; }
    TradeResult::Timeout
}
