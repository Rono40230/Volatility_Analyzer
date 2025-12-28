// services/straddle_simulator.rs
// Simule une stratégie Straddle sur l'historique complet d'un créneau

use super::straddle_adjustments::AdjustedMetrics;
use super::straddle_simulator_helpers::{
    calculate_risk_level, calculer_atr_moyen, get_asset_cost, simulate_trade_outcome,
};
use super::straddle_types::{StraddleSimulationResult, WhipsawDetail};
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
            confidence_score: 0.0,
            sample_size_warning: true,
        };
    }

    // Récupération des coûts pour cet actif
    let costs = get_asset_cost(symbol);
    let spread_cost = costs.spread_avg;
    let slippage_cost = costs.slippage;
    // Coût total par trade simple (Entrée + Sortie)
    // Entrée : Slippage + (Spread/2 ou Spread complet selon modèle)
    // Ici modèle conservateur : On paie le spread à l'exécution + slippage
    let cost_per_trade = spread_cost + (slippage_cost * 2.0); 

    // Calculer le percentile 95 GLOBAL des wicks (pour info statistique uniquement)
    let global_p95_wick = super::straddle_simulator_helpers::calculate_global_p95_wick(candles);

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
    let mut sum_offsets_used = 0.0; // Pour calculer la moyenne

    // Historique glissant des wicks pour le calcul dynamique de l'offset (Look-ahead bias removal)
    // On stocke les wicks de chaque bougie passée.
    // Window size = 5 (comme demandé dans task.md)
    let window_size = 5;
    let mut wicks_history: Vec<Vec<f64>> = Vec::new();

    // Boucle sur les bougies pour placer les trades
    for i in 0..candles.len() {
        // 1. Calculer l'offset dynamique basé sur l'historique (P95 des 5 dernières bougies)
        let current_p95_wick = super::straddle_simulator_helpers::calculate_dynamic_offset(&wicks_history, &candles[i]);

        let offset_optimal = current_p95_wick * 1.1;
        let offset_optimal_pips = normalize_to_pips(offset_optimal, symbol).ceil();
        sum_offsets_used += offset_optimal_pips;

        let marge = offset_optimal;
        // Ratio TP:SL de 2:1 (Standard Straddle)
        let tp_distance = marge * 2.0;
        let sl_distance = marge;

        let entry_price = candles[i].close;
        let buy_stop = entry_price + marge;
        let sell_stop = entry_price - marge;

        // Mise à jour de l'historique pour le PROCHAIN tour
        let cw = &candles[i];
        let mut current_wicks = Vec::new();
        let uw = cw.high - cw.close.max(cw.open);
        let lw = cw.open.min(cw.close) - cw.low;
        if uw > 0.0 { current_wicks.push(uw); }
        if lw > 0.0 { current_wicks.push(lw); }
        
        wicks_history.push(current_wicks);
        if wicks_history.len() > window_size {
            wicks_history.remove(0);
        }

        // Fenêtre de 60 bougies (1h si M1) pour le déroulement du trade
        let max_duration = 60;
        let outcome = simulate_trade_outcome(candles, i, buy_stop, sell_stop, tp_distance, max_duration);

        // Enregistrement des résultats et calcul P&L Net
        if let Some(res) = outcome {
            let buy_trigger_idx = res.buy_trigger_idx;
            let sell_trigger_idx = res.sell_trigger_idx;
            
            // Conversion des distances en Pips pour le calcul PnL
            let tp_pips = normalize_to_pips(tp_distance, symbol);
            let sl_pips = normalize_to_pips(sl_distance, symbol);

            match res.result.as_str() {
                "WIN" => {
                    total_trades += 1;
                    wins += 1;
                    // Gain Net = TP - Coûts
                    total_pnl_net += tp_pips - cost_per_trade;
                },
                "LOSS" => {
                    total_trades += 1;
                    losses += 1;
                    // Perte Nette = -SL - Coûts
                    total_pnl_net -= sl_pips + cost_per_trade;
                },
                "WHIPSAW" => {
                    total_trades += 1;
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
                "TIMEOUT" => {
                    // Déclenché mais pas de résultat (Time out) -> Considéré comme perte ou neutre
                    // Pour être conservateur, on compte comme perte si pas de TP
                    total_trades += 1;
                    losses += 1;
                    // Perte au timeout = Coûts + (Prix actuel - Prix entrée)
                    // On simplifie en comptant juste les coûts + une petite perte moyenne
                    total_pnl_net -= cost_per_trade;
                }
                _ => {}
            }
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

    // Moyenne de l'offset utilisé
    let avg_offset_used = if total_trades > 0 {
        sum_offsets_used / total_trades as f64
    } else {
        0.0
    };

    // === CALCUL DES VALEURS PONDÉRÉES PAR LE WHIPSAW + VOLATILITÉ + MULTIPLICATEURS PAIR-SPÉCIFIQUES ===
    let adjusted = AdjustedMetrics::new_with_pair(
        win_rate_percentage,
        avg_offset_used,
        whipsaw_frequency_percentage,
        atr_mean,
        symbol,
    );

    // === CALCUL DU SCORE DE CONFIANCE ===
    let sample_size_warning = total_trades < 5;
    
    // 1. Score Taille Échantillon (70%)
    // 10 trades = 100% du score échantillon
    let sample_score = (total_trades as f64 / 10.0).min(1.0) * 100.0;
    
    // 2. Score Régularité (30%)
    // Basé sur l'absence de whipsaws (plus c'est propre, plus on a confiance)
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
        // Valeurs pondérées
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Candle;
    use chrono::{TimeZone, Utc};

    fn create_candle(price: f64) -> Candle {
        Candle {
            id: Some(0),
            symbol: "EURUSD".to_string(),
            datetime: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).expect("Invalid date"),
            open: price,
            high: price + 10.0,
            low: price - 10.0,
            close: price,
            volume: 100.0,
        }
    }

    #[test]
    fn test_confidence_score_low_sample() {
        let candles = vec![create_candle(100.0), create_candle(100.0)]; // 2 candles
        let result = simulate_straddle(&candles, "EURUSD");
        
        assert!(result.sample_size_warning);
    }

    #[test]
    fn test_confidence_score_high_sample() {
        let candles: Vec<Candle> = (0..15).map(|_| create_candle(100.0)).collect();
        let result = simulate_straddle(&candles, "EURUSD");
        
        assert!(!result.sample_size_warning);
    }
}

