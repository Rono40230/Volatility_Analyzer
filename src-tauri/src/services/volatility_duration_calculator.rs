// services/volatility_duration_calculator.rs
// Calcule la durée de volatilité réelle basée sur l'historique d'un créneau

use crate::models::Candle;
use chrono::Timelike;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VolatilityDurationResult {
    pub peak_duration_minutes: i64,
    pub volatility_half_life_minutes: i64,
    pub recommended_trade_expiration_minutes: i64,
    pub confidence_score: f64,
    pub sample_size: usize,
}

/// Calcule la durée de volatilité en analysant comment la volatilité décroît après le pic
///
/// Méthode :
/// 1. Pour chaque jour, on identifie le pic de volatilité dans le créneau
/// 2. On mesure combien de temps la volatilité reste élevée après le pic
/// 3. On calcule la demi-vie (temps pour que la volatilité diminue de 50%)
pub fn calculer_duree_volatilite(
    candles: &[Candle],
    _symbol: &str,
) -> VolatilityDurationResult {
    if candles.is_empty() {
        return VolatilityDurationResult {
            peak_duration_minutes: 0,
            volatility_half_life_minutes: 0,
            recommended_trade_expiration_minutes: 0,
            confidence_score: 0.0,
            sample_size: 0,
        };
    }

    // Grouper les bougies par jour
    let mut candles_by_day: HashMap<String, Vec<&Candle>> = HashMap::new();
    for candle in candles {
        let day_key = candle.datetime.date_naive().to_string();
        candles_by_day.entry(day_key).or_default().push(candle);
    }

    let mut peak_durations: Vec<i64> = Vec::new();
    let mut half_lives: Vec<i64> = Vec::new();

    // Analyser chaque jour
    for (_day, day_candles) in candles_by_day.iter() {
        if day_candles.len() < 3 {
            continue; // Pas assez de données pour ce jour
        }

        // Trier les bougies par heure
        let mut sorted_candles = day_candles.clone();
        sorted_candles.sort_by_key(|c| c.datetime);

        // Calculer l'ATR pour chaque bougie
        let mut atrs: Vec<(i64, f64)> = Vec::new(); // (minute, atr)
        for (i, candle) in sorted_candles.iter().enumerate() {
            // Utilise le True Range réel via le module centralisé
            let prev_close = if i > 0 { Some(sorted_candles[i - 1].close) } else { None };
            let atr = crate::services::atr::calculate_true_range(
                candle.high, candle.low, prev_close,
            );
            let minute = candle.datetime.minute() as i64;
            atrs.push((minute, atr));
        }

        if atrs.is_empty() {
            continue;
        }

        // Trouver le pic de volatilité
        let max_atr = atrs
            .iter()
            .map(|(_, atr)| atr)
            .fold(0.0_f64, |a, &b| a.max(b));
        let peak_idx = atrs.iter().position(|(_, atr)| *atr == max_atr);

        if let Some(peak_idx) = peak_idx {
            // Calculer la durée du pic (combien de temps au-dessus de 75% du max)
            let threshold_75 = max_atr * 0.75;
            let mut duration = 0;
            for atr in atrs.iter().skip(peak_idx) {
                if atr.1 >= threshold_75 {
                    duration += 1;
                } else {
                    break;
                }
            }
            if duration > 0 {
                peak_durations.push(duration);
            }

            // Calculer la demi-vie (temps pour atteindre 50% du max)
            let threshold_50 = max_atr * 0.5;
            let mut half_life = 0;
            for atr in atrs.iter().skip(peak_idx) {
                if atr.1 >= threshold_50 {
                    half_life += 1;
                } else {
                    break;
                }
            }
            if half_life > 0 {
                half_lives.push(half_life);
            }
        }
    }

    // Calculer les moyennes
    let avg_peak_duration = if !peak_durations.is_empty() {
        peak_durations.iter().sum::<i64>() / peak_durations.len() as i64
    } else {
        5 // Défaut : 5 minutes
    };

    let avg_half_life = if !half_lives.is_empty() {
        half_lives.iter().sum::<i64>() / half_lives.len() as i64
    } else {
        10 // Défaut : 10 minutes
    };

    // Durée recommandée : 2x la demi-vie ou au moins la durée du pic
    let recommended_duration = (avg_half_life * 2).max(avg_peak_duration);

    // Score de confiance basé sur la taille de l'échantillon
    let sample_size = peak_durations.len();
    let confidence = if sample_size >= 100 {
        95.0
    } else if sample_size >= 50 {
        80.0
    } else if sample_size >= 20 {
        60.0
    } else if sample_size >= 10 {
        40.0
    } else {
        20.0
    };

    VolatilityDurationResult {
        peak_duration_minutes: avg_peak_duration,
        volatility_half_life_minutes: avg_half_life,
        recommended_trade_expiration_minutes: recommended_duration,
        confidence_score: confidence,
        sample_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_candles() {
        let result = calculer_duree_volatilite(&[], "EURUSD");
        assert_eq!(result.peak_duration_minutes, 0);
        assert_eq!(result.sample_size, 0);
    }
}
