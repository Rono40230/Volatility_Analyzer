// services/straddle_simulator_helpers.rs - Helpers pour simulation Straddle
// Contient les fonctions utilitaires pour éviter de dépasser 300 lignes

use crate::models::Candle;

/// Calcule l'ATR moyen (Average True Range) pour une liste de candles
/// Utilise une EMA(14) des True Ranges pour être conforme au standard MT5
/// et donner plus de poids aux mouvements récents
pub fn calculate_atr_mean(candles: &[Candle]) -> f64 {
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

/// Retourne le coefficient de pondération selon la durée du whipsaw
pub fn get_whipsaw_coefficient(minutes: i32) -> f64 {
    match minutes {
        0 => 1.0,      // Immédiat = coefficient 1.0 (très grave)
        1..=2 => 0.8,  // 1-2 min = coefficient 0.8
        3..=5 => 0.6,  // 3-5 min = coefficient 0.6
        6..=10 => 0.3, // 6-10 min = coefficient 0.3
        _ => 0.1,      // 11-15 min = coefficient 0.1 (très léger)
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

/// Cherche la résolution d'un trade (TP, SL ou timeout)
pub fn find_trade_resolution(
    candles: &[Candle],
    start_idx: usize,
    entry_time: chrono::DateTime<chrono::Utc>,
    tp_level: f64,
    sl_level: f64,
    is_buy: bool,
) -> (bool, bool, i32) {
    // Chercher dans les 15 MINUTES, pas dans les 15 indices suivants
    let max_time = entry_time + chrono::Duration::minutes(15);

    for candle in candles.iter().skip(start_idx + 1) {

        if candle.datetime > max_time {
            break;
        }

        let duration = candle.datetime.signed_duration_since(entry_time);
        let duration_minutes = duration.num_minutes() as i32;

        if is_buy {
            if candle.high >= tp_level {
                return (true, false, 0);
            }
            if candle.low <= sl_level {
                return (false, true, duration_minutes);
            }
        } else {
            if candle.low <= tp_level {
                return (true, false, 0);
            }
            if candle.high >= sl_level {
                return (false, true, duration_minutes);
            }
        }
    }

    // Non résolu = loss
    (false, false, 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whipsaw_coefficient() {
        assert_eq!(get_whipsaw_coefficient(0), 1.0);
        assert_eq!(get_whipsaw_coefficient(2), 0.8);
        assert_eq!(get_whipsaw_coefficient(5), 0.6);
    }

    #[test]
    fn test_risk_level() {
        let (level, _) = calculate_risk_level(5.0);
        assert_eq!(level, "Faible");
        let (level, _) = calculate_risk_level(25.0);
        assert_eq!(level, "Élevé");
    }
}
