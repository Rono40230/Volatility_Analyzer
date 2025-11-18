// services/volatility_duration_analyzer.rs - Analyse de la durée de volatilité par créneau
// Conforme .clinerules: <300L, Result<T, ServiceError>, pas d'unwrap

use crate::models::{Stats15Min, VolatilityDuration, VolatilityError};

/// Analyseur de la durée et du profil de décroissance de la volatilité
pub struct VolatilityDurationAnalyzer;

impl VolatilityDurationAnalyzer {
    /// Analyse un créneau 15min pour déterimer la durée du pic et la demi-vie de la volatilité
    ///
    /// # Arguments
    /// * `slice` - Les statistiques du créneau 15min contenant les données historiques
    ///
    /// # Returns
    /// Une instance de VolatilityDuration avec les métriques calculées
    pub fn analyze(slice: &Stats15Min) -> Result<VolatilityDuration, VolatilityError> {
        if slice.candle_count == 0 {
            return Err(VolatilityError::InsufficientData(
                "Impossible d'analyser créneau vide".to_string(),
            ));
        }

        // Calcule la durée du pic (où volatilité > 80% du max observé)
        let peak_duration = Self::estimate_peak_duration(slice)?;

        // Calcule la demi-vie (décroissance empirique de la volatilité)
        let half_life = Self::estimate_volatility_half_life(slice, peak_duration)?;

        // Valide que half_life < peak_duration
        if half_life >= peak_duration {
            return Err(VolatilityError::MetricCalculationError(
                "half_life doit être < peak_duration".to_string(),
            ));
        }

        // Le créneau a une taille moyenne d'environ 900 candles (900min ÷ 1min/candle)
        // On estime qu'on peut voir 2-3 cycles complets du créneau dans l'historique
        // Donc sample_size ≈ (candle_count × 2-3) / (peak_duration + half_life × 2)
        let estimated_occurrences = if peak_duration + half_life * 2 > 0 {
            ((slice.candle_count as u16 * 2) / (peak_duration + half_life * 2)).max(5)
        } else {
            5
        };

        Ok(VolatilityDuration::new(
            slice.hour,
            slice.quarter,
            peak_duration,
            half_life,
            estimated_occurrences,
        ))
    }

    /// Estime la durée du pic de volatilité (minutes où volatilité > 80% du max)
    ///
    /// Heuristique basée sur ATR et range:
    /// - ATR élevé (>0.002) + Range élevé (>60pts) = pic court et intense (90-150min)
    /// - ATR moyen (0.001-0.002) = pic modéré (120-200min)
    /// - ATR faible (<0.001) = plateau long (150-270min)
    fn estimate_peak_duration(slice: &Stats15Min) -> Result<u16, VolatilityError> {
        let atr = slice.atr_mean;
        let range = slice.range_mean;
        let body_range = slice.body_range_mean;

        // Heuristique empirique basée sur volatilité observée
        let duration_minutes = if atr > 0.002 && range > 60.0 {
            // Volatilité très élevée = pic intense et court
            if body_range > 50.0 {
                90 // Pic très pur et rapide
            } else {
                110 // Pic court mais bruité
            }
        } else if atr > 0.0015 && range > 45.0 {
            // Volatilité bonne
            if body_range > 40.0 {
                130 // Pic clair
            } else {
                155 // Pic moins pur
            }
        } else if atr > 0.001 && range > 30.0 {
            // Volatilité acceptable
            if body_range > 30.0 {
                180 // Pic décent
            } else {
                210 // Pic long
            }
        } else {
            // Volatilité faible = plateau prolongé
            if body_range > 20.0 {
                240
            } else {
                270
            }
        };

        // Limiter entre 60 et 300 minutes
        Ok(duration_minutes.clamp(60, 300))
    }

