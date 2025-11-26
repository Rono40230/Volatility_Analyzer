// models/analysis_result.rs - Résultat d'analyse complète
use super::calendar_event::CalendarEvent;
use super::{HourlyStats, Stats15Min};
use crate::models::trading_recommendation::{RiskLevel, TradingRecommendation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub symbol: String,
    pub period_start: String,
    pub period_end: String,
    pub timeframe: String,
    pub hourly_stats: Vec<HourlyStats>,
    pub stats_15min: Vec<Stats15Min>,
    pub best_quarter: (u8, u8), // (hour, quarter) - meilleur quarter de la journée
    pub confidence_score: f64,
    pub recommendation: TradingRecommendation,
    pub risk_level: RiskLevel,
    pub global_metrics: GlobalMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    pub mean_atr: f64,
    pub mean_volatility: f64,
    pub mean_body_range: f64,
    pub mean_noise_ratio: f64,
    pub mean_volume_imbalance: f64,
    pub mean_breakout_percentage: f64,
    pub mean_range: f64,
    pub total_candles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedEvent {
    pub event: CalendarEvent,
    pub volatility_hour: u8,
    pub volatility_increase: f64,
    pub correlation_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_result_creation() {
        let result = AnalysisResult {
            symbol: "EURUSD".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            timeframe: "M5".to_string(),
            hourly_stats: vec![],
            stats_15min: vec![],
            best_quarter: (14, 2), // 14:30-14:45
            confidence_score: 75.0,
            recommendation: TradingRecommendation::ScalpNormal,
            risk_level: RiskLevel::Medium,
            global_metrics: GlobalMetrics {
                mean_atr: 0.0045,
                mean_volatility: 0.12,
                mean_body_range: 0.0020,
                mean_tick_quality: 0.85,
                mean_noise_ratio: 2.1,
                mean_volume_imbalance: 0.15,
                mean_breakout_percentage: 0.35,
                total_candles: 15000,
            },
        };

        assert_eq!(result.symbol, "EURUSD");
        assert_eq!(result.best_quarter, (14, 2));
    }
}
