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
/// 
/// Règles MT5 (référence officielle):
/// - Forex 5 décimales (EURUSD, GBPUSD, USDCAD): 1 pip = 10 points → pip_value = 0.0001
/// - JPY 3 décimales (USDJPY, EURJPY, CADJPY): 1 pip = 10 points → pip_value = 0.01
/// - Commodités or (XAUUSD, XAUJPY): 1 pip = 10 points → pip_value = 0.1
/// - Commodités argent (XAGUSD): 1 pip = 1000 points → pip_value = 0.001
/// - Indices (USA500IDXUSD): 1 pip = 1 point → pip_value = 1.0
/// - Crypto (BTCUSD): 1 pip = 1 point → pip_value = 1.0
pub fn get_pip_value(symbol: &str) -> f64 {
    // JPY pairs (3 décimales): 1 pip = 10 points
    if symbol.contains("JPY") {
        return 0.01;
    }
    
    // Commodités or (XAUUSD, XAUJPY): 1 pip = 10 points
    if symbol.contains("XAU") {
        return 0.1;
    }
    
    // Commodités argent (XAGUSD): 1 pip = 1000 points
    if symbol.contains("XAG") {
        return 0.001;
    }
    
    // Indices (USA500IDXUSD, US30, NAS100, SPX500): 1 pip = 1 point
    if symbol.contains("US30") 
        || symbol.contains("DE30")
        || symbol.contains("NAS100")
        || symbol.contains("SPX500")
        || symbol.contains("USA500")
    {
        return 1.0;
    }
    
    // Crypto (BTCUSD, ETHUSD): 1 pip = 1 point
    if symbol.contains("BTC") || symbol.contains("ETH") {
        return 1.0;
    }
    
    // Forex 5 décimales par défaut (EURUSD, GBPUSD, USDCAD, etc.): 1 pip = 10 points
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

    // CORRECTION PHASE 8: Simuler UN SEUL trade avec fenêtre 60 min
    // (Au lieu de tester chaque candle comme point d'entrée)
    let mut total_trades = 0;
    let mut total_trades = 0;
    let mut whipsaws = 0;
    let mut whipsaw_weight_sum = 0.0;
    let mut whipsaw_details_vec: Vec<WhipsawDetail> = Vec::new();

    let tp_distance = offset_optimal * 2.0;
    let sl_distance = offset_optimal;

    // CORRECTION PHASE 8: Compter les double-triggers (whipsaw) en 60 min
    // Ne pas calculer win/loss car TP est dynamique (trailing stop)
    if candles.len() >= 61 {
        let i = 0; // Trade au début du quarter optimal
        let candle = &candles[i];
        let open = candle.open;

        let buy_stop = open + offset_optimal;
        let sell_stop = open - offset_optimal;

        // Test fenêtre 60 min
        let window_end = std::cmp::min(i + 61, candles.len());
        let test_window = &candles[i + 1..window_end];

        let mut buy_triggered = false;
        let mut sell_triggered = false;
        let mut buy_trigger_idx = candles.len();
        let mut sell_trigger_idx = candles.len();

        // Parcourir la fenêtre de 60 minutes
        for (idx, check_candle) in test_window.iter().enumerate() {
            // ===== BUY STOP =====
            if !buy_triggered && check_candle.high >= buy_stop {
                buy_triggered = true;
                buy_trigger_idx = i + 1 + idx;
            }

            // ===== SELL STOP =====
            if !sell_triggered && check_candle.low <= sell_stop {
                sell_triggered = true;
                sell_trigger_idx = i + 1 + idx;
            }

            // ===== Double Trigger = Whipsaw =====
            // Si les 2 se sont déclenchés dans la fenêtre 60min
            if buy_triggered && sell_triggered {
                whipsaws = 1;
                total_trades = 1;
                let whipsaw_duration_minutes = (buy_trigger_idx.max(sell_trigger_idx) - buy_trigger_idx.min(sell_trigger_idx)) as i32;
                let coefficient = get_whipsaw_coefficient(whipsaw_duration_minutes);
                whipsaw_weight_sum = coefficient;

                whipsaw_details_vec.push(WhipsawDetail {
                    entry_index: i,
                    entry_price: open,
                    buy_stop,
                    sell_stop,
                    buy_trigger_index: buy_trigger_idx,
                    sell_trigger_index: sell_trigger_idx,
                });
                break;
            }
        }

        // Si pas de double trigger = pas de trade comptabilisé
        if total_trades == 0 {
            total_trades = 0;
        }
    }

    let mut wins = 0;
    let mut losses = 0;
    let win_rate_percentage = 0.0; // Pas calculable avec TP dynamique
    
    // Whipsaw frequency = si double trigger détecté = 100%, sinon 0%
    let whipsaw_frequency_percentage = if whipsaws > 0 {
        whipsaw_weight_sum * 100.0
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
