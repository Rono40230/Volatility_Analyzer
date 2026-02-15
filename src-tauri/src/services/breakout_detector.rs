// services/breakout_detector.rs
// Détecte les breakouts dans l'historique d'un créneau

use crate::models::Candle;

/// Détecte les breakouts dans un ensemble de bougies
///
/// Un breakout est défini comme :
/// - Le prix casse un niveau de support/résistance récent
/// - Le mouvement est confirmé par un body significatif (> 50% du range)
pub fn calculer_pourcentage_breakout(candles: &[Candle]) -> f64 {
    if candles.len() < 3 {
        return 0.0;
    }

    let mut breakout_count = 0;
    let total_candles = candles.len() - 2; // Les 2 premières bougies servent de référence, pas évaluées

    // Pour chaque bougie, vérifier si c'est un breakout
    for i in 2..candles.len() {
        let current = &candles[i];
        let prev1 = &candles[i - 1];
        let prev2 = &candles[i - 2];

        // Calculer les niveaux de support/résistance des 2 bougies précédentes
        let recent_high = prev1.high.max(prev2.high);
        let recent_low = prev1.low.min(prev2.low);

        // Body de la bougie actuelle
        let body = (current.close - current.open).abs();
        let range = current.high - current.low;
        let body_ratio = if range > 0.0 { body / range } else { 0.0 };

        // Vérifier si c'est un breakout haussier
        let bullish_breakout = current.close > recent_high && body_ratio > 0.5;

        // Vérifier si c'est un breakout baissier
        let bearish_breakout = current.close < recent_low && body_ratio > 0.5;

        if bullish_breakout || bearish_breakout {
            breakout_count += 1;
        }
    }

    // Retourner le pourcentage de breakouts
    (breakout_count as f64 / total_candles as f64) * 100.0
}

/// Analyse la qualité des breakouts (force, suivi, etc.)
#[allow(dead_code)]
pub fn analyze_breakout_quality(candles: &[Candle]) -> BreakoutQuality {
    if candles.len() < 5 {
        return BreakoutQuality::default();
    }

    let mut strong_breakouts = 0;
    let mut followed_through = 0;
    let total_breakouts = (candles.len() - 2) as f64;

    for i in 2..candles.len() - 2 {
        let current = &candles[i];
        let prev1 = &candles[i - 1];
        let prev2 = &candles[i - 2];
        let next1 = &candles[i + 1];
        let next2 = &candles[i + 2];

        let recent_high = prev1.high.max(prev2.high);
        let recent_low = prev1.low.min(prev2.low);

        let body = (current.close - current.open).abs();
        let range = current.high - current.low;
        let body_ratio = if range > 0.0 { body / range } else { 0.0 };

        // Breakout haussier
        if current.close > recent_high && body_ratio > 0.5 {
            // Fort si body > 70% du range
            if body_ratio > 0.7 {
                strong_breakouts += 1;
            }

            // Suivi si les 2 prochaines bougies confirment
            if next1.close > current.close && next2.close > next1.close {
                followed_through += 1;
            }
        }

        // Breakout baissier
        if current.close < recent_low && body_ratio > 0.5 {
            if body_ratio > 0.7 {
                strong_breakouts += 1;
            }

            if next1.close < current.close && next2.close < next1.close {
                followed_through += 1;
            }
        }
    }

    BreakoutQuality {
        strength_score: (strong_breakouts as f64 / total_breakouts) * 100.0,
        follow_through_rate: (followed_through as f64 / total_breakouts) * 100.0,
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BreakoutQuality {
    pub strength_score: f64,
    pub follow_through_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn make_candle(open: f64, high: f64, low: f64, close: f64, offset: i64) -> Candle {
        Candle {
            symbol: "EURUSD".to_string(),
            datetime: DateTime::<Utc>::from_timestamp_millis(
                1609459200000 + (offset * 60000),
            )
            .expect("valid ts"),
            open,
            high,
            low,
            close,
            volume: 100.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_empty_candles() {
        let percentage = calculer_pourcentage_breakout(&[]);
        assert_eq!(percentage, 0.0);
    }

    #[test]
    fn test_less_than_3_candles_returns_zero() {
        let candles = vec![
            make_candle(1.1000, 1.1020, 1.0990, 1.1010, 0),
            make_candle(1.1010, 1.1030, 1.0995, 1.1020, 1),
        ];
        assert_eq!(calculer_pourcentage_breakout(&candles), 0.0);
    }

    #[test]
    fn test_no_breakout() {
        // 3 bougies qui restent dans le range → 0%
        let candles = vec![
            make_candle(1.1000, 1.1020, 1.0990, 1.1010, 0),
            make_candle(1.1010, 1.1015, 1.0995, 1.1005, 1),
            make_candle(1.1005, 1.1015, 1.0995, 1.1010, 2), // close=1.1010 < recent_high=1.1020
        ];
        assert_eq!(calculer_pourcentage_breakout(&candles), 0.0);
    }

    #[test]
    fn test_bullish_breakout_detected() {
        // La 3e bougie casse le recent_high avec body > 50% du range
        let candles = vec![
            make_candle(1.1000, 1.1010, 1.0990, 1.1005, 0), // prev2
            make_candle(1.1005, 1.1015, 1.0995, 1.1010, 1), // prev1, recent_high=1.1015
            // Bougie 3 : close=1.1040 > recent_high=1.1015, body=0.003, range=0.004, ratio=75%
            make_candle(1.1010, 1.1045, 1.1005, 1.1040, 2),
        ];
        let pct = calculer_pourcentage_breakout(&candles);
        // 1 bougie evaluée, 1 breakout → 100%
        assert!((pct - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_denominator_is_len_minus_2() {
        // 5 bougies → 3 évaluées (index 2,3,4) → dénominateur = 3
        let candles = vec![
            make_candle(1.1000, 1.1010, 1.0990, 1.1005, 0),
            make_candle(1.1005, 1.1015, 1.0995, 1.1010, 1),
            // index 2 : breakout haussier (close > 1.1015, body dominant)
            make_candle(1.1010, 1.1045, 1.1005, 1.1040, 2),
            // index 3 : pas de breakout (reste plat)
            make_candle(1.1040, 1.1050, 1.1030, 1.1040, 3),
            // index 4 : pas de breakout
            make_candle(1.1040, 1.1048, 1.1035, 1.1042, 4),
        ];
        let pct = calculer_pourcentage_breakout(&candles);
        // 1 breakout / 3 évaluées = 33.33%
        assert!((pct - 100.0 / 3.0).abs() < 0.1);
    }

    #[test]
    fn test_bearish_breakout_detected() {
        let candles = vec![
            make_candle(1.1000, 1.1010, 1.0990, 1.1005, 0),
            make_candle(1.1005, 1.1015, 1.0995, 1.1010, 1), // recent_low = 1.0990
            // close=1.0960 < recent_low=1.0990, body=0.003, range=0.005, ratio=60% > 50%
            make_candle(1.0990, 1.0990, 1.0955, 1.0960, 2),
        ];
        let pct = calculer_pourcentage_breakout(&candles);
        assert!((pct - 100.0).abs() < 1e-10);
    }
}
