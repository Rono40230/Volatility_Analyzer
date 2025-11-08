// models/event_metrics.rs - Métriques événementielles pour optimisation EA Straddle
// Conforme .clinerules : < 150L, structures uniquement, pas de logique métier

use serde::{Deserialize, Serialize};

/// Métriques complètes calculées pour un type d'événement spécifique
/// Permet d'optimiser les paramètres d'un EA Straddle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "diesel", derive(Queryable, Insertable))]
#[cfg_attr(feature = "diesel", diesel(table_name = event_metrics))]
pub struct EventMetrics {
    /// Type d'événement (ex: "NFP", "CPI", "FOMC")
    pub event_type: String,
    
    /// Symbole de la paire (ex: "EURUSD")
    pub pair_symbol: String,
    
    /// Durée moyenne du pic de volatilité (minutes)
    /// Utilité: Paramètre TradeExpiration optimal
    pub avg_duration_minutes: f64,
    
    /// Temps moyen avant le pic maximum (minutes)
    /// Utilité: Identifier quand se produit le mouvement max
    pub peak_time_minutes: f64,
    
    /// Temps moyen pour retour à volatilité normale (minutes)
    pub return_to_normal_minutes: f64,
    
    /// Win rate historique simulé (0-1)
    /// % de trades gagnants si on avait tradé avec paramètres standard
    /// Utilité: Validation faisabilité, filtrage événements
    pub win_rate: f64,
    
    /// Mouvement moyen en pips
    /// Utilité: Calcul TakeProfit optimal
    pub avg_movement_pips: f64,
    
    /// Mouvement maximum observé (pips)
    pub max_movement_pips: f64,
    
    /// Taux de whipsaw (fausses cassures) (0-1)
    /// % d'événements avec aller-retour sans mouvement directionnel
    /// Utilité: Détection événements à éviter
    pub whipsaw_rate: f64,
    
    /// Meilleur timing d'entrée (minutes AVANT événement)
    /// Ex: -15 = entrer 15 minutes avant
    /// Utilité: Paramètre EntryMinutesBeforeEvent
    pub best_entry_minutes_before: i32,
    
    /// ATR moyen 30min AVANT événement
    /// Utilité: Contexte baseline pour comparaison
    pub contextual_atr_before: f64,
    
    /// ATR moyen 30min APRÈS événement
    /// Utilité: Mesurer boost volatilité
    pub contextual_atr_after: f64,
    
    /// Ratio ATR_after / ATR_before
    /// Ex: 2.5 = volatilité multipliée par 2.5
    pub atr_increase_ratio: f64,
    
    /// Multiplicateur ATR recommandé pour SL
    /// Calculé pour avoir taux succès optimal
    pub recommended_sl_multiplier: f64,
    
    /// Multiplicateur ATR recommandé pour TP
    pub recommended_tp_multiplier: f64,
    
    /// Score de tradabilité global (0-10)
    /// Basé sur: win_rate, avg_movement, whipsaw_rate, reliability
    /// >7 = TRADE, 4-7 = CAUTION, <4 = AVOID
    pub tradability_score: f64,
    
    /// Recommandation automatique
    pub recommendation: TradingRecommendation,
    
    /// Nombre d'occurrences analysées
    /// Utilité: Fiabilité statistique (>20 recommandé)
    pub sample_size: i32,
    
    /// Date dernier calcul (cache)
    pub last_calculated: String,
}

/// Recommandation de trading pour un événement spécifique
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingRecommendation {
    /// Score >7, win rate >70%, whipsaw <20%
    Trade,
    
    /// Score 4-7, win rate 55-70%, whipsaw 20-35%
    Caution,
    
    /// Score <4, win rate <55%, ou whipsaw >35%
    Avoid,
}

impl TradingRecommendation {
    /// Détermine recommandation basée sur métriques
    pub fn from_metrics(score: f64, win_rate: f64, whipsaw_rate: f64) -> Self {
        if score >= 7.0 && win_rate >= 0.70 && whipsaw_rate < 0.20 {
            Self::Trade
        } else if score >= 4.0 && win_rate >= 0.55 && whipsaw_rate < 0.35 {
            Self::Caution
        } else {
            Self::Avoid
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Trade => "TRADE",
            Self::Caution => "CAUTION",
            Self::Avoid => "AVOID",
        }
    }
}

/// Analyse détaillée par fenêtre d'entrée
/// Pour optimiser EntryMinutesBeforeEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryWindowAnalysis {
    /// Minutes avant événement (-60, -30, -15, -5, etc.)
    pub minutes_before: i32,
    
    /// Win rate pour ce timing
    pub win_rate: f64,
    
    /// Profit moyen en R (risk units)
    pub avg_profit_r: f64,
    
    /// Nombre de trades simulés
    pub sample_size: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_from_metrics() {
        // Trade
        assert_eq!(
            TradingRecommendation::from_metrics(8.5, 0.75, 0.15),
            TradingRecommendation::Trade
        );
        
        // Caution
        assert_eq!(
            TradingRecommendation::from_metrics(5.5, 0.60, 0.25),
            TradingRecommendation::Caution
        );
        
        // Avoid
        assert_eq!(
            TradingRecommendation::from_metrics(3.0, 0.50, 0.40),
            TradingRecommendation::Avoid
        );
    }
}
