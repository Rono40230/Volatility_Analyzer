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
    pub point_value: f64, // Valeur d'un point pour normalisation (ex: 0.001 pour JPY)
    pub unit: String,     // Unité d'affichage (pips, points, $)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    pub mean_atr: f64,
    pub mean_max_true_range: f64,
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
