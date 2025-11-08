// models/analysis_result.rs - Résultat d'analyse complète
use serde::{Deserialize, Serialize};
use super::HourlyStats;
use super::calendar_event::CalendarEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub symbol: String,
    pub period_start: String,        // Date début analyse
    pub period_end: String,          // Date fin analyse
    pub timeframe: String,           // Ex: "M1", "M5", etc.
    pub hourly_stats: Vec<HourlyStats>,
    pub best_hours: Vec<u8>,
    pub confidence_score: f64,
    pub recommendation: TradingRecommendation,
    pub risk_level: RiskLevel,
    pub global_metrics: GlobalMetrics,
    pub correlated_events: Vec<CorrelatedEvent>,
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
        if score >= 80.0 { Self::ScalpAggressive }
        else if score >= 65.0 { Self::ScalpNormal }
        else if score >= 50.0 { Self::ScalpCautious }
        else if score >= 35.0 { Self::VeryCautious }
        else { Self::NoTrade }
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
        if volatility < 0.05 { Self::Low }
        else if volatility < 0.15 { Self::Medium }
        else { Self::High }
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
        assert_eq!(TradingRecommendation::from_confidence(85.0), TradingRecommendation::ScalpAggressive);
        assert_eq!(TradingRecommendation::from_confidence(70.0), TradingRecommendation::ScalpNormal);
        assert_eq!(TradingRecommendation::from_confidence(50.0), TradingRecommendation::ScalpCautious);
        assert_eq!(TradingRecommendation::from_confidence(30.0), TradingRecommendation::VeryCautious);
        assert_eq!(TradingRecommendation::from_confidence(10.0), TradingRecommendation::NoTrade);
    }

    #[test]
    fn test_risk_level_from_volatility() {
        assert_eq!(RiskLevel::from_volatility(0.20), RiskLevel::High);
        assert_eq!(RiskLevel::from_volatility(0.10), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_volatility(0.03), RiskLevel::Low);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedEvent {
    pub event: CalendarEvent,
    pub volatility_hour: u8,
    pub volatility_increase: f64,
    pub correlation_score: f64,
}
