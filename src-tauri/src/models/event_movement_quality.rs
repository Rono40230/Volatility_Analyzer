// models/event_movement_quality.rs - Analyse qualité mouvements événementiels
// Conforme .clinerules : structures uniquement, pas de logique métier

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Analyse qualité des mouvements générés par un type d'événement sur une paire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMovementQuality {
    pub id: Option<i32>,
    pub symbol: String,
    pub event_type: String,
    /// Proportion d'événements générant un mouvement directionnel > ATR × 0.75
    pub directional_move_rate: f64,
    /// Proportion d'événements avec reversal complet dans les 15 minutes
    pub whipsaw_rate: f64,
    /// Mouvement moyen en pips (à titre informatif)
    pub avg_pips_moved: f64,
    /// Proportion d'événements avec succès (mouvement directionnel sans reversal rapide)
    pub success_rate: f64,
    /// Score combiné (0-10) : directional_move_rate + (100 - whipsaw_rate) / 2
    pub quality_score: f64,
    /// Nombre d'occurrences analysées
    pub sample_size: i32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl EventMovementQuality {
    /// Crée une nouvelle instance avec les paramètres essentiels
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        event_type: String,
        directional_move_rate: f64,
        whipsaw_rate: f64,
        avg_pips_moved: f64,
        success_rate: f64,
        quality_score: f64,
        sample_size: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            symbol,
            event_type,
            directional_move_rate,
            whipsaw_rate,
            avg_pips_moved,
            success_rate,
            quality_score,
            sample_size,
            created_at: now,
            updated_at: now,
        }
    }

    /// Retourne une recommandation basée sur le quality_score
    #[allow(dead_code)]
    pub fn recommendation(&self) -> MovementRecommendation {
        match self.quality_score {
            score if score >= 7.0 => MovementRecommendation::HighQuality,
            score if score >= 5.0 => MovementRecommendation::MediumQuality,
            score if score >= 3.0 => MovementRecommendation::LowQuality,
            _ => MovementRecommendation::AvoidEvent,
        }
    }

    /// Vérifie si l'événement est suffisamment tradable
    #[allow(dead_code)]
    pub fn is_tradable(&self) -> bool {
        self.success_rate > 0.6 && self.quality_score >= 5.0
    }
}

/// Recommandations basées sur la qualité du mouvement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementRecommendation {
    HighQuality,
    MediumQuality,
    LowQuality,
    AvoidEvent,
}

impl std::fmt::Display for MovementRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighQuality => write!(f, "TRADE"),
            Self::MediumQuality => write!(f, "CAUTION"),
            Self::LowQuality => write!(f, "CAUTION"),
            Self::AvoidEvent => write!(f, "AVOID"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_movement_quality_creation() {
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.75,
            0.15,
            45.2,
            0.78,
            7.5,
            32,
        );

        assert_eq!(quality.symbol, "EURUSD");
        assert_eq!(quality.event_type, "NFP");
        assert!(quality.directional_move_rate > 0.0);
        assert!(quality.whipsaw_rate < 1.0);
    }

    #[test]
    fn test_quality_score_high_recommendation() {
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.85,
            0.10,
            50.0,
            0.82,
            8.5,
            45,
        );

        assert_eq!(
            quality.recommendation(),
            MovementRecommendation::HighQuality
        );
        assert!(quality.is_tradable());
    }

    #[test]
    fn test_quality_score_low_recommendation() {
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "FOMC".to_string(),
            0.45,
            0.55,
            20.0,
            0.35,
            2.5,
            15,
        );

        assert_eq!(quality.recommendation(), MovementRecommendation::AvoidEvent);
        assert!(!quality.is_tradable());
    }

    #[test]
    fn test_movement_recommendation_display() {
        assert_eq!(MovementRecommendation::HighQuality.to_string(), "TRADE");
        assert_eq!(MovementRecommendation::AvoidEvent.to_string(), "AVOID");
    }

    #[test]
    fn test_is_tradable_conditions() {
        // Cas 1 : Score bon mais success_rate bas
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.80,
            0.10,
            48.0,
            0.55, // < 0.6
            7.5,
            30,
        );
        assert!(!quality.is_tradable());

        // Cas 2 : Success_rate bon mais score bas
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.65,
            0.35,
            35.0,
            0.70,
            4.5, // < 5.0
            25,
        );
        assert!(!quality.is_tradable());

        // Cas 3 : Tous les critères OK
        let quality = EventMovementQuality::new(
            "EURUSD".to_string(),
            "NFP".to_string(),
            0.80,
            0.10,
            48.0,
            0.75, // > 0.6
            7.5,  // > 5.0
            30,
        );
        assert!(quality.is_tradable());
    }
}
