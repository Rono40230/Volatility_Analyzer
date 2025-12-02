// services/volatility_decay_calculator.rs - Calcul décroissance isolé
use crate::models::VolatilityError;

pub struct DecayCalculator;

impl DecayCalculator {
    /// Calcul du taux de décroissance (pips/minute)
    pub fn calculate_decay_rate(atr_values: &[f64]) -> Result<f64, VolatilityError> {
        if atr_values.len() < 3 {
            return Err(VolatilityError::InsufficientData(
                "Besoin au moins 3 valeurs ATR".to_string(),
            ));
        }

        let peak_atr = atr_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if peak_atr <= 0.0 {
            return Err(VolatilityError::MetricCalculationError(
                "ATR peak invalide".to_string(),
            ));
        }

        let peak_idx = atr_values
            .iter()
            .position(|&atr| (atr - peak_atr).abs() < 1e-10)
            .unwrap_or(0);

        if peak_idx >= atr_values.len() - 1 {
            return Ok(0.0);
        }

        let post_peak = &atr_values[peak_idx + 1..];
        let half_peak = peak_atr / 2.0;

        let minutes_to_half = post_peak
            .iter()
            .position(|&atr| atr < half_peak)
            .unwrap_or(post_peak.len());

        if minutes_to_half == 0 {
            return Ok(0.0);
        }

        Ok(peak_atr / (minutes_to_half as f64))
    }

    /// Classement de la décroissance
    pub fn classify_decay_speed(decay_rate: f64) -> String {
        match decay_rate {
            r if r > 3.0 => "FAST".to_string(),
            r if r > 1.5 => "MEDIUM".to_string(),
            _ => "SLOW".to_string(),
        }
    }

    /// Timeout recommandé basé décroissance
    pub fn recommend_timeout(decay_rate: f64) -> i16 {
        match decay_rate {
            r if r > 3.0 => 18,
            r if r > 1.5 => 25,
            _ => 32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_rate_calculation() {
        let atr_values = vec![0.5, 0.8, 1.0, 0.9, 0.7, 0.5, 0.3];
        let decay = DecayCalculator::calculate_decay_rate(&atr_values).expect("decay rate");
        assert!(decay > 0.0);
    }

    #[test]
    fn test_decay_classification() {
        assert_eq!(DecayCalculator::classify_decay_speed(5.0), "FAST");
        assert_eq!(DecayCalculator::classify_decay_speed(2.0), "MEDIUM");
        assert_eq!(DecayCalculator::classify_decay_speed(0.8), "SLOW");
    }

    #[test]
    fn test_timeout_recommendation() {
        assert_eq!(DecayCalculator::recommend_timeout(5.0), 18);
        assert_eq!(DecayCalculator::recommend_timeout(2.0), 25);
        assert_eq!(DecayCalculator::recommend_timeout(0.8), 32);
    }
}
