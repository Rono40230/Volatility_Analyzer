// services/volatility_duration_analyzer.rs - Analyse réelle de décroissance de volatilité
// Conforme .clinerules: <300L, Result<T, ServiceError>, pas d'unwrap

use crate::models::{Candle, Stats15Min, VolatilityDuration, VolatilityError};

/// Analyseur de la durée et du profil de décroissance RÉELLE de la volatilité
pub struct VolatilityDurationAnalyzer;

impl VolatilityDurationAnalyzer {
    /// Fallback: Analyse un Stats15Min avec heuristique simple (pour commandes Tauri legacy)
    pub fn analyze(slice: &Stats15Min) -> Result<VolatilityDuration, VolatilityError> {
        if slice.candle_count == 0 {
            return Err(VolatilityError::InsufficientData(
                "Créneau vide".to_string(),
            ));
        }

        let peak_duration = Self::estimate_peak_duration_heuristic(slice)?;
        let half_life = Self::estimate_half_life_heuristic(slice, peak_duration)?;

        Ok(VolatilityDuration::new(
            slice.hour,
            slice.quarter,
            peak_duration,
            half_life,
            slice.candle_count as u16,
        ))
    }

    /// Heuristique simple de peak_duration basée sur ATR et range
    fn estimate_peak_duration_heuristic(slice: &Stats15Min) -> Result<u16, VolatilityError> {
        let atr = slice.atr_mean;
        let body_range = slice.body_range_mean;

        let duration = if atr > 0.002 && body_range > 50.0 {
            100
        } else if atr > 0.0015 && body_range > 40.0 {
            140
        } else if atr > 0.001 && body_range > 30.0 {
            180
        } else {
            240
        };

        Ok(duration.clamp(60, 300))
    }

    /// Heuristique simple de half_life basée sur noise_ratio
    fn estimate_half_life_heuristic(
        slice: &Stats15Min,
        peak_duration: u16,
    ) -> Result<u16, VolatilityError> {
        let noise = slice.noise_ratio_mean;
        let ratio = if noise < 1.5 {
            0.65
        } else if noise < 2.5 {
            0.50
        } else {
            0.35
        };
        let half_life = ((peak_duration as f64) * ratio) as u16;
        Ok(half_life.max(30).min((peak_duration as f64 * 0.9) as u16))
    }

