// services/straddle_simulator.rs
// Simule une stratégie Straddle sur l'historique complet d'un créneau

use super::straddle_adjustments::AdjustedMetrics;
use super::straddle_simulator_helpers::{
    calculate_atr_mean, calculate_risk_level, find_trade_resolution, get_whipsaw_coefficient,
};
use crate::models::Candle;

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

/// Normalise une valeur en pips selon le symbole
pub fn normalize_to_pips(value: f64, symbol: &str) -> f64 {
    let pip_value = get_pip_value(symbol);
    value / pip_value
}

/// Retourne la valeur d'1 pip pour une paire donnée
pub fn get_pip_value(symbol: &str) -> f64 {
    // Indices
    if symbol.contains("US30")
        || symbol.contains("DE30")
        || symbol.contains("NAS100")
        || symbol.contains("SPX500")
    {
        return 1.0;
    }
    // Crypto
    if symbol.contains("BTC") {
        return 1.0;
    }
    if symbol.contains("ETH") {
        return 0.1;
    }
    // JPY Pairs
    if symbol.contains("JPY") {
        return 0.01;
    }
    // XAU (Gold)
    if symbol.contains("XAU") {
        return 0.01;
    }
    // Default Forex
    0.0001
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
    let atr_mean = calculate_atr_mean(candles);

    // Simuler les trades avec pondération temporelle du whipsaw
    let mut total_trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut whipsaws = 0;
    let mut whipsaw_weight_sum = 0.0;
    let mut whipsaw_details_vec: Vec<WhipsawDetail> = Vec::new();

    let tp_distance = offset_optimal * 2.0;
    let sl_distance = offset_optimal;

    for (i, candle) in candles.iter().enumerate() {
        let open = candle.open;
        let high = candle.high;
        let low = candle.low;
        let _close = candle.close;
        let entry_time = candle.datetime;

        let buy_stop = open + offset_optimal;
        let sell_stop = open - offset_optimal;

        let mut trade_triggered = false;
        let mut is_win = false;
        let mut is_whipsaw = false;
        let mut whipsaw_duration_minutes = 0;

        // ===== BUY STOP =====
        if high >= buy_stop {
            trade_triggered = true;
            let buy_tp = buy_stop + tp_distance;
            let buy_sl = buy_stop - sl_distance;

            // TOUJOURS chercher la résolution complète dans les 15 minutes suivantes
            // Même si le SL/TP est touché dans la bougie d'entrée, il faut voir ce qui se passe après
            let result = find_trade_resolution(candles, i, entry_time, buy_tp, buy_sl, true);
            is_win = result.0;
            is_whipsaw = result.1;
            whipsaw_duration_minutes = result.2;
        }
        // ===== SELL STOP =====
        else if low <= sell_stop {
            trade_triggered = true;
            let sell_tp = sell_stop - tp_distance;
            let sell_sl = sell_stop + sl_distance;

            // TOUJOURS chercher la résolution complète dans les 15 minutes suivantes
            let result = find_trade_resolution(candles, i, entry_time, sell_tp, sell_sl, false);
            is_win = result.0;
            is_whipsaw = result.1;
            whipsaw_duration_minutes = result.2;
        }

        if trade_triggered {
            total_trades += 1;
            if is_win {
                wins += 1;
            } else {
                losses += 1;
                if is_whipsaw {
                    whipsaws += 1;
                    let coefficient = get_whipsaw_coefficient(whipsaw_duration_minutes);
                    whipsaw_weight_sum += coefficient;

                    // Enregistrer le détail du whipsaw
                    let buy_stop_detail = open + offset_optimal;
                    let sell_stop_detail = open - offset_optimal;

                    // Chercher les indices de déclenchement réels
                    let max_look = std::cmp::min(i + 15, candles.len());
                    let mut buy_trigger_idx = candles.len();
                    let mut sell_trigger_idx = candles.len();

                    for check_idx in (i + 1)..max_look {
                        if buy_trigger_idx == candles.len()
                            && candles[check_idx].high >= buy_stop_detail
                        {
                            buy_trigger_idx = check_idx;
                        }
                        if sell_trigger_idx == candles.len()
                            && candles[check_idx].low <= sell_stop_detail
                        {
                            sell_trigger_idx = check_idx;
                        }
                    }

                    whipsaw_details_vec.push(WhipsawDetail {
                        entry_index: i,
                        entry_price: open,
                        buy_stop: buy_stop_detail,
                        sell_stop: sell_stop_detail,
                        buy_trigger_index: buy_trigger_idx,
                        sell_trigger_index: sell_trigger_idx,
                    });
                }
            }
        }
    }

    let win_rate_percentage = if total_trades > 0 {
        (wins as f64 / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let whipsaw_frequency_percentage = if total_trades > 0 {
        (whipsaw_weight_sum / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let (risk_level, risk_color) = calculate_risk_level(whipsaw_frequency_percentage);

    // === CALCUL DES VALEURS PONDÉRÉES PAR LE WHIPSAW + VOLATILITÉ ===
    let adjusted = AdjustedMetrics::new(
        win_rate_percentage,
        offset_optimal_pips,
        whipsaw_frequency_percentage,
        atr_mean,
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

    #[test]
    fn test_pip_value() {
        assert_eq!(get_pip_value("EURUSD"), 0.0001);
        assert_eq!(get_pip_value("BTCUSD"), 1.00);
        assert_eq!(get_pip_value("USDJPY"), 0.01);
    }

    #[test]
    fn test_normalize_to_pips() {
        let value = 0.0020;
        assert_eq!(normalize_to_pips(value, "EURUSD"), 20.0);
    }
}
