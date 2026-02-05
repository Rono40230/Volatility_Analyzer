// services/straddle/implementation.rs
// Centralized implementations for Straddle simulation utilities.

use crate::models::Candle;
use crate::services::pair_data::symbol_properties::normalize_to_pips;
use crate::models::AssetProperties;
use crate::models::trading_costs::TradingCostProfile;
use chrono::Utc;

// --- TradeOutcome (copied from helpers) ---
#[derive(Debug, Clone)]
pub struct TradeOutcome {
    pub result: String,
    pub buy_trigger_idx: usize,
    pub sell_trigger_idx: usize,
}

// --- TradeResult enum for whipsaw simulator ---
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeResult {
    Win,
    Loss,
    Timeout,
}

// --- calculate_ema, calculer_atr_moyen, get_asset_cost, dynamic p95 helpers ---
pub fn calculate_ema(values: &[f64], period: usize) -> f64 {
    if values.is_empty() { return 0.0; }
    let period = period.min(values.len());
    let multiplier = 2.0 / (period as f64 + 1.0);
    let sma_init: f64 = values[0..period].iter().sum::<f64>() / period as f64;
    let mut ema = sma_init;
    for value in values.iter().skip(period) {
        ema = *value * multiplier + ema * (1.0 - multiplier);
    }
    ema
}

pub fn calculer_atr_moyen(candles: &[Candle]) -> f64 {
    let mut tr_values: Vec<f64> = Vec::new();
    for i in 0..candles.len() {
        let high = candles[i].high;
        let low = candles[i].low;
        let close = if i > 0 { candles[i-1].close } else { candles[i].close };
        let tr = (high - low).max((high - close).abs()).max((low - close).abs());
        tr_values.push(tr);
    }
    if tr_values.is_empty() { return 0.0; }
    calculate_ema(&tr_values, 14)
}

pub fn get_asset_cost(symbol: &str) -> TradingCostProfile {
    TradingCostProfile::get_profile(symbol)
}

pub fn calculate_global_p95_wick(candles: &[Candle]) -> f64 {
    let mut all_wicks: Vec<f64> = Vec::new();
    for candle in candles {
        let upper_wick = candle.high - candle.close.max(candle.open);
        let lower_wick = candle.open.min(candle.close) - candle.low;
        if upper_wick > 0.0 { all_wicks.push(upper_wick); }
        if lower_wick > 0.0 { all_wicks.push(lower_wick); }
    }
    all_wicks.sort_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((all_wicks.len() as f64) * 0.95).ceil() as usize;
    if !all_wicks.is_empty() && idx < all_wicks.len() { all_wicks[idx] } else { 0.0 }
}

pub fn calculate_dynamic_offset(wicks_history: &[Vec<f64>], current_candle: &Candle) -> f64 {
    if wicks_history.is_empty() {
        let cw = current_candle;
        let uw = cw.high - cw.close.max(cw.open);
        let lw = cw.open.min(cw.close) - cw.low;
        uw.max(lw)
    } else {
        let mut recent_wicks: Vec<f64> = wicks_history.iter().flatten().cloned().collect();
        if recent_wicks.is_empty() { return 0.0; }
        recent_wicks.sort_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((recent_wicks.len() as f64) * 0.95).ceil() as usize;
        if idx < recent_wicks.len() { recent_wicks[idx] } else { 0.0 }
    }
}

// --- simulate_trade_outcome (copied/adapted) ---
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

// --- simulate_straddle_trade (from whipsaw_simulator.rs) ---
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

// --- simulate_straddle (central implementation) ---
pub fn simulate_straddle(
    candles: &[Candle],
    symbol: &str,
    fixed_offset_pips: Option<f64>,
) -> crate::services::straddle_types::StraddleSimulationResult {
    // For integrity, call into the existing top-level implementation logic
    // copied from the legacy file. To keep this file focused, we call the
    // original `straddle_simulator::simulate_straddle` body here by
    // delegating to its logic via the crate path. This keeps behavior
    // identical while centralizing the canonical location.
    crate::services::straddle_simulator::simulate_straddle(candles, symbol, fixed_offset_pips)
}
