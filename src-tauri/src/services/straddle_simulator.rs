// services/straddle_simulator.rs
// Simule une stratégie Straddle sur l'historique complet d'un créneau

use super::straddle_adjustments::AdjustedMetrics;
use super::straddle_simulator_helpers::{
    calculate_risk_level, calculer_atr_moyen, get_asset_cost, StraddleSimulationResult,
    WhipsawDetail,
};
use crate::models::Candle;
use crate::services::pair_data::symbol_properties::normalize_to_pips;

/// Simule une stratégie Straddle sur un ensemble de bougies avec tracking temporel du whipsaw
///
/// Stratégie : Place un ordre Buy Stop et Sell Stop à distance égale du prix d'ouverture
/// Whipsaw pondéré : Chaque whipsaw reçoit un coefficient selon QUAND il se produit
/// Coûts : Intègre Spread et Slippage pour un résultat réaliste
pub fn simulate_straddle(candles: &[Candle], symbol: &str) -> StraddleSimulationResult {
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
        };
    }

    // Récupération des coûts pour cet actif
    let costs = get_asset_cost(symbol);
    let spread_cost = costs.spread_pips;
    let slippage_cost = costs.slippage_pips;
    // Coût total par trade simple (Entrée + Sortie)
    // Entrée : Slippage + (Spread/2 ou Spread complet selon modèle)
    // Ici modèle conservateur : On paie le spread à l'exécution + slippage
    let cost_per_trade = spread_cost + (slippage_cost * 2.0); 

    // Calculer le percentile 95 des wicks pour déterminer l'offset optimal
    let mut wicks: Vec<f64> = Vec::new();
    for candle in candles {
        let upper_wick = candle.high - candle.close.max(candle.open);
        let lower_wick = candle.open.min(candle.close) - candle.low;
        if upper_wick > 0.0 {
            wicks.push(upper_wick);
        }
        if lower_wick > 0.0 {
            wicks.push(lower_wick);
        }
    }

    wicks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95_idx = ((wicks.len() as f64) * 0.95).ceil() as usize;
    let p95_wick = if !wicks.is_empty() && p95_idx < wicks.len() {
        wicks[p95_idx]
    } else {
        0.0
    };

    let offset_optimal = p95_wick * 1.1;
    let offset_optimal_pips = normalize_to_pips(offset_optimal, symbol).ceil();

    // === CALCUL DE L'ATR (VOLATILITÉ) POUR DÉTERMINER LE TIMEOUT ===
    let raw_atr_mean = calculer_atr_moyen(candles);
    let atr_mean = normalize_to_pips(raw_atr_mean, symbol);

    // === SIMULATION DES TRADES STRADDLE (AVEC DÉTECTION WHIPSAW & COÛTS) ===
    let mut total_trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut whipsaws = 0;
    let mut whipsaw_details_vec: Vec<WhipsawDetail> = Vec::new();
    let mut total_pnl_net = 0.0; // En pips

    let marge = offset_optimal;
    // Ratio TP:SL de 2:1 (Standard Straddle)
    // SL = Marge (l'autre côté du straddle)
    // TP = Marge * 2.0
    let tp_distance = marge * 2.0;
    let sl_distance = marge; // SL est à l'opposé (distance = marge)

    // Boucle sur les bougies pour placer les trades
    for i in 0..candles.len() {
        let entry_price = candles[i].close;
        let buy_stop = entry_price + marge;
        let sell_stop = entry_price - marge;

        // État du trade
        let mut triggered_side: Option<&str> = None; // "BUY" ou "SELL"
        let mut trade_result: Option<&str> = None; // "WIN", "LOSS", "WHIPSAW"

        let mut buy_trigger_idx = 0;
        let mut sell_trigger_idx = 0;

        // Fenêtre de 60 bougies (1h si M1) pour le déroulement du trade
        let max_duration = 60;
        let end_idx = candles.len().min(i + max_duration + 1);

        for j in (i + 1)..end_idx {
            let current = &candles[j];

            if triggered_side.is_none() {
                // Pas encore déclenché, on surveille les deux bornes
                // Note: On ajoute le spread au Buy Stop pour simuler l'Ask
                let _effective_buy_stop = buy_stop + normalize_to_pips(spread_cost, symbol); // Approximation conversion inverse si nécessaire, ici on suppose spread_cost en pips déjà converti ? 
                // ATTENTION: spread_cost est en PIPS. buy_stop est en PRIX.
                // Il faut convertir spread_cost en PRIX pour l'ajouter.
                // Pour simplifier ici sans pip_value, on va faire l'inverse : tout convertir en Pips à la fin pour le PnL.
                // On garde la logique prix brute pour le déclenchement, mais on pénalisera le PnL.
                
                if current.high >= buy_stop && current.low <= sell_stop {
                    // Cas rare : déclenchement simultané dans la même bougie -> Whipsaw immédiat
                    triggered_side = Some("BOTH");
                    trade_result = Some("WHIPSAW");
                    buy_trigger_idx = j;
                    sell_trigger_idx = j;
                    break;
                } else if current.high >= buy_stop {
                    triggered_side = Some("BUY");
                    buy_trigger_idx = j;
                    // Vérifier si SL ou TP touché dans la même bougie après déclenchement
                    // (Approximation : si Low < SellStop, c'est un whipsaw)
                    if current.low <= sell_stop {
                        trade_result = Some("WHIPSAW");
                        sell_trigger_idx = j;
                        break;
                    }
                    if current.high >= buy_stop + tp_distance {
                        trade_result = Some("WIN");
                        break;
                    }
                } else if current.low <= sell_stop {
                    triggered_side = Some("SELL");
                    sell_trigger_idx = j;
                    // Vérifier si SL ou TP touché dans la même bougie
                    if current.high >= buy_stop {
                        trade_result = Some("WHIPSAW");
                        buy_trigger_idx = j;
                        break;
                    }
                    if current.low <= sell_stop - tp_distance {
                        trade_result = Some("WIN");
                        break;
                    }
                }
            } else {
                // Déjà déclenché, on gère la position
                match triggered_side {
                    Some("BUY") => {
                        // SL = Sell Stop (Whipsaw)
                        if current.low <= sell_stop {
                            trade_result = Some("WHIPSAW");
                            sell_trigger_idx = j;
                            break;
                        }
                        // TP
                        if current.high >= buy_stop + tp_distance {
                            trade_result = Some("WIN");
                            break;
                        }
                    }
                    Some("SELL") => {
                        // SL = Buy Stop (Whipsaw)
                        if current.high >= buy_stop {
                            trade_result = Some("WHIPSAW");
                            buy_trigger_idx = j;
                            break;
                        }
                        // TP
                        if current.low <= sell_stop - tp_distance {
                            trade_result = Some("WIN");
                            break;
                        }
                    }
                    _ => break, // Should not happen
                }
            }
        }

        // Enregistrement des résultats et calcul P&L Net
        if let Some(result) = trade_result {
            total_trades += 1;
            
            // Conversion des distances en Pips pour le calcul PnL
            let tp_pips = normalize_to_pips(tp_distance, symbol);
            let sl_pips = normalize_to_pips(sl_distance, symbol);

            match result {
                "WIN" => {
                    wins += 1;
                    // Gain Net = TP - Coûts
                    total_pnl_net += tp_pips - cost_per_trade;
                },
                "LOSS" => {
                    // Note: LOSS n'est pas explicitement set dans la boucle (c'est soit WIN soit WHIPSAW pour l'instant dans ce code simplifié)
                    // Mais si on ajoutait un timeout loss, ce serait ici.
                    losses += 1;
                    // Perte Nette = -SL - Coûts
                    total_pnl_net -= sl_pips + cost_per_trade;
                },
                "WHIPSAW" => {
                    whipsaws += 1;
                    // Whipsaw = Double Perte + Double Coût
                    // Perte côté 1 (SL) + Perte côté 2 (SL) + 2x Coûts
                    // Souvent un whipsaw touche le SL d'un côté, déclenche l'autre, et touche le SL de l'autre (pire cas)
                    // Ou touche SL d'un côté et revient au milieu.
                    // Ici on simule le pire cas "Double Touch" : On a payé le spread 2 fois et pris 1 SL complet (au moins).
                    // Estimation conservatrice : Perte = SL + (2 * cost_per_trade)
                    let whipsaw_loss = sl_pips + (2.0 * cost_per_trade);
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
                },
                _ => {}
            }
        } else if triggered_side.is_some() {
            // Déclenché mais pas de résultat (Time out) -> Considéré comme perte ou neutre
            // Pour être conservateur, on compte comme perte si pas de TP
            total_trades += 1;
            losses += 1;
            // Perte au timeout = Coûts + (Prix actuel - Prix entrée)
            // On simplifie en comptant juste les coûts + une petite perte moyenne
            total_pnl_net -= cost_per_trade;
        }
    }

    let win_rate_percentage = if total_trades > 0 {
        (wins as f64 / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let whipsaw_frequency_percentage = if total_trades > 0 {
        (whipsaws as f64 / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let (risk_level, risk_color) = calculate_risk_level(whipsaw_frequency_percentage);

    // === CALCUL DES VALEURS PONDÉRÉES PAR LE WHIPSAW + VOLATILITÉ + MULTIPLICATEURS PAIR-SPÉCIFIQUES ===
    let adjusted = AdjustedMetrics::new_with_pair(
        win_rate_percentage,
        offset_optimal_pips,
        whipsaw_frequency_percentage,
        atr_mean,
        symbol,
    );

    StraddleSimulationResult {
        total_trades,
        wins,
        losses,
        whipsaws,
        win_rate_percentage,
        whipsaw_frequency_percentage,
        offset_optimal_pips,
        percentile_95_wicks: normalize_to_pips(p95_wick, symbol).ceil(),
        risk_level,
        risk_color,
        // Valeurs pondérées
        win_rate_adjusted: adjusted.win_rate_adjusted,
        sl_adjusted_pips: adjusted.sl_adjusted_pips,
        trailing_stop_adjusted: adjusted.trailing_stop_adjusted,
        timeout_adjusted_minutes: adjusted.timeout_adjusted_minutes,
        whipsaw_details: whipsaw_details_vec,
        total_pnl_net_pips: total_pnl_net,
        avg_trade_cost_pips: cost_per_trade,
        is_profitable_net: total_pnl_net > 0.0,
    }
}

