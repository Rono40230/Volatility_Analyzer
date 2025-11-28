// services/straddle_simulator.rs
// Simule une stratégie Straddle sur l'historique complet d'un créneau

use crate::models::Candle;

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
}

/// Normalise une valeur en pips selon le symbole
pub fn normalize_to_pips(value: f64, symbol: &str) -> f64 {
    let pip_value = get_pip_value(symbol);
    value / pip_value
}

/// Retourne la valeur d'1 pip pour une paire donnée
pub fn get_pip_value(symbol: &str) -> f64 {
    // Indices
    if symbol.contains("US30") || symbol.contains("DE30") || symbol.contains("NAS100") || symbol.contains("SPX500") {
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

/// Simule une stratégie Straddle sur un ensemble de bougies
/// 
/// Stratégie : Place un ordre Buy Stop et Sell Stop à distance égale du prix d'ouverture
/// - Si le prix monte et touche le Buy Stop, on gagne si ça continue, on perd si ça revient (whipsaw)
/// - Si le prix descend et touche le Sell Stop, on gagne si ça continue, on perd si ça revient (whipsaw)
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

    let offset_optimal = p95_wick * 1.1; // Ajouter 10% de marge
    let offset_optimal_pips = normalize_to_pips(offset_optimal, symbol);

    // Simuler les trades
    let mut total_trades = 0;
    let mut wins = 0;
    let mut losses = 0;
    let mut whipsaws = 0;
    let mut whipsaw_weight_sum = 0.0; // Pour le calcul pondéré

    for (i, candle) in candles.iter().enumerate() {
        // Pour chaque bougie, on simule un trade Straddle
        let open = candle.open;
        let high = candle.high;
        let low = candle.low;
        let close = candle.close;

        let buy_stop = open + offset_optimal;
        let sell_stop = open - offset_optimal;

        // Définir TP et SL (simplifié : 2x l'offset pour TP, 1x pour SL)
        let tp_distance = offset_optimal * 2.0;
        let sl_distance = offset_optimal;

        let mut trade_triggered = false;
        let mut is_win = false;
        let mut is_whipsaw = false;
        let mut whipsaw_duration_coefficient = 1.0; // Par défaut, très grave

        // Vérifier si le Buy Stop est touché
        if high >= buy_stop {
            trade_triggered = true;
            let buy_tp = buy_stop + tp_distance;
            let buy_sl = buy_stop - sl_distance;

            if high >= buy_tp {
                // TP atteint
                is_win = true;
            } else if low <= buy_sl {
                // SL atteint (whipsaw si le prix est monté puis redescendu)
                is_whipsaw = true;
                // Whipsaw détecté dans la même bougie = très rapide = coefficient 1.0
                whipsaw_duration_coefficient = 1.0;
            } else {
                // Trade en cours, on considère le close
                if close >= buy_stop + (tp_distance * 0.5) {
                    is_win = true;
                } else {
                    is_whipsaw = true;
                    // Trade fermé dans la même bougie = rapide = coefficient 0.8
                    whipsaw_duration_coefficient = 0.8;
                }
            }
        }
        // Vérifier si le Sell Stop est touché
        else if low <= sell_stop {
            trade_triggered = true;
            let sell_tp = sell_stop - tp_distance;
            let sell_sl = sell_stop + sl_distance;

            if low <= sell_tp {
                // TP atteint
                is_win = true;
            } else if high >= sell_sl {
                // SL atteint (whipsaw)
                is_whipsaw = true;
                whipsaw_duration_coefficient = 1.0;
            } else {
                // Trade en cours
                if close <= sell_stop - (tp_distance * 0.5) {
                    is_win = true;
                } else {
                    is_whipsaw = true;
                    whipsaw_duration_coefficient = 0.8;
                }
            }
        }

        if trade_triggered {
            total_trades += 1;
            if is_win {
                wins += 1;
            } else {
                losses += 1;
                if is_whipsaw {
                    whipsaws += 1;
                    // Ajouter le poids du whipsaw selon le coefficient
                    whipsaw_weight_sum += whipsaw_duration_coefficient;
                }
            }
        }
    }

    let win_rate_percentage = if total_trades > 0 {
        (wins as f64 / total_trades as f64) * 100.0
    } else {
        0.0
    };

    // Calculer la fréquence whipsaw pondérée par durée
    // Au lieu de: whipsaws / total_trades
    // On utilise: whipsaw_weight_sum / total_trades
    // Cela réduit significativement le % pour les whipsaws longs
    let whipsaw_frequency_percentage = if total_trades > 0 {
        (whipsaw_weight_sum / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let (risk_level, risk_color) = calculate_risk_level(whipsaw_frequency_percentage);

    StraddleSimulationResult {
        total_trades,
        wins,
        losses,
        whipsaws,
        win_rate_percentage,
        whipsaw_frequency_percentage,
        offset_optimal_pips,
        percentile_95_wicks: normalize_to_pips(p95_wick, symbol),
        risk_level,
        risk_color,
    }
}

fn calculate_risk_level(whipsaw_freq_pct: f64) -> (String, String) {
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
    fn test_pip_value() {
        assert_eq!(get_pip_value("EURUSD"), 0.0001);
        assert_eq!(get_pip_value("BTCUSD"), 1.00);
        assert_eq!(get_pip_value("USDJPY"), 0.01);
    }

    #[test]
    fn test_normalize_to_pips() {
        let value = 0.0020; // 20 pips pour EURUSD
        assert_eq!(normalize_to_pips(value, "EURUSD"), 20.0);
        
        let value_btc = 100.0; // 100 pips pour BTCUSD
        assert_eq!(normalize_to_pips(value_btc, "BTCUSD"), 100.0);
    }
}
