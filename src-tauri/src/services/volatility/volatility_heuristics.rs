// services/volatility_heuristics.rs - Estimations heuristiques
use crate::models::{Stats15Min, VolatilityError};

#[cfg(test)]
use crate::models::EventInHour;

pub struct VolatilityHeuristics;

impl VolatilityHeuristics {
    /// Estime la durée du pic basée sur ATR et événements
    pub fn estimate_peak_duration(slice: &Stats15Min) -> Result<u16, VolatilityError> {
        let atr = slice.atr_mean;
        let body_range = slice.body_range_mean;
        let base_duration = if atr > 0.002 && body_range > 50.0 {
            100
        } else if atr > 0.0015 && body_range > 40.0 {
            140
        } else if atr > 0.001 && body_range > 30.0 {
            180
        } else {
            240
        };
        let duration = if !slice.events.is_empty() {
            let max_impact = slice
                .events
                .iter()
                .map(|e| e.impact.as_str())
                .max_by_key(|impact| match *impact {
                    "HIGH" => 3,
                    "MEDIUM" => 2,
                    _ => 1,
                })
                .unwrap_or("MEDIUM");
            match max_impact {
                "HIGH" => (base_duration as f64 * 1.5) as u16,
                "MEDIUM" => base_duration,
                "LOW" => (base_duration as f64 * 0.7) as u16,
                _ => base_duration,
            }
        } else {
            base_duration
        };
        Ok(duration.clamp(60, 300))
    }

    /// Estime la demi-vie basée sur noise ratio
    pub fn estimate_half_life(
        peak_duration: u16,
        noise_ratio: f64,
    ) -> Result<u16, VolatilityError> {
        let ratio = if noise_ratio < 1.5 {
            0.65
        } else if noise_ratio < 2.5 {
            0.50
        } else {
            0.35
        };
        let half_life = ((peak_duration as f64) * ratio) as u16;
        Ok(half_life.max(30).min((peak_duration as f64 * 0.9) as u16))
    }

    /// Détecte si le quarter présente un risque de "Doji Géant" (Whipsaw statique)
    ///
    /// Critères :
    /// 1. Volatilité significative (ATR > seuil minimal)
    /// 2. Corps petit par rapport au range (Indécision)
    ///
    /// Un Doji Géant signifie que le prix explose mais revient au point de départ.
    /// C'est le pire scénario pour un Straddle (déclenche les deux ordres + SL).
    pub fn is_giant_doji(stats: &Stats15Min) -> bool {
        // Seuil de volatilité minimale (pour ne pas flagger les dojis de nuit sans volume)
        // Stats15Min contient des valeurs normalisées (Pips/Points) après agrégation
        // 15 pips = percentile ~75 d'ATR M15 sur Forex majeurs. En-dessous, le doji n'est
        // pas assez "géant" pour constituer un vrai risque de whipsaw.
        const MIN_ATR_FOR_WHIPSAW: f64 = 15.0;
        // Corps < 35% du range = indécision marquée (mèches > corps)
        const MAX_BODY_RATIO: f64 = 35.0;

        // Si l'ATR est faible, ce n'est pas un "Giant" Doji, juste un Doji calme (pas grave)
        if stats.atr_mean < MIN_ATR_FOR_WHIPSAW {
            return false;
        }

        // Si le corps moyen est petit (< 35%) alors que ça bouge bien
        if stats.body_range_mean.abs() < MAX_BODY_RATIO {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_slice(atr: f64, events_count: usize) -> Stats15Min {
        Stats15Min {
            hour: 14,
            quarter: 0,
            atr_mean: atr,
            atr_max: atr * 1.2,
            volatility_mean: 0.2,
            range_mean: atr * 1.1,
            body_range_mean: 20.0,
            shadow_ratio_mean: 0.5,
            volume_imbalance_mean: 0.1,
            noise_ratio_mean: 2.0,
            breakout_percentage: 30.0,
            candle_count: 15,
            events: vec![EventInHour {
                event_name: "Test Event".to_string(),
                impact: if events_count > 0 {
                    "HIGH".to_string()
                } else {
                    "LOW".to_string()
                },
                datetime: "2025-01-15 14:00:00".to_string(),
                volatility_increase: 0.5,
            }][0..events_count.min(1)]
                .to_vec(),
            peak_duration_minutes: None,
            volatility_half_life_minutes: None,
            recommended_trade_expiration_minutes: None,
            peak_duration_mean: None,
            volatility_half_life_mean: None,
            recommended_trade_expiration_mean: None,
            max_true_range: 0.0,
            p95_wick: 0.0,
            straddle_parameters: None,
            volatility_profile: None,
            optimal_entry_minute: None,
        }
    }

    #[test]
    fn test_estimate_peak_high_vol() {
        let slice = create_test_slice(25.0, 1); // 25 pips
        let duration = VolatilityHeuristics::estimate_peak_duration(&slice).expect("Peak duration");
        assert!(duration >= 100);
    }

    #[test]
    fn test_estimate_peak_low_vol() {
        let slice = create_test_slice(8.0, 0); // 8 pips
        let duration = VolatilityHeuristics::estimate_peak_duration(&slice).expect("Peak duration");
        assert!(duration <= 300);
    }

    #[test]
    fn test_estimate_half_life_clean() {
        let half_life = VolatilityHeuristics::estimate_half_life(100, 1.0).expect("Half life");
        assert!(half_life > 0);
    }

    #[test]
    fn test_estimate_half_life_noisy() {
        let half_life = VolatilityHeuristics::estimate_half_life(100, 3.0).expect("Half life");
        assert!(half_life > 0);
    }

    #[test]
    fn test_is_giant_doji_true() {
        let slice = create_test_slice(30.0, 1); // 30 pips
        assert!(VolatilityHeuristics::is_giant_doji(&slice));
    }

    #[test]
    fn test_is_giant_doji_false_atr() {
        let slice = create_test_slice(10.0, 1); // 10 pips
        assert!(!VolatilityHeuristics::is_giant_doji(&slice));
    }

    #[test]
    fn test_is_giant_doji_false_body() {
        let mut slice = create_test_slice(40.0, 1); // 40 pips
        slice.body_range_mean = 40.0;
        assert!(!VolatilityHeuristics::is_giant_doji(&slice));
    }
}
