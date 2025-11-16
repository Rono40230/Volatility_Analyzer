// models/stats_15min.rs - Statistiques par 15 minutes (pour scalping)
use crate::models::EventInHour;
use serde::{Deserialize, Serialize};

/// Statistiques de volatilité pour une tranche de 15 minutes spécifique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats15Min {
    pub hour: u8,
    pub quarter: u8, // 0 = 00-15min, 1 = 15-30min, 2 = 30-45min, 3 = 45-60min
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EventInHour>,
}

impl Stats15Min {
    /// Retourne le label de la tranche (ex: "00:00-00:15")
    pub fn time_label(&self) -> String {
        let start_min = self.quarter * 15;
        let end_min = start_min + 15;
        format!(
            "{:02}:{:02}-{:02}:{:02}",
            self.hour, start_min, self.hour, end_min
        )
    }

    /// Calcule un score de qualité global (0-100) pour scalping 15min
    pub fn quality_score(&self) -> f64 {
        if self.candle_count == 0 {
            return 0.0;
        }
        let mut score: f64 = 0.0;

        // ATR adapté Forex M1 scalping (30 pts)
        if self.atr_mean > 0.00025 {
            score += 30.0;
        } else if self.atr_mean > 0.00015 {
            score += 25.0;
        } else if self.atr_mean > 0.00010 {
            score += 20.0;
        } else if self.atr_mean > 0.00005 {
            score += 10.0;
        }

        // Body Range réaliste (25 pts)
        if self.body_range_mean > 45.0 {
            score += 25.0;
        } else if self.body_range_mean > 35.0 {
            score += 20.0;
        } else if self.body_range_mean > 25.0 {
            score += 15.0;
        } else if self.body_range_mean > 15.0 {
            score += 8.0;
        }

        // Volatilité (bonus) (20 pts)
        if self.volatility_mean > 0.30 {
            score += 20.0;
        } else if self.volatility_mean > 0.20 {
            score += 16.0;
        } else if self.volatility_mean > 0.10 {
            score += 12.0;
        } else if self.volatility_mean > 0.05 {
            score += 6.0;
        }

        // Noise Ratio (15 pts)
        if self.noise_ratio_mean < 2.0 {
            score += 15.0;
        } else if self.noise_ratio_mean < 3.0 {
            score += 10.0;
        } else if self.noise_ratio_mean < 4.0 {
            score += 5.0;
        }

        // Breakout % - CRITIQUE pour Straddle (10 pts)
        if self.breakout_percentage > 15.0 {
            score += 10.0;
        } else if self.breakout_percentage > 10.0 {
            score += 7.0;
        } else if self.breakout_percentage > 5.0 {
            score += 4.0;
        }

        score.min(100.0)
    }

    /// Retourne le rating textuel basé sur le score
    pub fn quality_rating(&self) -> &'static str {
        match self.quality_score() as u8 {
            80..=100 => "Excellent",
            60..=79 => "Bon",
            40..=59 => "Moyen",
            20..=39 => "Faible",
            _ => "Très faible",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_label() {
        let stats = Stats15Min {
            hour: 14,
            quarter: 0,
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
            events: Vec::new(),
        };
        assert_eq!(stats.time_label(), "14:00-14:15");

        let stats = Stats15Min {
            hour: 14,
            quarter: 2,
            ..stats
        };
        assert_eq!(stats.time_label(), "14:30-14:45");
    }

    #[test]
    fn test_quality_score() {
        let stats = Stats15Min {
            hour: 14,
            quarter: 0,
            candle_count: 100,
            atr_mean: 0.0003,
            atr_max: 0.0005,
            volatility_mean: 0.15,
            range_mean: 0.0008,
            body_range_mean: 50.0,
            shadow_ratio_mean: 1.2,
            tick_quality_mean: 0.0012,
            volume_imbalance_mean: 0.15,
            noise_ratio_mean: 1.8,
            breakout_percentage: 25.0,
            events: Vec::new(),
        };

        let score = stats.quality_score();
        assert!(score >= 0.0 && score <= 100.0);
        assert!(score > 50.0);
    }

    #[test]
    fn test_quality_score_empty() {
        let stats = Stats15Min {
            hour: 0,
            quarter: 0,
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
            events: Vec::new(),
        };

        assert_eq!(stats.quality_score(), 0.0);
    }
}
