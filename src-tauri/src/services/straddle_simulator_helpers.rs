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
    for i in period..values.len() {
        ema = values[i] * multiplier + ema * (1.0 - multiplier);
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
    max_duration: usize,
) -> Option<TradeOutcome> {
    let end_idx = candles.len().min(start_idx + max_duration + 1);
    let mut triggered_side: Option<&str> = None;
    let mut buy_trigger_idx = 0;
    let mut sell_trigger_idx = 0;

    for j in (start_idx + 1)..end_idx {
        let current = &candles[j];

        if triggered_side.is_none() {
            // Pas encore déclenché
            if current.high >= buy_stop && current.low <= sell_stop {
                // Whipsaw immédiat (déclenchement des deux côtés dans la même bougie)
                return Some(TradeOutcome {
                    result: "WHIPSAW".to_string(),
                    buy_trigger_idx: j,
                    sell_trigger_idx: j,
                });
            } else if current.high >= buy_stop {
                triggered_side = Some("BUY");
                buy_trigger_idx = j;
                
                if current.low <= sell_stop {
                    return Some(TradeOutcome {
                        result: "WHIPSAW".to_string(),
                        buy_trigger_idx: j,
                        sell_trigger_idx: j,
                    });
                }
                if current.high >= buy_stop + tp_distance {
                    return Some(TradeOutcome {
                        result: "WIN".to_string(),
                        buy_trigger_idx: j,
                        sell_trigger_idx: 0,
                    });
                }
            } else if current.low <= sell_stop {
                triggered_side = Some("SELL");
                sell_trigger_idx = j;
                
                if current.high >= buy_stop {
                    return Some(TradeOutcome {
                        result: "WHIPSAW".to_string(),
                        buy_trigger_idx: j,
                        sell_trigger_idx: j,
                    });
                }
                if current.low <= sell_stop - tp_distance {
                    return Some(TradeOutcome {
                        result: "WIN".to_string(),
                        buy_trigger_idx: 0,
                        sell_trigger_idx: j,
                    });
                }
            }
        } else {
            // Déjà déclenché
            match triggered_side {
                Some("BUY") => {
                    if current.low <= sell_stop {
                        return Some(TradeOutcome {
                            result: "WHIPSAW".to_string(),
                            buy_trigger_idx,
                            sell_trigger_idx: j,
                        });
                    }
                    if current.high >= buy_stop + tp_distance {
                        return Some(TradeOutcome {
                            result: "WIN".to_string(),
                            buy_trigger_idx,
                            sell_trigger_idx: 0,
                        });
                    }
                }
                Some("SELL") => {
                    if current.high >= buy_stop {
                        return Some(TradeOutcome {
                            result: "WHIPSAW".to_string(),
                            buy_trigger_idx: j,
                            sell_trigger_idx,
                        });
                    }
                    if current.low <= sell_stop - tp_distance {
                        return Some(TradeOutcome {
                            result: "WIN".to_string(),
                            buy_trigger_idx: 0,
                            sell_trigger_idx,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    if triggered_side.is_some() {
        Some(TradeOutcome {
            result: "TIMEOUT".to_string(),
            buy_trigger_idx,
            sell_trigger_idx,
        })
    } else {
        None
    }
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
