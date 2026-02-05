use super::models::*;
use crate::models::{CalendarEvent, Candle};

pub struct EventSimulator;

impl EventSimulator {
    pub fn simulate(event: &CalendarEvent, candles: &[Candle], config: &BacktestConfig) -> TradeResult {
        let mut logs = Vec::new();
        let event_time = event.event_time.and_utc();

        // Bougie de référence (T0 ou juste avant)
        let t0_candle = candles.iter().find(|c| c.datetime >= event_time);

        let reference_price = match t0_candle {
            Some(c) => c.open,
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

        logs.push(format!("Prix référence (Open T0): {:.5}", reference_price));

        let entry_price = reference_price;
        let sl_dist = config.stop_loss_pips * config.point_value;
        let spread = config.spread_pips * config.point_value;
        let slippage = config.slippage_pips * config.point_value;
        let offset_dist = config.offset_pips * config.point_value;
        let tp_dist = offset_dist * 2.0;
        let buy_stop = entry_price + offset_dist;
        let sell_stop = entry_price - offset_dist;

        let mut long_pos: Option<Position> = None;
        let mut short_pos: Option<Position> = None;
        let mut long_closed = false;
        let mut short_closed = false;
        let mut long_pips = 0.0;
        let mut short_pips = 0.0;
        let mut exit_time_final = event_time;
        let mut entry_time_final: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut timeout_triggered = false;

        for candle in candles.iter().filter(|c| c.datetime >= event_time) {
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

            if long_pos.is_none() && candle.high >= buy_stop {
                let entry = buy_stop + spread + slippage;
                long_pos = Some(Position {
                    direction: Direction::Long,
                    entry_price: entry,
                    entry_time: candle.datetime,
                    stop_loss: entry - sl_dist,
                    highest_price: entry,
                    lowest_price: entry,
                    mfe: 0.0,
                    mae: 0.0,
                });
                if entry_time_final.is_none() {
                    entry_time_final = Some(candle.datetime);
                }
                logs.push(format!("✅ Long déclenché @ {:.5}", entry));
            }

            if short_pos.is_none() && candle.low <= sell_stop {
                let entry = sell_stop - slippage;
                short_pos = Some(Position {
                    direction: Direction::Short,
                    entry_price: entry,
                    entry_time: candle.datetime,
                    stop_loss: entry + sl_dist + spread,
                    highest_price: entry,
                    lowest_price: entry,
                    mfe: 0.0,
                    mae: 0.0,
                });
                if entry_time_final.is_none() {
                    entry_time_final = Some(candle.datetime);
                }
                logs.push(format!("✅ Short déclenché @ {:.5}", entry));
            }

            if let Some(long) = &mut long_pos {
                if !long_closed {
                    if candle.high > long.highest_price {
                        long.highest_price = candle.high;
                    }
                    if candle.low < long.lowest_price {
                        long.lowest_price = candle.low;
                    }
                    long.mfe = long.highest_price - long.entry_price;
                    long.mae = long.entry_price - long.lowest_price;

                    if candle.high >= long.entry_price + tp_dist {
                        let exit = long.entry_price + tp_dist - slippage;
                        long_pips = (exit - long.entry_price) / config.point_value;
                        long_closed = true;
                        logs.push(format!(
                            "✅ TP Long: Close @ {:.5}, P/L: {:.1}",
                            exit, long_pips
                        ));
                        exit_time_final = candle.datetime;
                    } else if candle.low <= long.stop_loss {
                        let exit = long.stop_loss - slippage;
                        long_pips = (exit - long.entry_price) / config.point_value;
                        long_closed = true;
                        logs.push(format!(
                            "💥 SL Long: Close @ {:.5}, P/L: {:.1}",
                            long.stop_loss, long_pips
                        ));
                        exit_time_final = candle.datetime;
                    }
                }
            }

            if let Some(short) = &mut short_pos {
                if !short_closed {
                    if candle.high > short.highest_price {
                        short.highest_price = candle.high;
                    }
                    if candle.low < short.lowest_price {
                        short.lowest_price = candle.low;
                    }
                    short.mfe = short.entry_price - short.lowest_price;
                    short.mae = short.highest_price - short.entry_price;

                    if candle.low <= short.entry_price - tp_dist {
                        let exit = short.entry_price - tp_dist + slippage;
                        short_pips = (short.entry_price - exit) / config.point_value;
                        short_closed = true;
                        logs.push(format!(
                            "✅ TP Short: Close @ {:.5}, P/L: {:.1}",
                            exit, short_pips
                        ));
                        exit_time_final = candle.datetime;
                    } else {
                        let ask_high = candle.high + spread;
                        if ask_high >= short.stop_loss {
                        let exit = short.stop_loss + slippage;
                        short_pips = (short.entry_price - exit) / config.point_value;
                        short_closed = true;
                        logs.push(format!(
                            "💥 SL Short: Close @ {:.5}, P/L: {:.1}",
                            short.stop_loss, short_pips
                        ));
                        exit_time_final = candle.datetime;
                        }
                    }
                }
            }

            let _ = (long_closed, short_closed);
            if long_closed && short_closed { break; }
        }

        let total_pips = long_pips + short_pips;
        let outcome = if long_pos.is_none() && short_pos.is_none() {
            TradeOutcome::NoEntry
        } else if timeout_triggered {
            TradeOutcome::Timeout
        } else if total_pips > 0.0 {
            TradeOutcome::TakeProfit
        } else {
            TradeOutcome::StopLoss
        };

        if long_pos.is_none() && short_pos.is_none() {
            logs.push("⚠️ Aucun déclenchement avant timeout".to_string());
        }

        let entry_time_output = entry_time_final
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        let exit_time_output = if long_pos.is_none() && short_pos.is_none() {
            "".to_string()
        } else {
            exit_time_final.to_rfc3339()
        };

        let mfe_total = long_pos.as_ref().map(|p| p.mfe).unwrap_or(0.0)
            + short_pos.as_ref().map(|p| p.mfe).unwrap_or(0.0);
        let mae_total = long_pos.as_ref().map(|p| p.mae).unwrap_or(0.0)
            + short_pos.as_ref().map(|p| p.mae).unwrap_or(0.0);

        TradeResult {
            event_date: event_time.to_rfc3339(),
            entry_time: entry_time_output,
            exit_time: exit_time_output,
            duration_minutes: if entry_time_final.is_some() {
                (exit_time_final - event_time).num_minutes() as i32
            } else {
                0
            },
            pips_net: total_pips,
            outcome,
            max_favorable_excursion: mfe_total / config.point_value,
            max_adverse_excursion: mae_total / config.point_value,
            logs,
        }
    }
}
