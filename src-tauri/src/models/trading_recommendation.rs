// models/trading_recommendation.rs - Enums et logique pour recommandations de trading
//
// DEPRECATED (Phase 2): Remplacé par EntryAnalysisResult qui utilise le profit net réel
// après spread au lieu de scores heuristiques. Conservé temporairement pour compatibilité.
// Sera supprimé en Phase 3.
use serde::{Deserialize, Serialize};

/// Recommandation de trading pour stratégie STRADDLE (News Trading)
/// DEPRECATED: utiliser EntryAnalysisResult (Phase 2) à la place
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingRecommendation {
    /// Setup idéal - Offset standard, forte probabilité de breakout
    StraddleOptimal,
    /// Setup correct - Offset légèrement élargi recommandé
    StraddleGood,
    /// Setup acceptable - Offset large, surveillance accrue
    StraddleCautious,
    /// Setup médiocre - Envisager de passer l'événement
    StraddleRisky,
    /// Ne pas trader - Conditions inadaptées au Straddle
    NoTrade,
}

impl TradingRecommendation {
    pub fn from_confidence(score: f64) -> Self {
        if score >= 80.0 {
            Self::StraddleOptimal
        } else if score >= 65.0 {
            Self::StraddleGood
        } else if score >= 50.0 {
            Self::StraddleCautious
        } else if score >= 35.0 {
            Self::StraddleRisky
        } else {
            Self::NoTrade
        }
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::StraddleOptimal => "✅ SETUP OPTIMAL",
            Self::StraddleGood => "🟢 SETUP CORRECT",
            Self::StraddleCautious => "🟡 SETUP ACCEPTABLE",
            Self::StraddleRisky => "🟠 SETUP RISQUÉ",
            Self::NoTrade => "❌ NE PAS TRADER",
        }
    }

    /// Valide et ajuste la recommandation pour cohérence avec le risque
    pub fn validate_with_risk(self, risk: &RiskLevel) -> Self {
        match (&self, risk) {
            // ✅ COHÉRENT - pas d'ajustement
            (TradingRecommendation::StraddleOptimal, RiskLevel::Low) => self,
            (TradingRecommendation::StraddleOptimal, RiskLevel::Medium) => self,
            (TradingRecommendation::StraddleGood, _) => self,
            (TradingRecommendation::StraddleCautious, RiskLevel::Medium) => self,
            (TradingRecommendation::StraddleCautious, RiskLevel::High) => self,
            (TradingRecommendation::StraddleRisky, RiskLevel::Medium) => self,
            (TradingRecommendation::StraddleRisky, RiskLevel::High) => self,
            (TradingRecommendation::NoTrade, _) => self,

            // ❌ INCOHÉRENT - ajuste Recommendation
            (TradingRecommendation::StraddleOptimal, RiskLevel::High) => {
                tracing::warn!("Cohérence : StraddleOptimal + High Risk → ajuste à StraddleGood");
                TradingRecommendation::StraddleGood
            }
            (TradingRecommendation::StraddleCautious, RiskLevel::Low) => {
                tracing::warn!("Cohérence : StraddleCautious + Low Risk → ajuste à StraddleGood");
                TradingRecommendation::StraddleGood
            }
            (TradingRecommendation::StraddleRisky, RiskLevel::Low) => {
                tracing::warn!("Cohérence : StraddleRisky + Low Risk → ajuste à StraddleCautious");
                TradingRecommendation::StraddleCautious
            }
        }
    }
}

/// Qualité du mouvement pour stratégie STRADDLE
/// (Basé sur volatilité ET bruit - un mouvement erratique est risqué pour le Straddle)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    /// Mouvement directionnel et propre - Idéal pour Straddle
    Low,
    /// Mouvement modéré avec du bruit acceptable
    Medium,
    /// Mouvement erratique ou trop de faux breakouts - Risqué
    High,
}

impl RiskLevel {
    pub fn from_volatility(volatility: f64) -> Self {
        // Pour le Straddle, on veut de la volatilité mais pas trop de chaos
        // Low = mouvement directionnel fort
        // Medium = volatilité normale
        // High = trop erratique (ou trop calme, pas de mouvement)
        if volatility < 0.05 {
            Self::High // Trop calme, pas de breakout
        } else if volatility < 0.15 {
            Self::Medium
        } else if volatility < 0.30 {
            Self::Low // Sweet spot : volatilité forte mais contrôlée
        } else {
            Self::High // Trop chaotique
        }
    }
}
