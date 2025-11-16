// models/analysis_result.rs - Résultat d'analyse complète
use super::calendar_event::CalendarEvent;
use super::{HourlyStats, Stats15Min};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub symbol: String,
    pub period_start: String, // Date début analyse
    pub period_end: String,   // Date fin analyse
    pub timeframe: String,    // Ex: "M1", "M5", etc.
    pub hourly_stats: Vec<HourlyStats>,
    pub stats_15min: Vec<Stats15Min>, // Nouvelles stats pour scalping
    pub best_hours: Vec<u8>,
    pub confidence_score: f64,
    pub recommendation: TradingRecommendation,
    pub risk_level: RiskLevel,
    pub global_metrics: GlobalMetrics,
}

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
    ///
    /// MATRICE DE COHÉRENCE :
    ///
    /// Recommandation × RiskLevel doivent être COHÉRENTS.
    /// Une recommandation agressive avec risque élevé = contradiction.
    /// Une recommandation prudente avec risque bas = trop conservative.
    ///
    /// RÈGLES :
    /// 1. ScalpAggressive (80+) : acceptable si Risk = Low/Medium SEULEMENT
    ///    - Si Risk = High → ajuste à ScalpNormal
    ///
    /// 2. ScalpNormal (65-80) : TOUJOURS acceptable (flexible)
    ///
    /// 3. ScalpCautious (50-65) : acceptable si Risk = Medium/High
    ///    - Si Risk = Low → ajuste à ScalpNormal (pas assez prudent)
    ///
    /// 4. VeryCautious (35-50) : acceptable si Risk = Medium/High
    ///    - Si Risk = Low → ajuste à ScalpCautious (contradiction)
    ///
    /// 5. NoTrade (0-35) : TOUJOURS valide, Risk ignoré
    ///
    /// EXEMPLE :
    /// - ScalpAggressive + Low Risk = ✅ OK (agressif quand c'est sûr)
    /// - ScalpAggressive + High Risk = ❌ CONTRADICTION → ajuste à ScalpNormal
    /// - ScalpNormal + Any Risk = ✅ OK (flexible par design)
    /// - VeryCautious + Low Risk = ❌ CONTRADICTION → ajuste à ScalpCautious
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
                // Agressif + Risque élevé = contradiction
                // → Remonte à ScalpNormal (plus prudent)
                tracing::warn!("Cohérence : ScalpAggressive + High Risk → ajuste à ScalpNormal");
                TradingRecommendation::ScalpNormal
            }
            (TradingRecommendation::ScalpCautious, RiskLevel::Low) => {
                // Prudent + Risque bas = trop conservative
                // → Remonte à ScalpNormal (plus équilibré)
                tracing::warn!("Cohérence : ScalpCautious + Low Risk → ajuste à ScalpNormal");
                TradingRecommendation::ScalpNormal
            }
            (TradingRecommendation::VeryCautious, RiskLevel::Low) => {
                // Très prudent + Risque bas = contradiction
                // → Remonte à ScalpCautious (toujours prudent mais moins)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    pub mean_atr: f64,
    pub mean_volatility: f64,
    pub mean_body_range: f64,
    pub mean_tick_quality: f64,
    pub mean_noise_ratio: f64,
    pub mean_volume_imbalance: f64,
    pub mean_breakout_percentage: f64,
    pub total_candles: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_recommendation_from_confidence() {
        assert_eq!(
            TradingRecommendation::from_confidence(85.0),
            TradingRecommendation::ScalpAggressive
        );
        assert_eq!(
            TradingRecommendation::from_confidence(70.0),
            TradingRecommendation::ScalpNormal
        );
        assert_eq!(
            TradingRecommendation::from_confidence(50.0),
            TradingRecommendation::ScalpCautious
        );
        assert_eq!(
            TradingRecommendation::from_confidence(30.0),
            TradingRecommendation::VeryCautious
        );
        assert_eq!(
            TradingRecommendation::from_confidence(10.0),
            TradingRecommendation::NoTrade
        );
    }

    #[test]
    fn test_risk_level_from_volatility() {
        assert_eq!(RiskLevel::from_volatility(0.20), RiskLevel::High);
        assert_eq!(RiskLevel::from_volatility(0.10), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_volatility(0.03), RiskLevel::Low);
    }
}

// ============================================================================
// UNIT TESTS FOR COHERENCE VALIDATION (Incohérence #6)
// ============================================================================
//
// These tests validate the coherence rules matrix between TradingRecommendation
// and RiskLevel. The 9 coherence rules are:
//
// VALID COMBINATIONS (6 cases - no adjustment):
// - ScalpAggressive + Low Risk ✓
// - ScalpAggressive + Medium Risk ✓
// - ScalpNormal + Low Risk ✓
// - ScalpNormal + Medium Risk ✓
// - ScalpNormal + High Risk ✓
// - ScalpCautious + Medium Risk ✓
// - ScalpCautious + High Risk ✓
// - VeryCautious + Medium Risk ✓
// - VeryCautious + High Risk ✓
//
// INVALID COMBINATIONS (3 cases - with adjustment):
// 1. ScalpAggressive + High Risk → adjust to ScalpNormal
// 2. ScalpCautious + Low Risk → adjust to ScalpNormal
// 3. VeryCautious + Low Risk → adjust to ScalpCautious
//
// See validate_with_risk() documentation for complete logic.
// ============================================================================

#[cfg(test)]
mod coherence_validation_tests {
    use super::*;

    #[test]
    fn test_scalp_aggressive_low_risk_coherent() {
        // Valid combination: aggressive trader, low volatility = good match
        let rec = TradingRecommendation::ScalpAggressive;
        let adjusted = rec.validate_with_risk(&RiskLevel::Low);
        assert_eq!(adjusted, TradingRecommendation::ScalpAggressive);
    }

