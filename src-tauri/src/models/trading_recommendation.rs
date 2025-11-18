// models/trading_recommendation.rs - Enums et logique pour recommandations de trading
use serde::{Deserialize, Serialize};

/// Recommandation de trading basée sur le score de confiance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingRecommendation {
    /// Scalper agressivement
    ScalpAggressive,
    /// Scalper normalement
    ScalpNormal,
    /// Scalper prudemment
    ScalpCautious,
    /// Très prudent / breakouts only
    VeryCautious,
    /// Ne pas trader
    NoTrade,
}

impl TradingRecommendation {
    pub fn from_confidence(score: f64) -> Self {
        if score >= 80.0 {
            Self::ScalpAggressive
        } else if score >= 65.0 {
            Self::ScalpNormal
        } else if score >= 50.0 {
            Self::ScalpCautious
        } else if score >= 35.0 {
            Self::VeryCautious
        } else {
            Self::NoTrade
        }
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::ScalpAggressive => "✅ SCALPER AGRESSIF",
            Self::ScalpNormal => "🟢 SCALPER NORMAL",
            Self::ScalpCautious => "🟡 SCALPER PRUDENT",
            Self::VeryCautious => "🟠 TRÈS PRUDENT",
            Self::NoTrade => "❌ NE PAS TRADER",
        }
    }

    /// Valide et ajuste la recommandation pour cohérence avec le risque
    pub fn validate_with_risk(self, risk: &RiskLevel) -> Self {
        match (&self, risk) {
            // ✅ COHÉRENT - pas d'ajustement
            (TradingRecommendation::ScalpAggressive, RiskLevel::Low) => self,
            (TradingRecommendation::ScalpAggressive, RiskLevel::Medium) => self,
            (TradingRecommendation::ScalpNormal, _) => self,
            (TradingRecommendation::ScalpCautious, RiskLevel::Medium) => self,
            (TradingRecommendation::ScalpCautious, RiskLevel::High) => self,
            (TradingRecommendation::VeryCautious, RiskLevel::Medium) => self,
            (TradingRecommendation::VeryCautious, RiskLevel::High) => self,
            (TradingRecommendation::NoTrade, _) => self,

            // ❌ INCOHÉRENT - ajuste Recommendation
            (TradingRecommendation::ScalpAggressive, RiskLevel::High) => {
                tracing::warn!("Cohérence : ScalpAggressive + High Risk → ajuste à ScalpNormal");
                TradingRecommendation::ScalpNormal
            }
            (TradingRecommendation::ScalpCautious, RiskLevel::Low) => {
                tracing::warn!("Cohérence : ScalpCautious + Low Risk → ajuste à ScalpNormal");
                TradingRecommendation::ScalpNormal
            }
            (TradingRecommendation::VeryCautious, RiskLevel::Low) => {
                tracing::warn!("Cohérence : VeryCautious + Low Risk → ajuste à ScalpCautious");
                TradingRecommendation::ScalpCautious
            }
        }
    }
}

/// Niveau de risque basé sur la volatilité
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn from_volatility(volatility: f64) -> Self {
        if volatility < 0.05 {
            Self::Low
        } else if volatility < 0.15 {
            Self::Medium
        } else {
            Self::High
        }
    }
}
