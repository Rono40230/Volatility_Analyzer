// services/straddle_simulator.rs
// Simule une stratégie Straddle sur l'historique complet d'un créneau

use super::straddle_adjustments::AdjustedMetrics;
use super::straddle_simulator_helpers::{calculer_atr_moyen, calculate_risk_level};
use crate::models::Candle;
use crate::services::pair_data::symbol_properties::normalize_to_pips;

#[derive(Debug, Clone)]
pub struct WhipsawDetail {
    pub entry_index: usize,
    pub entry_price: f64,
    pub buy_stop: f64,
    pub sell_stop: f64,
    pub buy_trigger_index: usize,
    pub sell_trigger_index: usize,
}

#[derive(Debug, Clone)]
pub struct StraddleSimulationResult {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate_percentage: f64,
    pub whipsaw_frequency_percentage: f64,
    pub offset_optimal_pips: f64,
    pub percentile_95_wicks: f64,
    pub risk_level: String,
    pub risk_color: String,
    // Valeurs pondérées par le whipsaw (Option B - affichage direct)
    pub win_rate_adjusted: f64,        // Win Rate pondéré par whipsaw
    pub sl_adjusted_pips: f64,         // SL ajusté par whipsaw
    pub trailing_stop_adjusted: f64,   // Trailing Stop réduit
    pub timeout_adjusted_minutes: i32, // Timeout réduit
    pub whipsaw_details: Vec<WhipsawDetail>, // Détails de chaque whipsaw
}

/// Simule une stratégie Straddle sur un ensemble de bougies avec tracking temporel du whipsaw
///
/// Stratégie : Place un ordre Buy Stop et Sell Stop à distance égale du prix d'ouverture
/// Whipsaw pondéré : Chaque whipsaw reçoit un coefficient selon QUAND il se produit
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
        };
    }

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
    let atr_mean = calculer_atr_moyen(candles);

    // === SIMULATION DES TRADES STRADDLE (AVEC DÉTECTION WHIPSAW) ===
    let mut total_trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut whipsaws = 0;
    let mut whipsaw_details_vec: Vec<WhipsawDetail> = Vec::new();

    let marge = offset_optimal;
    // Ratio TP:SL de 2:1 (Standard Straddle)
    // SL = Marge (l'autre côté du straddle)
    // TP = Marge * 2.0
    let tp_distance = marge * 2.0;

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

        // Enregistrement des résultats
        if let Some(result) = trade_result {
            total_trades += 1;
            match result {
                "WIN" => wins += 1,
                "WHIPSAW" => {
                    whipsaws += 1;
                    losses += 1; // Un whipsaw est une perte
                    whipsaw_details_vec.push(WhipsawDetail {
                        entry_index: i,
                        entry_price,
                        buy_stop,
                        sell_stop,
                        buy_trigger_index: buy_trigger_idx,
                        sell_trigger_index: sell_trigger_idx,
                    });
                }
                _ => losses += 1, // LOSS (Time out ou autre)
            }
        } else if triggered_side.is_some() {
            // Déclenché mais pas de résultat (Time out) -> Considéré comme perte ou neutre
            // Pour être conservateur, on compte comme perte si pas de TP
            total_trades += 1;
            losses += 1;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed as they tested moved functions
}
