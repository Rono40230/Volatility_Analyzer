use super::models::*;
use crate::models::{CalendarEvent, Candle};

pub struct EventSimulator;

impl EventSimulator {
    pub fn simulate(
        event: &CalendarEvent,
        candles: &[Candle],
        config: &BacktestConfig,
        mode: StrategyMode,
    ) -> TradeResult {
        let mut logs = Vec::new();
        let event_time = event.event_time.and_utc();
        
        // Trouver la bougie de référence (T0 ou juste avant)
        // On suppose qu'on place les ordres à l'ouverture de la bougie de l'événement
        let t0_candle = candles.iter().find(|c| c.datetime >= event_time);
        
        let reference_price = match t0_candle {
            Some(c) => c.open,
            None => return TradeResult {
                event_date: event_time.to_rfc3339(),
                entry_time: "".to_string(),
                exit_time: "".to_string(),
                duration_minutes: 0,
                pips_net: 0.0,
                outcome: TradeOutcome::NoEntry,
                max_favorable_excursion: 0.0,
                max_adverse_excursion: 0.0,
                logs: vec!["Pas de données pour T0".to_string()],
            }
        };

        logs.push(format!("Prix référence (Open T0): {:.5}", reference_price));

        // Calcul des niveaux d'entrée
        let offset_val = config.offset_pips * config.point_value;
        let spread_val = config.spread_pips * config.point_value;
        
        let buy_trigger = reference_price + offset_val + spread_val;
        let sell_trigger = reference_price - offset_val;

        logs.push(format!("Buy Stop: {:.5}, Sell Stop: {:.5}", buy_trigger, sell_trigger));

        // Simulation boucle par boucle
        let mut position: Option<Position> = None;
        let mut trades_count = 0;
        let mut total_pips_event = 0.0;

        // On commence à scanner à partir de T0
        for candle in candles.iter().filter(|c| c.datetime >= event_time) {
            // Vérifier le Timeout
            let elapsed = (candle.datetime - event_time).num_minutes();
            if elapsed > config.timeout_minutes as i64 {
                if let Some(pos) = position.as_ref() {
                    // Clôture au Timeout
                    let exit_price = if pos.direction == Direction::Long { candle.close } else { candle.close + spread_val }; // Spread en sortie short
                    let pips = (if pos.direction == Direction::Long { exit_price - pos.entry_price } else { pos.entry_price - exit_price }) / config.point_value;
                    total_pips_event += pips;
                    
                    return TradeResult {
                        event_date: event_time.to_rfc3339(),
                        entry_time: pos.entry_time.to_rfc3339(),
                        exit_time: candle.datetime.to_rfc3339(),
                        duration_minutes: elapsed as i32,
                        pips_net: total_pips_event,
                        outcome: if trades_count > 0 { TradeOutcome::RecoveryWin } else { TradeOutcome::Timeout },
                        max_favorable_excursion: pos.mfe / config.point_value,
                        max_adverse_excursion: pos.mae / config.point_value,
                        logs,
                    };
                } else {
                    // Timeout sans entrée
                    return TradeResult {
                        event_date: event_time.to_rfc3339(),
                        entry_time: "".to_string(),
                        exit_time: "".to_string(),
                        duration_minutes: elapsed as i32,
                        pips_net: 0.0,
                        outcome: TradeOutcome::NoEntry,
                        max_favorable_excursion: 0.0,
                        max_adverse_excursion: 0.0,
                        logs,
                    };
                }
            }

            // Si pas de position, vérifier les déclenchements
            if position.is_none() {
                // Check Buy
                if candle.high >= buy_trigger {
                    // On suppose qu'on est exécuté au prix trigger (slippage ignoré pour l'instant)
                    let entry = buy_trigger;
                    let sl_dist = config.stop_loss_pips * config.point_value;
                    let sl = entry - sl_dist;
                    
                    position = Some(Position {
                        direction: Direction::Long,
                        entry_price: entry,
                        entry_time: candle.datetime,
                        stop_loss: sl,
                        highest_price: entry,
                        lowest_price: entry,
                        mfe: 0.0,
                        mae: 0.0,
                    });
                    logs.push(format!("Entrée LONG à {:.5} (High: {:.5})", entry, candle.high));
                }
                // Check Sell (si pas Buy sur la même bougie, ou gestion priorité)
                // Pour simplifier M1: si High touche Buy, on prend Buy.
                // TODO: Gérer le cas où High et Low touchent (bougie volatile) -> souvent Whipsaw
                else if candle.low <= sell_trigger {
                    let entry = sell_trigger;
                    let sl_dist = config.stop_loss_pips * config.point_value;
                    let sl = entry + sl_dist + spread_val; // SL d'un short paie le spread
                    
                    position = Some(Position {
                        direction: Direction::Short,
                        entry_price: entry,
                        entry_time: candle.datetime,
                        stop_loss: sl,
                        highest_price: entry,
                        lowest_price: entry,
                        mfe: 0.0,
                        mae: 0.0,
                    });
                    logs.push(format!("Entrée SHORT à {:.5} (Low: {:.5})", entry, candle.low));
                }
            }

            // Si position active, gérer SL et Trailing
            if let Some(pos) = position.as_mut() {
                // Update MAE/MFE
                if pos.direction == Direction::Long {
                    if candle.high > pos.highest_price { pos.highest_price = candle.high; }
                    if candle.low < pos.lowest_price { pos.lowest_price = candle.low; }
                    pos.mfe = pos.highest_price - pos.entry_price;
                    pos.mae = pos.entry_price - pos.lowest_price;

                    // Check SL
                    if candle.low <= pos.stop_loss {
                        logs.push(format!("SL touché à {:.5}", pos.stop_loss));
                        let pips = (pos.stop_loss - pos.entry_price) / config.point_value;
                        total_pips_event += pips;
                        trades_count += 1;
                        
                        // GESTION RECOVERY (Mode Simultané) - Uniquement si perte
                        if mode == StrategyMode::Simultane && trades_count == 1 && pips < 0.0 {
                            logs.push(format!("Activation Recovery (Trade #2)"));
                            
                            // Reverse position
                            let recovery_sl_pips = config.sl_recovery_pips.unwrap_or(config.stop_loss_pips);
                            let recovery_sl_dist = recovery_sl_pips * config.point_value;
                            
                            // Short entry at SL price
                            let entry = pos.stop_loss;
                            let sl = entry + recovery_sl_dist + spread_val;
                            
                            position = Some(Position {
                                direction: Direction::Short,
                                entry_price: entry,
                                entry_time: candle.datetime,
                                stop_loss: sl,
                                highest_price: entry,
                                lowest_price: entry,
                                mfe: 0.0,
                                mae: 0.0,
                            });
                            logs.push(format!("Entrée RECOVERY SHORT à {:.5}", entry));
                            continue;
                        }

                        let outcome = if pips >= 0.0 {
                            TradeOutcome::TakeProfit // Trailing Stop en profit
                        } else if trades_count > 1 {
                            TradeOutcome::DoubleLoss
                        } else {
                            TradeOutcome::StopLoss
                        };

                        return TradeResult {
                            event_date: event_time.to_rfc3339(),
                            entry_time: pos.entry_time.to_rfc3339(),
                            exit_time: candle.datetime.to_rfc3339(),
                            duration_minutes: (candle.datetime - pos.entry_time).num_minutes() as i32,
                            pips_net: total_pips_event,
                            outcome,
                            max_favorable_excursion: pos.mfe / config.point_value,
                            max_adverse_excursion: pos.mae / config.point_value,
                            logs,
                        };
                    }

                    // Trailing Stop
                    let ts_dist = config.trailing_stop_pips * config.point_value;
                    let new_sl = candle.close - ts_dist;
                    if new_sl > pos.stop_loss {
                        pos.stop_loss = new_sl;
                        // logs.push(format!("Trailing Stop monté à {:.5}", new_sl));
                    }

                } else { // Short
                    if candle.high > pos.highest_price { pos.highest_price = candle.high; }
                    if candle.low < pos.lowest_price { pos.lowest_price = candle.low; }
                    pos.mfe = pos.entry_price - pos.lowest_price;
                    pos.mae = pos.highest_price - pos.entry_price;

                    // Check SL (Ask price = High + Spread)
                    let ask_high = candle.high + spread_val;
                    if ask_high >= pos.stop_loss {
                        logs.push(format!("SL touché à {:.5}", pos.stop_loss));
                        let pips = (pos.entry_price - pos.stop_loss) / config.point_value;
                        total_pips_event += pips;
                        trades_count += 1;

                        // GESTION RECOVERY (Mode Simultané) - Uniquement si perte
                        if mode == StrategyMode::Simultane && trades_count == 1 && pips < 0.0 {
                            logs.push(format!("Activation Recovery (Trade #2)"));
                            
                            // Reverse position
                            let recovery_sl_pips = config.sl_recovery_pips.unwrap_or(config.stop_loss_pips);
                            let recovery_sl_dist = recovery_sl_pips * config.point_value;
                            
                            // Long entry at SL price
                            let entry = pos.stop_loss;
                            let sl = entry - recovery_sl_dist;
                            
                            position = Some(Position {
                                direction: Direction::Long,
                                entry_price: entry,
                                entry_time: candle.datetime,
                                stop_loss: sl,
                                highest_price: entry,
                                lowest_price: entry,
                                mfe: 0.0,
                                mae: 0.0,
                            });
                            logs.push(format!("Entrée RECOVERY LONG à {:.5}", entry));
                            continue;
                        }
                        
                        let outcome = if pips >= 0.0 {
                            TradeOutcome::TakeProfit // Trailing Stop en profit
                        } else if trades_count > 1 {
                            TradeOutcome::DoubleLoss
                        } else {
                            TradeOutcome::StopLoss
                        };

                        return TradeResult {
                            event_date: event_time.to_rfc3339(),
                            entry_time: pos.entry_time.to_rfc3339(),
                            exit_time: candle.datetime.to_rfc3339(),
                            duration_minutes: (candle.datetime - pos.entry_time).num_minutes() as i32,
                            pips_net: total_pips_event,
                            outcome,
                            max_favorable_excursion: pos.mfe / config.point_value,
                            max_adverse_excursion: pos.mae / config.point_value,
                            logs,
                        };
                    }

                    // Trailing Stop
                    let ts_dist = config.trailing_stop_pips * config.point_value;
                    let new_sl = candle.close + ts_dist + spread_val;
                    if new_sl < pos.stop_loss {
                        pos.stop_loss = new_sl;
                    }
                }
            }
        }

        // Fin de boucle sans sortie (ne devrait pas arriver avec le check Timeout, mais au cas où)
        TradeResult {
            event_date: event_time.to_rfc3339(),
            entry_time: "".to_string(),
            exit_time: "".to_string(),
            duration_minutes: 0,
            pips_net: 0.0,
            outcome: TradeOutcome::NoEntry,
            max_favorable_excursion: 0.0,
            max_adverse_excursion: 0.0,
            logs,
        }
    }
}
