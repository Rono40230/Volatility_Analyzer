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
/// Utilise les profils de volatilité par classe d'actif pour les valeurs par défaut
///
/// Méthode :
/// 1. Pour chaque jour, on identifie le pic de volatilité dans le créneau
/// 2. On mesure combien de temps la volatilité reste élevée après le pic
/// 3. On calcule la demi-vie (temps pour que la volatilité diminue de 50%)
/// 4. FIX 2.3: Utilise le profil d'asset_type pour les défauts au lieu de hardcoder
pub fn calculer_duree_volatilite(
    candles: &[Candle],
    _symbol: &str,
    default_peak_duration: Option<i64>,
    default_half_life: Option<i64>,
) -> VolatilityDurationResult {
    if candles.is_empty() {
        return VolatilityDurationResult {
            peak_duration_minutes: default_peak_duration.unwrap_or(5),
            volatility_half_life_minutes: default_half_life.unwrap_or(10),
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
            let peak_time = sorted_candles[peak_idx].datetime;

            // Calculer la durée du pic (combien de temps au-dessus de 75% du max)
            let threshold_75 = max_atr * 0.75;
            let mut last_above_75_idx = peak_idx;
            for (j, atr) in atrs.iter().enumerate().skip(peak_idx) {
                if atr.1 >= threshold_75 {
                    last_above_75_idx = j;
                } else {
                    break;
                }
            }
            // Utiliser les timestamps réels pour calculer la durée en minutes
            let duration = sorted_candles[last_above_75_idx]
                .datetime
                .signed_duration_since(peak_time)
                .num_minutes()
                + 1; // +1 pour inclure la bougie du pic
            if duration > 0 {
                peak_durations.push(duration);
            }

            // Calculer la demi-vie (temps pour que la volatilité passe sous 50% du max)
            let threshold_50 = max_atr * 0.5;
            let mut last_half_idx = peak_idx;
            for (j, atr) in atrs.iter().enumerate().skip(peak_idx) {
                if atr.1 >= threshold_50 {
                    last_half_idx = j;
                } else {
                    break;
                }
            }
            let half_life = sorted_candles[last_half_idx]
                .datetime
                .signed_duration_since(peak_time)
                .num_minutes()
                + 1; // +1 pour inclure la bougie du pic
            if half_life > 0 {
                half_lives.push(half_life);
            }
        }
    }

    // Calculer les moyennes (avec FIX 2.3: utiliser les profils pour défauts)
    let avg_peak_duration = if !peak_durations.is_empty() {
        peak_durations.iter().sum::<i64>() / peak_durations.len() as i64
    } else {
        default_peak_duration.unwrap_or(5) // Défaut 5 ou depuis profil
    };

    let avg_half_life = if !half_lives.is_empty() {
        half_lives.iter().sum::<i64>() / half_lives.len() as i64
    } else {
        default_half_life.unwrap_or(10) // Défaut 10 ou depuis profil
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

/// Wrapper pour appels simples (compatibilité tests)
pub fn calculer_duree_volatilite_simple(
    candles: &[Candle],
    _symbol: &str,
) -> VolatilityDurationResult {
    calculer_duree_volatilite(candles, _symbol, None, None)
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn make_candle_at(year: i32, month: u32, day: u32, hour: u32, min: u32, high: f64, low: f64) -> Candle {
        Candle {
            symbol: "EURUSD".to_string(),
            datetime: Utc.with_ymd_and_hms(year, month, day, hour, min, 0).unwrap(),
            open: (high + low) / 2.0,
            high,
            low,
            close: (high + low) / 2.0,
            volume: 100.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_empty_candles() {
        let result = calculer_duree_volatilite(&[], "EURUSD", None, None);
        assert_eq!(result.peak_duration_minutes, 5); // Défaut when empty
        assert_eq!(result.sample_size, 0);
    }

    #[test]
    fn test_peak_duration_uses_datetime_diff() {
        // Crée un jour avec un pic clair à min 5, haute vol jusqu'à min 8
        // Le pic devrait durer 4 minutes (min 5 à min 8 inclus)
        let candles = vec![
            make_candle_at(2025, 1, 6, 14, 0, 1.1010, 1.1000),  // range=0.001
            make_candle_at(2025, 1, 6, 14, 1, 1.1012, 1.1002),  // range=0.001
            make_candle_at(2025, 1, 6, 14, 2, 1.1015, 1.1005),  // range=0.001
            make_candle_at(2025, 1, 6, 14, 3, 1.1010, 1.1002),  // range=0.0008
            make_candle_at(2025, 1, 6, 14, 4, 1.1012, 1.1004),  // range=0.0008
            make_candle_at(2025, 1, 6, 14, 5, 1.1080, 1.1000),  // range=0.008 ← PIC
            make_candle_at(2025, 1, 6, 14, 6, 1.1070, 1.1000),  // range=0.007 (>75% de 0.008=0.006)
            make_candle_at(2025, 1, 6, 14, 7, 1.1065, 1.1005),  // range=0.006 (=75% → still above)
            make_candle_at(2025, 1, 6, 14, 8, 1.1060, 1.1010),  // range=0.005 (<0.006 → below 75%)
            make_candle_at(2025, 1, 6, 14, 9, 1.1020, 1.1010),  // range=0.001
        ];
        let result = calculer_duree_volatilite(&candles, "EURUSD", None, None);
        assert_eq!(result.sample_size, 1);
        // Pic à min5, last_above_75 at min7 (range=0.006 >= 0.006)
        // Duration = (min7 - min5).num_minutes() + 1 = 2 + 1 = 3
        assert_eq!(result.peak_duration_minutes, 3);
    }

    #[test]
    fn test_half_life_calculation() {
        // Pic à min 0, half_life quand vol drops below 50% du max
        let candles = vec![
            make_candle_at(2025, 1, 6, 14, 0, 1.1100, 1.1000),  // range=0.01 ← PIC
            make_candle_at(2025, 1, 6, 14, 1, 1.1080, 1.1010),  // range=0.007 (>50% de 0.01=0.005)
            make_candle_at(2025, 1, 6, 14, 2, 1.1060, 1.1010),  // range=0.005 (=50% → still above)
            make_candle_at(2025, 1, 6, 14, 3, 1.1030, 1.1010),  // range=0.002 (<0.005 → below 50%)
        ];
        let result = calculer_duree_volatilite(&candles, "EURUSD", None, None);
        // half_life = (min2 - min0).num_minutes() + 1 = 2 + 1 = 3
        assert_eq!(result.volatility_half_life_minutes, 3);
    }

    #[test]
    fn test_recommended_trade_expiration() {
        // recommended = max(2 * half_life, peak_duration)
        let candles = vec![
            make_candle_at(2025, 1, 6, 14, 0, 1.1100, 1.1000),  // PIC
            make_candle_at(2025, 1, 6, 14, 1, 1.1080, 1.1000),  // still above 75%
            make_candle_at(2025, 1, 6, 14, 2, 1.1060, 1.1010),  // range=0.005 (<0.0075 threshold)
            make_candle_at(2025, 1, 6, 14, 3, 1.1030, 1.1010),  // range=0.002 (<0.005 half)
        ];
        let result = calculer_duree_volatilite(&candles, "EURUSD", None, None);
        let expected_recommended = (result.volatility_half_life_minutes * 2)
            .max(result.peak_duration_minutes);
        assert_eq!(result.recommended_trade_expiration_minutes, expected_recommended);
        assert!(result.recommended_trade_expiration_minutes > 0);
    }

    #[test]
    fn test_confidence_based_on_sample_size() {
        // Avec 1 jour → sample_size=1 → confidence = 20.0
        let candles = vec![
            make_candle_at(2025, 1, 6, 14, 0, 1.1100, 1.1000),
            make_candle_at(2025, 1, 6, 14, 1, 1.1080, 1.1010),
            make_candle_at(2025, 1, 6, 14, 2, 1.1030, 1.1010),
        ];
        let result = calculer_duree_volatilite(&candles, "EURUSD", None, None);
        assert_eq!(result.sample_size, 1);
        assert!((result.confidence_score - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_too_few_candles_per_day() {
        // Un jour avec seulement 2 bougies → ignoré (< 3)
        let candles = vec![
            make_candle_at(2025, 1, 6, 14, 0, 1.1100, 1.1000),
            make_candle_at(2025, 1, 6, 14, 1, 1.1080, 1.1010),
        ];
        let result = calculer_duree_volatilite(&candles, "EURUSD", None, None);
        assert_eq!(result.sample_size, 0);
        // Defaults: 5 min peak, 10 min half_life
        assert_eq!(result.peak_duration_minutes, 5);
        assert_eq!(result.volatility_half_life_minutes, 10);
    }
}