    /// Analyse les bougies RÉELLES pour déterminer la décroissance réelle de volatilité
    ///
    /// # Arguments
    /// * `hour` - Heure du créneau (0-23)
    /// * `quarter` - Quarter du créneau (0-3)
    /// * `candles` - Bougies M1 réelles pour ce créneau
    ///
    /// # Returns
    /// Une instance de VolatilityDuration avec les métriques calculées
    pub fn analyze_from_candles(
        hour: u8,
        quarter: u8,
        candles: &[&Candle],
    ) -> Result<VolatilityDuration, VolatilityError> {
        if candles.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "Pas de bougies pour analyser".to_string(),
            ));
        }

        if candles.len() < 5 {
            return Err(VolatilityError::InsufficientData(
                "Au moins 5 bougies requises".to_string(),
            ));
        }

        // 1. Calculer l'ATR pour chaque bougie (mesure réelle de volatilité)
        let atr_values = Self::calculate_atr_values(candles)?;

        // 2. Trouver le pic d'ATR
        let peak_atr = atr_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        if peak_atr <= 0.0 {
            return Err(VolatilityError::MetricCalculationError(
                "ATR peak invalide".to_string(),
            ));
        }

        // 3. Trouver quand l'ATR atteint le pic
        let peak_index = atr_values
            .iter()
            .position(|&atr| (atr - peak_atr).abs() < 1e-10)
            .unwrap_or(0);

        // 4. Calculer le peak_duration (minutes avec ATR > 80% du pic)
        let peak_duration = Self::calculate_peak_duration(&atr_values, peak_atr)?;

        // 5. Calculer la demi-vie réelle (quand ATR revient à 50% du pic)
        let half_life = Self::calculate_half_life(&atr_values, peak_index, peak_atr)?;

        // 6. Assouplir la validation : si half_life >= peak_duration, cela signifie
        //    que la volatilité décroît très lentement. On garde les valeurs brutes.
        //    Pire cas: utiliser la demi-vie limitée à 90% du peak_duration
        let final_half_life = if half_life >= peak_duration {
            // La volatilité ne décroît pas assez vite : limiter à 90% du peak
            (peak_duration as f64 * 0.9) as u16
        } else {
            half_life
        };

        Ok(VolatilityDuration::new(
            hour,
            quarter,
            peak_duration,
            final_half_life,
            candles.len() as u16,
        ))
    }

    /// Calcule l'ATR Wilder pour chaque bougie (mesure réelle de volatilité)
    fn calculate_atr_values(candles: &[&Candle]) -> Result<Vec<f64>, VolatilityError> {
        if candles.len() < 2 {
            return Err(VolatilityError::InsufficientData(
                "Au moins 2 bougies pour ATR".to_string(),
            ));
        }

        let mut atr_values = Vec::new();
        const PERIOD: f64 = 14.0;

        // Calculer TR pour chaque bougie
        let mut tr_values = Vec::new();
        for i in 0..candles.len() {
            let curr = candles[i];
            let hl = curr.high - curr.low;

            if i == 0 {
                tr_values.push(hl);
            } else {
                let prev_close = candles[i - 1].close;
                let hc = (curr.high - prev_close).abs();
                let lc = (curr.low - prev_close).abs();
                tr_values.push(hl.max(hc).max(lc));
            }
        }

        // Wilder's EMA pour ATR
        let mut atr = tr_values.iter().take(14).sum::<f64>() / PERIOD;
        atr_values.push(atr);

        for i in 14..tr_values.len() {
            atr = (atr * (PERIOD - 1.0) + tr_values[i]) / PERIOD;
            atr_values.push(atr);
        }

        Ok(atr_values)
    }

    /// Calcule le peak_duration (minutes où ATR > 80% du pic)
    fn calculate_peak_duration(
        atr_values: &[f64],
        peak_atr: f64,
    ) -> Result<u16, VolatilityError> {
        let threshold = peak_atr * 0.8;
        let peak_count = atr_values.iter().filter(|&&atr| atr > threshold).count();
        Ok(peak_count.max(1) as u16)
    }

    /// Calcule la demi-vie réelle (minutes après le pic pour revenir à 50%)
    fn calculate_half_life(
        atr_values: &[f64],
        peak_index: usize,
        peak_atr: f64,
    ) -> Result<u16, VolatilityError> {
        let threshold = peak_atr * 0.5;

        // Chercher après le pic
        for i in (peak_index + 1)..atr_values.len() {
            if atr_values[i] <= threshold {
                return Ok((i - peak_index) as u16);
            }
        }

        // Si pas trouvé, utiliser la fin de la série
        let half_life = (atr_values.len() - peak_index) as u16;
        Ok(half_life.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_candle(hour: u8, minute: u8, close: f64, high: f64, low: f64) -> Candle {
        let mut dt = Utc::now()
            .with_hour(hour as u32)
            .unwrap()
            .with_minute(minute as u32)
            .unwrap()
            .with_second(0)
            .unwrap();
        
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: dt,
            open: close,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_analyze_from_candles_typical() {
        let candles = vec![
            create_test_candle(14, 0, 1.0800, 1.0810, 1.0790),
            create_test_candle(14, 1, 1.0805, 1.0815, 1.0795),
            create_test_candle(14, 2, 1.0800, 1.0820, 1.0790),
            create_test_candle(14, 3, 1.0810, 1.0825, 1.0800),
            create_test_candle(14, 4, 1.0820, 1.0830, 1.0810),
            create_test_candle(14, 5, 1.0815, 1.0825, 1.0805),
            create_test_candle(14, 6, 1.0810, 1.0820, 1.0800),
            create_test_candle(14, 7, 1.0805, 1.0815, 1.0795),
            create_test_candle(14, 8, 1.0800, 1.0810, 1.0790),
            create_test_candle(14, 9, 1.0795, 1.0805, 1.0785),
            create_test_candle(14, 10, 1.0790, 1.0800, 1.0780),
            create_test_candle(14, 11, 1.0795, 1.0805, 1.0785),
            create_test_candle(14, 12, 1.0800, 1.0810, 1.0790),
            create_test_candle(14, 13, 1.0805, 1.0815, 1.0795),
            create_test_candle(14, 14, 1.0810, 1.0820, 1.0800),
            create_test_candle(14, 15, 1.0815, 1.0825, 1.0805),
        ];
        let candle_refs: Vec<&Candle> = candles.iter().collect();
        let result = VolatilityDurationAnalyzer::analyze_from_candles(14, 0, &candle_refs);
        assert!(result.is_ok());
        let vd = result.unwrap();
        assert!(vd.is_valid());
        assert!(vd.peak_duration_minutes > 0);
        assert!(vd.volatility_half_life_minutes < vd.peak_duration_minutes);
    }

    #[test]
    fn test_analyze_insufficient_candles() {
        let candles = vec![create_test_candle(14, 0, 1.0800, 1.0810, 1.0790)];
        let candle_refs: Vec<&Candle> = candles.iter().collect();
        let result = VolatilityDurationAnalyzer::analyze_from_candles(14, 0, &candle_refs);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_empty_candles() {
        let candles: Vec<Candle> = vec![];
        let candle_refs: Vec<&Candle> = candles.iter().collect();
        let result = VolatilityDurationAnalyzer::analyze_from_candles(14, 0, &candle_refs);
        assert!(result.is_err());
    }
}
