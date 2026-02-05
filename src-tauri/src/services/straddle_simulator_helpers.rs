// services/straddle_simulator_helpers.rs - Helpers pour simulation Straddle
// Contient les fonctions utilitaires pour éviter de dépasser 300 lignes

use crate::models::Candle;
// use crate::services::straddle_types::{WhipsawDetail, StraddleSimulationResult};

/// Calcule l'ATR moyen (Average True Range) pour une liste de candles
/// Utilise une EMA(14) des True Ranges pour être conforme au standard MT5
/// et donner plus de poids aux mouvements récents
pub fn calculer_atr_moyen(candles: &[Candle]) -> f64 {
    let mut tr_values: Vec<f64> = Vec::new();

    // Calcul du True Range pour chaque candle
    for i in 0..candles.len() {
        let high = candles[i].high;
        let low = candles[i].low;
        let close = if i > 0 {
            candles[i - 1].close
        } else {
            candles[i].close
        };

        let tr = (high - low)
            .max((high - close).abs())
            .max((low - close).abs());
        tr_values.push(tr);
    }

    if tr_values.is_empty() {
        return 0.0;
    }

    // Calcul de l'EMA(14) des True Ranges
    calculate_ema(&tr_values, 14)
}

/// Calcule l'EMA (Exponential Moving Average) avec une période donnée
pub fn calculate_ema(values: &[f64], period: usize) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let period = period.min(values.len()); // Limiter la période au nombre de valeurs disponibles

    // Coefficient de lissage EMA = 2 / (period + 1)
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Initialiser avec la SMA des premières valeurs
    let sma_init: f64 = values[0..period].iter().sum::<f64>() / period as f64;
    let mut ema = sma_init;

    // Appliquer l'EMA sur les valeurs restantes
    for value in values.iter().skip(period) {
        ema = *value * multiplier + ema * (1.0 - multiplier);
    }

    ema
}

use crate::models::trading_costs::TradingCostProfile;

/// Récupère les coûts estimés (Spread + Slippage) pour le News Trading selon l'actif
pub fn get_asset_cost(symbol: &str) -> TradingCostProfile {
    TradingCostProfile::get_profile(symbol)
}

/// Calcule le percentile 95 GLOBAL des wicks
pub fn calculate_global_p95_wick(candles: &[Candle]) -> f64 {
    let mut all_wicks: Vec<f64> = Vec::new();
    for candle in candles {
        let upper_wick = candle.high - candle.close.max(candle.open);
        let lower_wick = candle.open.min(candle.close) - candle.low;
        if upper_wick > 0.0 { all_wicks.push(upper_wick); }
        if lower_wick > 0.0 { all_wicks.push(lower_wick); }
    }
    all_wicks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let global_p95_idx = ((all_wicks.len() as f64) * 0.95).ceil() as usize;
    if !all_wicks.is_empty() && global_p95_idx < all_wicks.len() {
        all_wicks[global_p95_idx]
    } else {
        0.0
    }
}

/// Calcule l'offset dynamique basé sur l'historique (P95 des 5 dernières bougies)
pub fn calculate_dynamic_offset(wicks_history: &[Vec<f64>], current_candle: &Candle) -> f64 {
    if wicks_history.is_empty() {
        let cw = current_candle;
        let uw = cw.high - cw.close.max(cw.open);
        let lw = cw.open.min(cw.close) - cw.low;
        uw.max(lw)
    } else {
        let mut recent_wicks: Vec<f64> = wicks_history.iter().flatten().cloned().collect();
        if recent_wicks.is_empty() {
            0.0
        } else {
            recent_wicks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let idx = ((recent_wicks.len() as f64) * 0.95).ceil() as usize;
            if idx < recent_wicks.len() { recent_wicks[idx] } else { 0.0 }
        }
    }
}

// --- SIMULATION LOGIC ---

pub struct TradeOutcome {
    pub result: String, // "WIN", "LOSS", "WHIPSAW", "TIMEOUT"
    pub buy_trigger_idx: usize,
    pub sell_trigger_idx: usize,
}

/// Simule le déroulement d'un trade sur une fenêtre de temps
pub fn simulate_trade_outcome(
    candles: &[Candle],
    start_idx: usize,
    buy_stop: f64,
    sell_stop: f64,
    tp_distance: f64,
    sl_distance: f64,
    max_duration: usize,
) -> Option<TradeOutcome> {
    crate::services::straddle::simulate_trade_outcome(
        candles,
        start_idx,
        buy_stop,
        sell_stop,
        tp_distance,
        sl_distance,
        max_duration,
    )
}

/// Calcule le risque et la couleur basé sur la fréquence whipsaw
pub fn calculate_risk_level(whipsaw_freq_pct: f64) -> (String, String) {
    if whipsaw_freq_pct < 10.0 {
        ("Faible".to_string(), "#22c55e".to_string())
    } else if whipsaw_freq_pct < 20.0 {
        ("Moyen".to_string(), "#eab308".to_string())
    } else if whipsaw_freq_pct < 30.0 {
        ("Élevé".to_string(), "#f97316".to_string())
    } else {
        ("Critique".to_string(), "#ef4444".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level() {
        let (level, _) = calculate_risk_level(5.0);
        assert_eq!(level, "Faible");
        let (level, _) = calculate_risk_level(25.0);
        assert_eq!(level, "Élevé");
    }
}
