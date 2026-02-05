// services/volatility_duration_analyzer.rs - Analyse durée volatilité (<250L)
use crate::models::{Candle, Stats15Min, VolatilityDuration, VolatilityError};
use crate::services::volatility::volatility_heuristics::VolatilityHeuristics;

pub struct VolatilityDurationAnalyzer;

impl VolatilityDurationAnalyzer {
    pub fn analyser(slice: &Stats15Min) -> Result<VolatilityDuration, VolatilityError> {
        if slice.candle_count == 0 {
            return Err(VolatilityError::InsufficientData(
                "Créneau vide".to_string(),
            ));
        }
        let peak_duration = VolatilityHeuristics::estimate_peak_duration(slice)?;
        let half_life =
            VolatilityHeuristics::estimate_half_life(peak_duration, slice.noise_ratio_mean)?;
        Ok(VolatilityDuration::new(
            slice.hour,
            slice.quarter,
            peak_duration,
            half_life,
            slice.candle_count as u16,
        ))
    }

    pub fn analyser_depuis_bougies(
        hour: u8,
        quarter: u8,
        candles: &[&Candle],
    ) -> Result<VolatilityDuration, VolatilityError> {
        if candles.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "Pas de bougies".to_string(),
            ));
        }
        if candles.len() < 5 {
            return Err(VolatilityError::InsufficientData(
                "Min 5 bougies requises".to_string(),
            ));
        }
        let atr_values = Self::calculer_valeurs_atr(candles)?;
        let peak_atr = atr_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if peak_atr <= 0.0 {
            return Err(VolatilityError::MetricCalculationError(
                "ATR peak invalide".to_string(),
            ));
        }
        let peak_index = atr_values
            .iter()
            .position(|&atr| (atr - peak_atr).abs() < 1e-10)
            .unwrap_or(0);
        let peak_duration = Self::calculer_duree_pic(&atr_values, peak_atr)?;
        let half_life = Self::calculer_demi_vie(&atr_values, peak_index, peak_atr)?;
        let final_half_life = if half_life >= peak_duration {
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

    fn calculer_valeurs_atr(candles: &[&Candle]) -> Result<Vec<f64>, VolatilityError> {
        if candles.len() < 4 {
            return Err(VolatilityError::InsufficientData(
                "Min 4 bougies pour ATR (période 3)".to_string(),
            ));
        }
        let mut atr_values = Vec::new();
        const PERIOD: f64 = 3.0;
        const PERIOD_USIZE: usize = 3;

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

        if tr_values.len() < PERIOD_USIZE {
            return Err(VolatilityError::InsufficientData(
                "Pas assez de données TR".to_string(),
            ));
        }

        let mut atr = tr_values.iter().take(PERIOD_USIZE).sum::<f64>() / PERIOD;
        atr_values.push(atr);
        for tr in tr_values.iter().skip(PERIOD_USIZE) {
            atr = (atr * (PERIOD - 1.0) + tr) / PERIOD;
            atr_values.push(atr);
        }
        Ok(atr_values)
    }

    fn calculer_duree_pic(atr_values: &[f64], peak_atr: f64) -> Result<u16, VolatilityError> {
        let threshold = peak_atr * 0.8;
        let peak_count = atr_values.iter().filter(|&&atr| atr > threshold).count();
        Ok(peak_count.max(1) as u16)
    }

    fn calculer_demi_vie(
        atr_values: &[f64],
        peak_index: usize,
        peak_atr: f64,
    ) -> Result<u16, VolatilityError> {
        let half_threshold = peak_atr / 2.0;
        let post_peak = &atr_values[peak_index.min(atr_values.len() - 1)..];
        let index = post_peak
            .iter()
            .position(|&atr| atr < half_threshold)
            .unwrap_or(post_peak.len());
        Ok(index.max(5) as u16)
    }

    pub fn calculer_delai_pic(
        atr_values: &[f64],
        event_minute: u8,
    ) -> Result<i16, VolatilityError> {
        if atr_values.is_empty() {
            return Err(VolatilityError::InsufficientData("ATR vide".to_string()));
        }
        let peak_atr = atr_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let peak_index = atr_values
            .iter()
            .position(|&atr| (atr - peak_atr).abs() < 1e-10)
            .unwrap_or(0);
        let peak_minute = peak_index as i16;
        let event_minute_i16 = event_minute as i16;
        let delay_minutes = peak_minute - event_minute_i16;
        Ok(delay_minutes.clamp(-10, 15))
    }

    pub fn calculer_profil_decroissance(atr_values: &[f64]) -> Result<(f64, String), VolatilityError> {
        if atr_values.len() < 12 {
            return Err(VolatilityError::InsufficientData(
                "Min 12 ATR values".to_string(),
            ));
        }
        let peak_atr = atr_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let peak_index = atr_values
            .iter()
            .position(|&atr| (atr - peak_atr).abs() < 1e-10)
            .ok_or_else(|| VolatilityError::MetricCalculationError("Peak not found".to_string()))?;
        let end_index = (peak_index + 10).min(atr_values.len() - 1);
        let atr_at_end = atr_values[end_index];
        let minutes_elapsed = (end_index - peak_index).max(1) as f64;
        let decay_rate = (peak_atr - atr_at_end) / minutes_elapsed;
        let decay_speed = if decay_rate > 3.0 {
            "FAST"
        } else if decay_rate > 1.5 {
            "MEDIUM"
        } else {
            "SLOW"
        };
        Ok((decay_rate, decay_speed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_candle(close: f64, high: f64, low: f64) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: close,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_analyze_typical() {
        let candles = vec![
            create_test_candle(1.0800, 1.0810, 1.0790),
            create_test_candle(1.0805, 1.0815, 1.0795),
            create_test_candle(1.0800, 1.0820, 1.0790),
            create_test_candle(1.0810, 1.0825, 1.0800),
            create_test_candle(1.0820, 1.0830, 1.0810),
            create_test_candle(1.0815, 1.0825, 1.0805),
            create_test_candle(1.0810, 1.0820, 1.0800),
            create_test_candle(1.0805, 1.0815, 1.0795),
            create_test_candle(1.0800, 1.0810, 1.0790),
            create_test_candle(1.0795, 1.0805, 1.0785),
            create_test_candle(1.0790, 1.0800, 1.0780),
            create_test_candle(1.0795, 1.0805, 1.0785),
            create_test_candle(1.0800, 1.0810, 1.0790),
            create_test_candle(1.0805, 1.0815, 1.0795),
            create_test_candle(1.0810, 1.0820, 1.0800),
            create_test_candle(1.0815, 1.0825, 1.0805),
        ];
        let candle_refs: Vec<&Candle> = candles.iter().collect();
        let result = VolatilityDurationAnalyzer::analyser_depuis_bougies(14, 0, &candle_refs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insufficient_candles() {
        let candles = [create_test_candle(1.0800, 1.0810, 1.0790)];
        let candle_refs: Vec<&Candle> = candles.iter().collect();
        assert!(VolatilityDurationAnalyzer::analyser_depuis_bougies(14, 0, &candle_refs).is_err());
    }

    #[test]
    fn test_peak_delay_after_event() {
        let atr_values = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.0, 0.9, 0.8];
        let result = VolatilityDurationAnalyzer::calculer_delai_pic(&atr_values, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_peak_delay_before_event() {
        let atr_values = vec![0.5, 0.6, 1.2, 1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3];
        let result = VolatilityDurationAnalyzer::calculer_delai_pic(&atr_values, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decay_profile_fast() {
        let atr_values = vec![
            5.0, 6.0, 7.0, 8.0, 9.0, 30.0, 25.0, 20.0, 15.0, 10.0, 5.0, 3.0, 2.0,
        ];
        let result = VolatilityDurationAnalyzer::calculer_profil_decroissance(&atr_values);
        assert!(result.is_ok());
        let (_, decay_speed) = result.expect("Should have decay profile");
        assert_eq!(decay_speed, "FAST");
    }

    #[test]
    fn test_decay_profile_slow() {
        let atr_values = vec![
            5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 11.5, 11.0, 10.5, 10.0, 9.5,
        ];
        let result = VolatilityDurationAnalyzer::calculer_profil_decroissance(&atr_values);
        assert!(result.is_ok());
        let (_, decay_speed) = result.expect("Should have decay profile");
        assert_eq!(decay_speed, "SLOW");
    }
}
