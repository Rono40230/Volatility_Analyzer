// models/stats_15min.rs - Statistiques par 15 minutes (pour scalping)
use crate::models::EventInHour;
use crate::models::hourly_stats_thresholds::*;
use serde::{Deserialize, Serialize};

/// Statistiques de volatilité pour une tranche de 15 minutes spécifique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats15Min {
    pub hour: u8,
    pub quarter: u8, // 0 = 00-15min, 1 = 15-30min, 2 = 30-45min, 3 = 45-60min
    pub candle_count: usize,
    pub atr_mean: f64,
    pub atr_max: f64,
    pub max_true_range: f64,
    pub volatility_mean: f64,
    pub range_mean: f64,
    pub body_range_mean: f64,
    pub p95_wick: f64,
    pub shadow_ratio_mean: f64,
    pub volume_imbalance_mean: f64,
    pub noise_ratio_mean: f64,
    pub breakout_percentage: f64,
    pub events: Vec<EventInHour>,
    // Analyse de décroissance de volatilité (TÂCHE 4) - par jour
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_duration_minutes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_half_life_minutes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_trade_expiration_minutes: Option<u16>,
    // Moyennes historiques sur toute la période (pour affichage dans le calendrier)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_duration_mean: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_half_life_mean: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_trade_expiration_mean: Option<u16>,
    // Profil de volatilité minute par minute (0-14) pour le graphique
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_profile: Option<Vec<f64>>,
    // Minute optimale d'entrée (0-14) basée sur le profil
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimal_entry_minute: Option<u8>,
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
            format!("{:02}:{:02}-{:02}:00", self.hour, start_min, end_hour)
        } else {
            format!(
                "{:02}:{:02}-{:02}:{:02}",
                self.hour, start_min, self.hour, end_min
            )
        }
    }

    /// Calcule un score de qualité global (0-100) pour scalping 15min
    /// `atr_factor` = `AssetProperties::atr_scaling_factor()` (1.0 pour ForexMajor)
    #[allow(dead_code)]
    pub fn quality_score_scaled(&self, atr_factor: f64) -> f64 {
        if self.candle_count == 0 {
            return 0.0;
        }
        let mut score: f64 = 0.0;

        // ATR adapté M15 : seuils H1 / 4 environ, puis × factor
        let m15_factor = atr_factor * 0.25; // M15 ATR ≈ H1 ATR / 4
        if self.atr_mean > scaled_atr_excellent(m15_factor) {
            score += 30.0;
        } else if self.atr_mean > scaled_atr_good(m15_factor) {
            score += 25.0;
        } else if self.atr_mean > scaled_atr_fair(m15_factor) {
            score += 20.0;
        } else if self.atr_mean > scaled_atr_poor(m15_factor) {
            score += 10.0;
        }

        // Body Range réaliste (25 pts) — universel
        if self.body_range_mean > BODY_RANGE_EXCELLENT {
            score += 25.0;
        } else if self.body_range_mean > BODY_RANGE_GOOD {
            score += 20.0;
        } else if self.body_range_mean > BODY_RANGE_FAIR {
            score += 15.0;
        } else if self.body_range_mean > BODY_RANGE_POOR {
            score += 8.0;
        }

        // Volatilité (bonus) (20 pts) — universel
        if self.volatility_mean > VOL_EXCELLENT {
            score += 20.0;
        } else if self.volatility_mean > VOL_GOOD {
            score += 16.0;
        } else if self.volatility_mean > VOL_FAIR {
            score += 12.0;
        } else if self.volatility_mean > VOL_POOR {
            score += 6.0;
        }

        // Noise Ratio (15 pts) — universel
        if self.noise_ratio_mean < NOISE_EXCELLENT {
            score += 15.0;
        } else if self.noise_ratio_mean < NOISE_GOOD {
            score += 10.0;
        } else if self.noise_ratio_mean < NOISE_FAIR {
            score += 5.0;
        }

        // Breakout % - CRITIQUE pour Straddle (10 pts) — universel
        if self.breakout_percentage > BREAKOUT_EXCELLENT {
            score += 10.0;
        } else if self.breakout_percentage > BREAKOUT_GOOD {
            score += 7.0;
        } else if self.breakout_percentage > BREAKOUT_FAIR {
            score += 4.0;
        }

        score.min(100.0)
    }

    /// Score avec seuils Forex Major (rétrocompatibilité)
    #[allow(dead_code)]
    pub fn quality_score(&self) -> f64 {
        self.quality_score_scaled(1.0)
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
