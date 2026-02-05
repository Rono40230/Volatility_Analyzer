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
    use crate::services::straddle_adjustments::AdjustedMetrics;
    use crate::services::straddle_types::{StraddleSimulationResult, WhipsawDetail};
    use crate::models::Candle;
    use crate::services::pair_data::symbol_properties::normalize_to_pips;
    use crate::models::AssetProperties;

    if candles.is_empty() {
        return StraddleSimulationResult {
            total_trades: 0,
            wins: 0,
            losses: 0,
            whipsaws: 0,
            win_rate_percentage: 0.0,
            whipsaw_frequency_percentage: 0.0,
            offset_optimal_pips: 0.0,
            percentile_95_wicks: 0.0,
            risk_level: "N/A".to_string(),
            risk_color: "#6b7280".to_string(),
            win_rate_adjusted: 0.0,
            sl_adjusted_pips: 0.0,
            trailing_stop_adjusted: 0.0,
            timeout_adjusted_minutes: 0,
            whipsaw_details: Vec::new(),
            total_pnl_net_pips: 0.0,
            avg_trade_cost_pips: 0.0,
            is_profitable_net: false,
            confidence_score: 0.0,
            sample_size_warning: true,
        };
    }

    let costs = get_asset_cost(symbol);
    let spread_cost = costs.spread_avg;
    let slippage_cost = costs.slippage;
    let cost_per_trade = spread_cost + (slippage_cost * 2.0);

    let global_p95_wick = calculate_global_p95_wick(candles);

    let raw_atr_mean = calculer_atr_moyen(candles);
    let atr_mean = normalize_to_pips(raw_atr_mean, symbol);

    let timeout_adjusted_minutes = AdjustedMetrics::calculer_timeout_from_atr(atr_mean).max(1);
    let max_duration = timeout_adjusted_minutes as usize;

    let mut total_trades = 0usize;
    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut whipsaws = 0usize;
    let mut whipsaw_details_vec: Vec<WhipsawDetail> = Vec::new();
    let mut total_pnl_net = 0.0;
    let mut sum_offsets_used = 0.0;

    let asset_props = AssetProperties::from_symbol(symbol);

    let window_size = 5usize;
    let mut wicks_history: Vec<Vec<f64>> = Vec::new();

    for i in 0..candles.len() {
        let offset_optimal_pips = match fixed_offset_pips {
            Some(value) => value.max(0.0),
            None => {
                let current_p95_wick = calculate_dynamic_offset(&wicks_history, &candles[i]);
                normalize_to_pips(current_p95_wick * 1.1, symbol).ceil()
            }
        };

        sum_offsets_used += offset_optimal_pips;

        let marge = asset_props.denormalize(offset_optimal_pips);
        let tp_distance = marge * 2.0;
        let sl_distance = marge;

        let entry_price = candles[i].close;
        let buy_stop = entry_price + marge;
        let sell_stop = entry_price - marge;

        if fixed_offset_pips.is_none() {
            let cw = &candles[i];
            let mut current_wicks = Vec::new();
            let uw = cw.high - cw.close.max(cw.open);
            let lw = cw.open.min(cw.close) - cw.low;
            if uw > 0.0 { current_wicks.push(uw); }
            if lw > 0.0 { current_wicks.push(lw); }

            wicks_history.push(current_wicks);
            if wicks_history.len() > window_size { wicks_history.remove(0); }
        }

        let outcome = simulate_trade_outcome(
            candles,
            i,
            buy_stop,
            sell_stop,
            tp_distance,
            sl_distance,
            max_duration,
        );

        if let Some(res) = outcome {
            let buy_trigger_idx = res.buy_trigger_idx;
            let sell_trigger_idx = res.sell_trigger_idx;

            let tp_pips = normalize_to_pips(tp_distance, symbol);
            let sl_pips = normalize_to_pips(sl_distance, symbol);

            match res.result.as_str() {
                "WIN" => {
                    total_trades += 1;
                    wins += 1;
                    total_pnl_net += tp_pips - cost_per_trade;
                }
                "LOSS" => {
                    total_trades += 1;
                    losses += 1;
                    total_pnl_net -= sl_pips + cost_per_trade;
                }
                "WHIPSAW" => {
                    total_trades += 1;
                    whipsaws += 1;
                    let whipsaw_loss = (2.0 * sl_pips) + (2.0 * cost_per_trade);
                    total_pnl_net -= whipsaw_loss;

                    whipsaw_details_vec.push(WhipsawDetail {
                        entry_index: i,
                        entry_price,
                        buy_stop,
                        sell_stop,
                        buy_trigger_index: buy_trigger_idx,
                        sell_trigger_index: sell_trigger_idx,
                        net_loss_pips: whipsaw_loss,
                    });
                }
                "TIMEOUT" => {
                    total_trades += 1;
                    losses += 1;
                    let exit_idx = (i + max_duration).min(candles.len().saturating_sub(1));
                    let exit_price = candles.get(exit_idx).map(|c| c.close).unwrap_or(entry_price);
                    let raw_pips = if buy_trigger_idx > 0 {
                        normalize_to_pips(exit_price - buy_stop, symbol)
                    } else if sell_trigger_idx > 0 {
                        normalize_to_pips(sell_stop - exit_price, symbol)
                    } else { 0.0 };
                    total_pnl_net += raw_pips - cost_per_trade;
                }
                _ => {}
            }
        }
    }

    let win_rate_percentage = if total_trades > 0 { (wins as f64 / total_trades as f64) * 100.0 } else { 0.0 };
    let whipsaw_frequency_percentage = if total_trades > 0 { (whipsaws as f64 / total_trades as f64) * 100.0 } else { 0.0 };

    let (risk_level, risk_color) = crate::services::straddle_simulator_helpers::calculate_risk_level(whipsaw_frequency_percentage);

    let avg_offset_used = if total_trades > 0 { sum_offsets_used / total_trades as f64 } else { 0.0 };

    let adjusted = AdjustedMetrics::new_with_pair(
        win_rate_percentage,
        avg_offset_used,
        whipsaw_frequency_percentage,
        atr_mean,
        symbol,
    );

    let sample_size_warning = total_trades < 5;
    let sample_score = (total_trades as f64 / 10.0).min(1.0) * 100.0;
    let regularity_score = (100.0 - whipsaw_frequency_percentage).max(0.0);
    let confidence_score = (sample_score * 0.7) + (regularity_score * 0.3);

    StraddleSimulationResult {
        total_trades,
        wins,
        losses,
        whipsaws,
        win_rate_percentage,
        whipsaw_frequency_percentage,
        offset_optimal_pips: avg_offset_used,
        percentile_95_wicks: normalize_to_pips(global_p95_wick, symbol).ceil(),
        risk_level,
        risk_color,
        win_rate_adjusted: adjusted.win_rate_adjusted,
        sl_adjusted_pips: adjusted.sl_adjusted_pips,
        trailing_stop_adjusted: adjusted.trailing_stop_adjusted,
        timeout_adjusted_minutes: adjusted.timeout_adjusted_minutes,
        whipsaw_details: whipsaw_details_vec,
        total_pnl_net_pips: total_pnl_net,
        avg_trade_cost_pips: cost_per_trade,
        is_profitable_net: total_pnl_net > 0.0,
        confidence_score,
        sample_size_warning,
    }
}
