use super::models::*;
use super::position_tracker::{
    build_atr_series, build_trade_result, update_long_excursion, update_long_trailing,
    update_short_excursion, update_short_trailing, TradeResultParams,
};
use crate::models::{CalendarEvent, Candle};

pub struct EventSimulator;

impl EventSimulator {
    pub fn simulate(event: &CalendarEvent, candles: &[Candle], config: &BacktestConfig) -> TradeResult {
        let mut logs = Vec::new();
        let event_time = event.event_time.and_utc();

        // Bougie de référence (T0 ou juste avant)
        let t0_candle = match candles.iter().find(|c| c.datetime >= event_time) {
            Some(c) => c,
            None => {
                return TradeResult {
                    event_date: event_time.to_rfc3339(),
                    entry_time: "".to_string(),
                    exit_time: "".to_string(),
                    duration_minutes: 0,
                    pips_net: 0.0,
                    outcome: TradeOutcome::NoEntry,
                    max_favorable_excursion: 0.0,
                    max_adverse_excursion: 0.0,
                    logs: vec!["Pas de données pour T0".to_string()],
                };
            }
        };

        let reference_price = t0_candle.open;

        logs.push(format!("Prix référence (Open T0): {:.5}", reference_price));

        let entry_price = reference_price;
        let sl_dist = config.stop_loss_pips * config.point_value;
        let spread = config.spread_pips * config.point_value;
        let slippage = config.slippage_pips * config.point_value;
        let tp_dist = sl_dist * config.tp_rr.max(0.0);
        let atr_period = config.atr_period.max(1) as usize;
        let atr_values = build_atr_series(candles, atr_period);

        let long_entry = entry_price + spread + slippage;
        let short_entry = entry_price - slippage;
        let long_sl = long_entry - sl_dist;
        let short_sl = short_entry + sl_dist + spread;
        let long_tp = long_entry + tp_dist;
        let short_tp = short_entry - tp_dist;
        let be_long = short_sl;
        let be_short = long_sl;

        let mut long_pos: Option<Position> = Some(Position {
            direction: Direction::Long,
            entry_price: long_entry,
            entry_time: t0_candle.datetime,
            stop_loss: long_sl,
            highest_price: long_entry,
            lowest_price: long_entry,
            mfe: 0.0,
            mae: 0.0,
        });
        let mut short_pos: Option<Position> = Some(Position {
            direction: Direction::Short,
            entry_price: short_entry,
            entry_time: t0_candle.datetime,
            stop_loss: short_sl,
            highest_price: short_entry,
            lowest_price: short_entry,
            mfe: 0.0,
            mae: 0.0,
        });
        let mut long_closed = false;
        let mut short_closed = false;
        let mut long_pips = 0.0;
        let mut short_pips = 0.0;
        let mut exit_time_final = event_time;
        let entry_time_final: Option<chrono::DateTime<chrono::Utc>> = Some(t0_candle.datetime);
        let mut timeout_triggered = false;
        let mut tp_actually_hit = false;
        let mut long_trailing = false;
        let mut short_trailing = false;
        let mut last_trail_update_long: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut last_trail_update_short: Option<chrono::DateTime<chrono::Utc>> = None;
        let refresh_seconds = config.trailing_refresh_seconds.max(0) as i64;

        logs.push(format!("Entrée Long @ {:.5}", long_entry));
        logs.push(format!("Entrée Short @ {:.5}", short_entry));

        for (idx, candle) in candles.iter().enumerate() {
            if candle.datetime < event_time {
                continue;
            }
            let elapsed = (candle.datetime - event_time).num_minutes();

            if elapsed > config.timeout_minutes as i64 {
                if let Some(long) = &long_pos {
                    if !long_closed {
                        let exit = candle.close - slippage;
                        long_pips = (exit - long.entry_price) / config.point_value;
                        logs.push(format!(
                            "⏰ Timeout Long: Close @ {:.5}, P/L: {:.1}",
                            exit, long_pips
                        ));
                        exit_time_final = candle.datetime;
                        timeout_triggered = true;
                    }
                }
                if let Some(short) = &short_pos {
                    if !short_closed {
                        let exit = candle.close + spread + slippage; // Buy to cover avec slippage
                        short_pips = (short.entry_price - exit) / config.point_value;
                        logs.push(format!(
                            "⏰ Timeout Short: Close @ {:.5}, P/L: {:.1}",
                            exit, short_pips
                        ));
                        exit_time_final = candle.datetime;
                        timeout_triggered = true;
                    }
                }
                break;
            }

            if let Some(long) = &mut long_pos {
                if !long_closed {
                    update_long_excursion(long, candle);
                }
            }

            if let Some(short) = &mut short_pos {
                if !short_closed {
                    update_short_excursion(short, candle);
                }
            }

            // === TP CHECK AVANT trailing (le trailing ne peut pas court-circuiter le TP) ===
            if let Some(long) = &mut long_pos {
                if !long_closed && candle.high >= long_tp {
                    if candle.low <= long.stop_loss {
                        // SL + TP même bougie → SL prioritaire (conservateur)
                        let exit = long.stop_loss - slippage;
                        long_pips = (exit - long.entry_price) / config.point_value;
                        long_closed = true;
                        logs.push(format!(
                            "💥 SL Long (même bougie que TP): Close @ {:.5}, P/L: {:.1}",
                            long.stop_loss, long_pips
                        ));
                        exit_time_final = candle.datetime;
                    } else {
                        let exit = long_tp - slippage;
                        long_pips = (exit - long.entry_price) / config.point_value;
                        long_closed = true;
                        tp_actually_hit = true;
                        logs.push(format!(
                            "✅ TP Long: Close @ {:.5}, P/L: {:.1}",
                            exit, long_pips
                        ));
                        exit_time_final = candle.datetime;
                    }
                }
            }

            if let Some(short) = &mut short_pos {
                if !short_closed && candle.low <= short_tp {
                    let ask_high = candle.high + spread;
                    if ask_high >= short.stop_loss {
                        // SL + TP même bougie → SL prioritaire (conservateur)
                        let exit = short.stop_loss + slippage;
                        short_pips = (short.entry_price - exit) / config.point_value;
                        short_closed = true;
                        logs.push(format!(
                            "💥 SL Short (même bougie que TP): Close @ {:.5}, P/L: {:.1}",
                            short.stop_loss, short_pips
                        ));
                        exit_time_final = candle.datetime;
                    } else {
                        let exit = short_tp + slippage;
                        short_pips = (short.entry_price - exit) / config.point_value;
                        short_closed = true;
                        tp_actually_hit = true;
                        logs.push(format!(
                            "✅ TP Short: Close @ {:.5}, P/L: {:.1}",
                            exit, short_pips
                        ));
                        exit_time_final = candle.datetime;
                    }
                }
            }

            // === BE check + trailing (seulement si TP pas encore atteint) ===
            if !long_trailing && !long_closed && candle.high >= be_long {
                long_trailing = true;
                if let Some(long) = &mut long_pos {
                    long.stop_loss = long.stop_loss.max(be_long);
                }
                logs.push(format!("🔒 BE Long activé @ {:.5}", be_long));
            }

            if !short_trailing && !short_closed && candle.low <= be_short {
                short_trailing = true;
                if let Some(short) = &mut short_pos {
                    short.stop_loss = short.stop_loss.min(be_short);
                }
                logs.push(format!("🔒 BE Short activé @ {:.5}", be_short));
            }

            let atr = atr_values.get(idx).copied().unwrap_or(0.0);
            let trail_dist = atr * config.trailing_atr_coef.max(0.0);
            if trail_dist > 0.0 {
                if long_trailing && !long_closed {
                    if let Some(long) = &mut long_pos {
                        update_long_trailing(long, candle, trail_dist, &mut last_trail_update_long, refresh_seconds);
                    }
                }
                if short_trailing && !short_closed {
                    if let Some(short) = &mut short_pos {
                        update_short_trailing(short, candle, trail_dist, &mut last_trail_update_short, refresh_seconds);
                    }
                }
            }

            // === SL check APRÈS trailing (TP déjà géré au-dessus) ===
            if let Some(long) = &mut long_pos {
                if !long_closed && candle.low <= long.stop_loss {
                    let exit = long.stop_loss - slippage;
                    long_pips = (exit - long.entry_price) / config.point_value;
                    long_closed = true;
                    if long_trailing {
                        logs.push(format!(
                            "🧭 TS Long: Close @ {:.5}, P/L: {:.1}",
                            long.stop_loss, long_pips
                        ));
                    } else {
                        logs.push(format!(
                            "💥 SL Long: Close @ {:.5}, P/L: {:.1}",
                            long.stop_loss, long_pips
                        ));
                    }
                    exit_time_final = candle.datetime;
                }
            }

            if let Some(short) = &mut short_pos {
                if !short_closed {
                    let ask_high = candle.high + spread;
                    if ask_high >= short.stop_loss {
                        let exit = short.stop_loss + slippage;
                        short_pips = (short.entry_price - exit) / config.point_value;
                        short_closed = true;
                        if short_trailing {
                            logs.push(format!(
                                "🧭 TS Short: Close @ {:.5}, P/L: {:.1}",
                                short.stop_loss, short_pips
                            ));
                        } else {
                            logs.push(format!(
                                "💥 SL Short: Close @ {:.5}, P/L: {:.1}",
                                short.stop_loss, short_pips
                            ));
                        }
                        exit_time_final = candle.datetime;
                    }
                }
            }

            let _ = (long_closed, short_closed);
            if long_closed && short_closed { break; }
        }

        build_trade_result(TradeResultParams {
            event_time,
            entry_time_final,
            exit_time_final,
            long_pos,
            short_pos,
            long_pips,
            short_pips,
            timeout_triggered,
            tp_hit: tp_actually_hit,
            point_value: config.point_value,
            logs,
        })
    }
}