    /// Estime la demi-vie de la volatilité (temps pour décroître de 50%)
    ///
    /// Basée sur le rapport noise_ratio:
    /// - noise_ratio faible (<1.5) = volatilité stable = demi-vie longue
    /// - noise_ratio moyen (1.5-3.0) = décroissance normale = demi-vie 60-90min
    /// - noise_ratio élevé (>3.0) = volatilité très décroissante = demi-vie courte
    fn estimate_volatility_half_life(
        slice: &Stats15Min,
        peak_duration: u16,
    ) -> Result<u16, VolatilityError> {
        let noise = slice.noise_ratio_mean;
        let atr = slice.atr_mean;

        // La demi-vie est estimée comme une fraction du peak_duration
        // Plus le noise_ratio est bas, plus la volatilité persiste longtemps
        let half_life_ratio = if noise < 1.5 {
            // Volatilité très stable = demi-vie longue (60-70% du peak)
            0.65
        } else if noise < 2.5 {
            // Normal = demi-vie modérée (45-55% du peak)
            0.50
        } else {
            // Volatilité décroissante rapidement = demi-vie courte (30-40% du peak)
            0.35
        };

        let half_life = ((peak_duration as f64) * half_life_ratio) as u16;

        // Minimum 30min, maximum 90% du peak_duration
        let min_half_life = if atr > 0.002 { 30 } else { 45 };
        let max_half_life = (peak_duration as f64 * 0.9) as u16;

        Ok(half_life.max(min_half_life).min(max_half_life))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_slice(hour: u8, quarter: u8) -> Stats15Min {
        Stats15Min {
            hour,
            quarter,
            candle_count: 500,
            atr_mean: 0.0020,
            atr_max: 0.0035,
            volatility_mean: 0.25,
            range_mean: 50.0,
            body_range_mean: 45.0,
            shadow_ratio_mean: 0.5,
            tick_quality_mean: 0.0008,
            volume_imbalance_mean: 1.2,
            noise_ratio_mean: 2.0,
            breakout_percentage: 18.0,
            events: vec![],
        }
    }

    #[test]
    fn test_analyze_typical_scenario() {
        let slice = create_test_slice(14, 2);
        let result = VolatilityDurationAnalyzer::analyze(&slice);
        assert!(result.is_ok());
        let vd = result.unwrap();
        assert!(vd.is_valid());
        assert!(vd.peak_duration_minutes > 0 && vd.peak_duration_minutes <= 300);
        assert!(vd.volatility_half_life_minutes < vd.peak_duration_minutes);
    }

    #[test]
    fn test_high_volatility_scenario() {
        let mut slice = create_test_slice(14, 2);
        slice.atr_mean = 0.0025;
        slice.range_mean = 70.0;
        slice.body_range_mean = 55.0;
        let vd = VolatilityDurationAnalyzer::analyze(&slice)
            .expect("Devrait analyser volatilité élevée");
        // Volatilité élevée = pic court
        assert!(vd.peak_duration_minutes <= 150);
    }

    #[test]
    fn test_low_volatility_scenario() {
        let mut slice = create_test_slice(10, 0);
        slice.atr_mean = 0.0008;
        slice.range_mean = 20.0;
        slice.body_range_mean = 15.0;
        let vd = VolatilityDurationAnalyzer::analyze(&slice)
            .expect("Devrait analyser volatilité faible");
        // Volatilité faible = pic long
        assert!(vd.peak_duration_minutes >= 200);
    }

    #[test]
    fn test_stable_volatility_long_half_life() {
        let mut slice = create_test_slice(10, 0);
        slice.noise_ratio_mean = 1.2; // Très stable
        let vd = VolatilityDurationAnalyzer::analyze(&slice).expect("Devrait analyser");
        // Noise faible = half_life longue
        assert!(vd.volatility_half_life_minutes > 60);
    }

    #[test]
    fn test_empty_slice_error() {
        let slice = Stats15Min {
            hour: 14,
            quarter: 2,
            candle_count: 0,
            atr_mean: 0.0,
            atr_max: 0.0,
            volatility_mean: 0.0,
            range_mean: 0.0,
            body_range_mean: 0.0,
            shadow_ratio_mean: 0.0,
            tick_quality_mean: 0.0,
            volume_imbalance_mean: 0.0,
            noise_ratio_mean: 0.0,
            breakout_percentage: 0.0,
            events: vec![],
        };
        let result = VolatilityDurationAnalyzer::analyze(&slice);
        assert!(result.is_err());
    }
}
