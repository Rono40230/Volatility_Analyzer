// models/volatility_duration.rs - Analyse de la durée et décroissance de la volatilité
// Conforme .clinerules : structures uniquement, pas de logique métier

use serde::{Deserialize, Serialize};

/// Analyse de la durée et du profil de décroissance de la volatilité pour un créneau horaire
///
/// Utilise la distribution empirique des mouvements pour estimater:
/// - Combien de temps dure le pic de volatilité
/// - Combien de temps pour que la volatilité décroisse de 50% (demi-vie)
/// - La durée de trading optimale basée sur ces métriques
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityDuration {
    /// Heure du créneau (0-23)
    pub hour: u8,
    /// Quarter du créneau (0-3, 0 = 00-15min, 1 = 15-30min, etc.)
    pub quarter: u8,
    /// Nombre de minutes où la volatilité reste > 80% du pic observé
    pub peak_duration_minutes: u16,
    /// Nombre de minutes pour que la volatilité décroisse de 50% du pic
    pub volatility_half_life_minutes: u16,
    /// Durée optimale de trading = max(peak_duration, half_life × 2)
    /// Basée sur la décroissance empirique de la volatilité
    pub recommended_trade_expiration_minutes: u16,
    /// Score de confiance (0-100) basé sur sample_size et variance des mesures
    pub confidence_score: u8,
    /// Nombre d'occurrences utilisées pour les calculs
    pub sample_size: u16,
}

impl VolatilityDuration {
    /// Crée une nouvelle instance avec tous les champs
    #[allow(dead_code)]
    pub fn new(
        hour: u8,
        quarter: u8,
        peak_duration_minutes: u16,
        volatility_half_life_minutes: u16,
        sample_size: u16,
    ) -> Self {
        // Calcule la durée de trading optimale : max(peak, 2 × half_life)
        let trade_exp_from_peak = peak_duration_minutes;
        let trade_exp_from_halflife = volatility_half_life_minutes.saturating_mul(2);
        let recommended_trade_expiration_minutes = trade_exp_from_peak.max(trade_exp_from_halflife);

        // Score de confiance basé sur la taille de l'échantillon
        // Minimum 50% confiance, jusqu'à 100% si sample_size >= 100
        let confidence_score = if sample_size >= 100 {
            100
        } else if sample_size >= 50 {
            90
        } else if sample_size >= 30 {
            75
        } else if sample_size >= 15 {
            60
        } else {
            50
        };

        Self {
            hour,
            quarter,
            peak_duration_minutes,
            volatility_half_life_minutes,
            recommended_trade_expiration_minutes,
            confidence_score: confidence_score as u8,
            sample_size,
        }
    }

    /// Retourne le label du créneau (ex: "14:30-14:45")
    #[allow(dead_code)]
    pub fn time_label(&self) -> String {
        let start_min = self.quarter * 15;
        let end_min = start_min + 15;
        
        // Gestion du débordement (45-60 devient 45-00)
        if end_min < 60 {
            format!(
                "{:02}:{:02}-{:02}:{:02}",
                self.hour, start_min, self.hour, end_min
            )
        } else {
            // 45-60 → 45-00
            format!(
                "{:02}:{:02}-{:02}:{:02}",
                self.hour, start_min, self.hour, 0
            )
        }
    }

    /// Valide que les valeurs sont cohérentes
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        // peak_duration doit être > 0 et <= 300 (5h)
        self.peak_duration_minutes > 0 && self.peak_duration_minutes <= 300
            // half_life doit être > 0 et < peak_duration
            && self.volatility_half_life_minutes > 0
            && self.volatility_half_life_minutes < self.peak_duration_minutes
            // trade_expiration doit être >= peak_duration
            && self.recommended_trade_expiration_minutes >= self.peak_duration_minutes
            // confidence entre 0-100
            && self.confidence_score <= 100
            // sample_size > 0
            && self.sample_size > 0
    }
}

#[cfg(test)]
#[path = "volatility_duration_tests.rs"]
mod tests;
