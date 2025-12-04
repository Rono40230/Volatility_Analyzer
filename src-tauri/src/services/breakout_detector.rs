// services/breakout_detector.rs
// Détecte les breakouts dans l'historique d'un créneau

use crate::models::Candle;

/// Détecte les breakouts dans un ensemble de bougies
///
/// Un breakout est défini comme :
/// - Le prix casse un niveau de support/résistance récent
/// - Le mouvement est confirmé par un body significatif (> 50% du range)
pub fn calculate_breakout_percentage(candles: &[Candle]) -> f64 {
    if candles.len() < 3 {
        return 0.0;
    }

    let mut breakout_count = 0;
    let total_candles = candles.len();

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

    #[test]
    fn test_empty_candles() {
        let percentage = calculate_breakout_percentage(&[]);
        assert_eq!(percentage, 0.0);
    }
}
