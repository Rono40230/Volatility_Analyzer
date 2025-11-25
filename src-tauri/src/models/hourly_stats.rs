// models/hourly_stats.rs - Statistiques par heure UTC
use serde::{Deserialize, Serialize};
use super::hourly_stats_thresholds::*;

/// Événement économique dans une heure spécifique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInHour {
    pub event_name: String,
    pub impact: String, // HIGH, MEDIUM, LOW
    pub datetime: String,
    pub volatility_increase: f64,
}

/// Statistiques de volatilité pour une heure UTC spécifique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: u8,
    pub candle_count: usize,
    pub atr_mean: f64,
    pub atr_max: f64,
    pub volatility_mean: f64,
    pub range_mean: f64,
    pub body_range_mean: f64,
    pub shadow_ratio_mean: f64,
    pub tick_quality_mean: f64,
    pub volume_imbalance_mean: f64,
    pub noise_ratio_mean: f64,
    pub breakout_percentage: f64,
    pub events: Vec<EventInHour>,
}

impl HourlyStats {
    /// Calcule un score de qualité global (0-100)
    pub fn quality_score(&self) -> f64 {
        if self.candle_count == 0 {
            return 0.0;
        }
        let mut score: f64 = 0.0;
        score += if self.atr_mean > ATR_EXCELLENT { 30.0 }
                 else if self.atr_mean > ATR_GOOD { 25.0 }
                 else if self.atr_mean > ATR_FAIR { 20.0 }
                 else if self.atr_mean > ATR_POOR { 10.0 } else { 0.0 };
        score += if self.body_range_mean > BODY_RANGE_EXCELLENT { 25.0 }
                 else if self.body_range_mean > BODY_RANGE_GOOD { 20.0 }
                 else if self.body_range_mean > BODY_RANGE_FAIR { 15.0 }
                 else if self.body_range_mean > BODY_RANGE_POOR { 8.0 } else { 0.0 };
        score += if self.volatility_mean > VOL_EXCELLENT { 20.0 }
                 else if self.volatility_mean > VOL_GOOD { 16.0 }
                 else if self.volatility_mean > VOL_FAIR { 12.0 }
                 else if self.volatility_mean > VOL_POOR { 6.0 } else { 0.0 };
        score += if self.noise_ratio_mean < NOISE_EXCELLENT { 15.0 }
                 else if self.noise_ratio_mean < NOISE_GOOD { 10.0 }
                 else if self.noise_ratio_mean < NOISE_FAIR { 5.0 } else { 0.0 };
        score += if self.breakout_percentage > BREAKOUT_EXCELLENT { 10.0 }
                 else if self.breakout_percentage > BREAKOUT_GOOD { 7.0 }
                 else if self.breakout_percentage > BREAKOUT_FAIR { 4.0 } else { 0.0 };
        score.min(100.0)
    }

    #[allow(dead_code)]
    pub fn quality_rating(&self) -> &'static str {
        let score = self.quality_score();
        if score >= 80.0 {
            "Excellent"
        } else if score >= 60.0 {
            "Good"
        } else if score >= 40.0 {
            "Fair"
        } else {
            "Poor"
        }
    }
}

#[cfg(test)]
#[path = "hourly_stats_tests.rs"]
mod tests;
