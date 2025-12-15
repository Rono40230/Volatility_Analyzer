/// Analyseurs simples pour peak delay et decay profile
use super::helpers::calculer_atr;
use crate::services::VolatilityDurationAnalyzer;
use chrono::{Duration, Timelike};

pub struct PeakDelayAnalyzer;
pub struct DecayProfileAnalyzer;

impl PeakDelayAnalyzer {
    pub async fn calculer(
        pair: &str,
        _event_type: &str,
        events: &[crate::models::CalendarEvent],
        loader: &crate::services::DatabaseLoader,
    ) -> Result<(Vec<i16>, Vec<f64>), String> {
        let mut peak_delays = Vec::new();
        let mut peak_atrs = Vec::new();

        for event in events {
            let window_start = event.event_time.and_utc() - Duration::hours(2);
            let window_end = event.event_time.and_utc() + Duration::hours(2);
            let candles = loader
                .load_candles_by_pair(pair, "M1", window_start, window_end)
                .unwrap_or_default();

            if !candles.is_empty() {
                let atr_values: Vec<f64> = candles
                    .iter()
                    .map(|c| calculer_atr(c.high, c.low, c.close))
                    .collect();
                if let Ok(pd) = VolatilityDurationAnalyzer::calculer_delai_pic(
                    &atr_values,
                    event.event_time.minute() as u8,
                ) {
                    peak_delays.push(pd);
                }
                peak_atrs.push(atr_values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
            }
        }

        if peak_delays.is_empty() {
            return Err(format!("Cannot compute peak: {}", pair));
        }

        Ok((peak_delays, peak_atrs))
    }
}

impl DecayProfileAnalyzer {
    pub async fn calculer(
        pair: &str,
        events: &[crate::models::CalendarEvent],
        loader: &crate::services::DatabaseLoader,
    ) -> Result<(Vec<f64>, Vec<f64>), String> {
        let mut decay_rates = Vec::new();
        let mut peak_atrs = Vec::new();

        for event in events {
            let window_start = event.event_time.and_utc() - Duration::hours(1);
            let window_end = event.event_time.and_utc() + Duration::hours(3);
            let candles = loader
                .load_candles_by_pair(pair, "M1", window_start, window_end)
                .unwrap_or_default();

            if !candles.is_empty() {
                let atr_values: Vec<f64> = candles
                    .iter()
                    .map(|c| calculer_atr(c.high, c.low, c.close))
                    .collect();
                if let Ok((rate, _)) =
                    VolatilityDurationAnalyzer::calculer_profil_decroissance(&atr_values)
                {
                    decay_rates.push(rate);
                }
                peak_atrs.push(atr_values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
            }
        }

        if decay_rates.is_empty() {
            return Err(format!("Cannot compute decay: {}", pair));
        }

        Ok((decay_rates, peak_atrs))
    }
}
