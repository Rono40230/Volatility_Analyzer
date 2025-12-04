// services/volatility_heuristics.rs - Estimations heuristiques
use crate::models::{Stats15Min, VolatilityError};

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CalendarEvent;

    fn create_test_slice(atr: f64, events_count: usize) -> Stats15Min {
        Stats15Min {
            hour: 14,
            quarter: 0,
            atr_mean: atr,
            body_range_mean: 50.0,
            noise_ratio_mean: 2.0,
            breakout_percent: 30.0,
            candle_count: 15,
            events: vec![CalendarEvent {
                id: 1,
                symbol: "EURUSD".to_string(),
                event_time: chrono::Utc::now(),
                impact: if events_count > 0 {
                    "HIGH".to_string()
                } else {
                    "LOW".to_string()
                },
                description: "Test".to_string(),
                actual: None,
                forecast: None,
                previous: None,
                created_at: chrono::Utc::now(),
            }][0..events_count]
                .to_vec(),
        }
    }

    #[test]
    fn test_estimate_peak_high_vol() {
        let slice = create_test_slice(0.0025, 1);
        let duration = VolatilityHeuristics::estimate_peak_duration(&slice).expect("Peak duration");
        assert!(duration >= 100);
    }

    #[test]
    fn test_estimate_peak_low_vol() {
        let slice = create_test_slice(0.0008, 0);
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
}
