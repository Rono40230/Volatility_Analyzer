// models/hourly_stats.rs - Statistiques par heure UTC
use serde::{Deserialize, Serialize};

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EventInHour>,
}

impl HourlyStats {
    /// Calcule un score de qualité global (0-100) - Adapté Forex M1 scalping
    pub fn quality_score(&self) -> f64 {
        if self.candle_count == 0 {
            return 0.0;
        }
        let mut score: f64 = 0.0;

        // ATR adapté Forex (30 pts)
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

    /// Calcule le score de potentiel de mouvement pour stratégie STRADDLE (0-100)
    ///
    /// PHILOSOPHIE :
    /// Straddle scalping cherche les heures avec gros mouvements nets et rapides.
    /// Cette métrique mesure le POTENTIEL DE PIC + VOLATILITÉ SOUTENUE.
    ///
    /// Formule :
    /// - Range (60%) : amplitude haute-basse de l'heure (gros mouvements)
    /// - ATR (25%)   : volatilité moyenne soutenue
    /// - BodyRange (15%) : directionnalité (vs bruit)
    ///
    /// Seuils Forex standard M1→H1 pour scalping agressif :
    /// - Range > 25 pips = excellent (0.0025 en prix absolu)
    /// - Range > 15 pips = bon (0.0015)
    /// - Range > 10 pips = acceptable (0.0010)
    /// - ATR > 15 pips = soutenu (0.0015)
    /// - ATR > 10 pips = acceptable (0.0010)
    /// - BodyRange > 30% = directif, pas du bruit
    pub fn movement_potential_score_straddle(&self) -> f64 {
        if self.candle_count == 0 {
            return 0.0;
        }
        let mut score: f64 = 0.0;

        // 1. RANGE (60 pts max) - Dominante pour straddle
        // Range mesure le mouvement total high-low de l'heure
        // Seuils basés sur données Forex M1 2024
        if self.range_mean > 0.0025 {
            // >25 pips = pic excellent
            score += 60.0;
        } else if self.range_mean > 0.0020 {
            // 20-25 pips = très bon
            score += 50.0;
        } else if self.range_mean > 0.0015 {
            // 15-20 pips = bon
            score += 40.0;
        } else if self.range_mean > 0.0010 {
            // 10-15 pips = acceptable
            score += 20.0;
        }

        // 2. ATR (25 pts max) - Volatilité soutenue
        // ATR valide que la volatilité est CONSTANTE, pas juste un spike isolé
        if self.atr_mean > 0.0020 {
            // >20 pips = excellent volatilité soutenue
            score += 25.0;
        } else if self.atr_mean > 0.0015 {
            // 15-20 pips = très bon
            score += 20.0;
        } else if self.atr_mean > 0.0010 {
            // 10-15 pips = bon
            score += 15.0;
        } else if self.atr_mean > 0.0005 {
            // 5-10 pips = acceptable
            score += 8.0;
        }

        // 3. BodyRange (15 pts max) - Directionnalité
        // BodyRange > 30% = bougies directionnelles, pas du bruit
        // Important pour straddle : on veut des mouvements nets
        if self.body_range_mean > 45.0 {
            // >45% = excellent, très directif
            score += 15.0;
        } else if self.body_range_mean > 35.0 {
            // 35-45% = bon
            score += 12.0;
        } else if self.body_range_mean > 25.0 {
            // 25-35% = acceptable
            score += 8.0;
        } else if self.body_range_mean > 15.0 {
            // 15-25% = limite
            score += 3.0;
        }

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
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_calculation() {
        let stats = HourlyStats {
            hour: 13,
            candle_count: 500,
            atr_mean: 0.0015,
            atr_max: 0.003,
            volatility_mean: 0.10,
            range_mean: 0.002,
            body_range_mean: 55.0,
            shadow_ratio_mean: 1.2,
            tick_quality_mean: 0.0012,
            volume_imbalance_mean: 0.15,
            noise_ratio_mean: 1.8,
            breakout_percentage: 25.0,
            events: Vec::new(),
        };

        let score = stats.quality_score();
        assert!(score >= 0.0 && score <= 100.0);
        assert!(score > 50.0); // Ces bonnes stats devraient donner > 50
    }

    #[test]
    fn test_quality_score_empty() {
        let stats = HourlyStats {
            hour: 0,
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

    #[test]
    fn test_quality_rating() {
        let excellent = HourlyStats {
            hour: 13,
            candle_count: 500,
            atr_mean: 0.002,
            atr_max: 0.004,
            volatility_mean: 0.12,
            range_mean: 0.003,
            body_range_mean: 60.0,
            shadow_ratio_mean: 1.5,
            tick_quality_mean: 0.0015,
            volume_imbalance_mean: 0.2,
            noise_ratio_mean: 1.5,
            breakout_percentage: 30.0,
            events: Vec::new(),
        };

        assert_eq!(excellent.quality_rating(), "Excellent");
    }
}
