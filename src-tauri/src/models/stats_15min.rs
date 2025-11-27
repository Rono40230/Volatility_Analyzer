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
    pub volume_imbalance_mean: f64,
    pub noise_ratio_mean: f64,
    pub breakout_percentage: f64,
    pub events: Vec<EventInHour>,
    // Analyse de décroissance de volatilité (TÂCHE 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_duration_minutes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_half_life_minutes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_trade_expiration_minutes: Option<u16>,
}

impl Stats15Min {
    /// Retourne le label de la tranche (ex: "00:00-00:15", "23:45-00:00")
    #[allow(dead_code)]
    pub fn time_label(&self) -> String {
        let start_min = self.quarter * 15;
        let end_min = start_min + 15;
        
        // Si end_min = 60, c'est l'heure suivante
        if end_min >= 60 {
            let end_hour = (self.hour + 1) % 24;
            format!(
                "{:02}:{:02}-{:02}:00",
                self.hour, start_min, end_hour
            )
        } else {
            format!(
                "{:02}:{:02}-{:02}:{:02}",
                self.hour, start_min, self.hour, end_min
            )
        }
    }

    /// Calcule un score de qualité global (0-100) pour scalping 15min
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
#[path = "stats_15min_tests.rs"]
mod tests;