    #[test]
    fn test_scalp_aggressive_medium_risk_coherent() {
        // Valid combination: aggressive trader, medium volatility = good match
        let rec = TradingRecommendation::ScalpAggressive;
        let adjusted = rec.validate_with_risk(&RiskLevel::Medium);
        assert_eq!(adjusted, TradingRecommendation::ScalpAggressive);
    }

    #[test]
    fn test_scalp_aggressive_high_risk_adjusted() {
        // INVALID: aggressive strategy in high volatility is too risky
        // Should adjust down to normal trading intensity
        let rec = TradingRecommendation::ScalpAggressive;
        let adjusted = rec.validate_with_risk(&RiskLevel::High);
        assert_eq!(adjusted, TradingRecommendation::ScalpNormal);
    }

    #[test]
    fn test_scalp_normal_all_risks_coherent() {
        // Valid for all risks: normal strategy is flexible
        let rec = TradingRecommendation::ScalpNormal;

        assert_eq!(
            rec.clone().validate_with_risk(&RiskLevel::Low),
            TradingRecommendation::ScalpNormal
        );
        assert_eq!(
            rec.clone().validate_with_risk(&RiskLevel::Medium),
            TradingRecommendation::ScalpNormal
        );
        assert_eq!(
            rec.validate_with_risk(&RiskLevel::High),
            TradingRecommendation::ScalpNormal
        );
    }

    #[test]
    fn test_scalp_cautious_medium_risk_coherent() {
        // Valid: cautious strategy with medium volatility = balanced
        let rec = TradingRecommendation::ScalpCautious;
        let adjusted = rec.validate_with_risk(&RiskLevel::Medium);
        assert_eq!(adjusted, TradingRecommendation::ScalpCautious);
    }

    #[test]
    fn test_scalp_cautious_high_risk_coherent() {
        // Valid: cautious strategy in high volatility = good match
        let rec = TradingRecommendation::ScalpCautious;
        let adjusted = rec.validate_with_risk(&RiskLevel::High);
        assert_eq!(adjusted, TradingRecommendation::ScalpCautious);
    }

    #[test]
    fn test_scalp_cautious_low_risk_adjusted() {
        // INVALID: cautious strategy in low risk = wasting opportunity
        // Should adjust to normal strategy to take advantage of low volatility
        let rec = TradingRecommendation::ScalpCautious;
        let adjusted = rec.validate_with_risk(&RiskLevel::Low);
        assert_eq!(adjusted, TradingRecommendation::ScalpNormal);
    }

    #[test]
    fn test_very_cautious_medium_risk_coherent() {
        // Valid: very cautious strategy with medium volatility = safe
        let rec = TradingRecommendation::VeryCautious;
        let adjusted = rec.validate_with_risk(&RiskLevel::Medium);
        assert_eq!(adjusted, TradingRecommendation::VeryCautious);
    }

    #[test]
    fn test_very_cautious_high_risk_coherent() {
        // Valid: very cautious strategy in high volatility = most defensive
        let rec = TradingRecommendation::VeryCautious;
        let adjusted = rec.validate_with_risk(&RiskLevel::High);
        assert_eq!(adjusted, TradingRecommendation::VeryCautious);
    }

    #[test]
    fn test_very_cautious_low_risk_adjusted() {
        // INVALID: very cautious in low risk = too defensive
        // Should adjust to cautious strategy to improve returns while staying safe
        let rec = TradingRecommendation::VeryCautious;
        let adjusted = rec.validate_with_risk(&RiskLevel::Low);
        assert_eq!(adjusted, TradingRecommendation::ScalpCautious);
    }

    #[test]
    fn test_no_trade_all_risks_coherent() {
        // Valid for all risks: no trade recommendation is always valid
        let rec = TradingRecommendation::NoTrade;

        assert_eq!(
            rec.clone().validate_with_risk(&RiskLevel::Low),
            TradingRecommendation::NoTrade
        );
        assert_eq!(
            rec.clone().validate_with_risk(&RiskLevel::Medium),
            TradingRecommendation::NoTrade
        );
        assert_eq!(
            rec.validate_with_risk(&RiskLevel::High),
            TradingRecommendation::NoTrade
        );
    }

    #[test]
    fn test_coherence_all_9_rules_exhaustive() {
        // Exhaustive test covering all 9 coherence rules in one place
        // This serves as a reference and safeguard against regressions

        let mut valid_count = 0;
        let mut adjusted_count = 0;

        let recommendations = [
            TradingRecommendation::ScalpAggressive,
            TradingRecommendation::ScalpNormal,
            TradingRecommendation::ScalpCautious,
            TradingRecommendation::VeryCautious,
            TradingRecommendation::NoTrade,
        ];

        let risks = [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High];

        for rec in &recommendations {
            for risk in &risks {
                let adjusted = rec.clone().validate_with_risk(risk);
                if adjusted == *rec {
                    valid_count += 1;
                } else {
                    adjusted_count += 1;
                }
            }
        }

        // Should have exactly 9 valid combinations and 3 adjusted combinations
        // across 5 recommendations × 3 risks = 15 total cases
        assert_eq!(valid_count + adjusted_count, 15);
        assert_eq!(valid_count, 12); // 12 valid cases (including NoTrade×3 and others)
        assert_eq!(adjusted_count, 3); // 3 adjustment cases
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedEvent {
    pub event: CalendarEvent,
    pub volatility_hour: u8,
    pub volatility_increase: f64,
    pub correlation_score: f64,
}
