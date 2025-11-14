// commands/correlation/types.rs
// Structures de données communes pour les commandes de corrélation
// Conforme .clinerules: < 100 lignes, types seulement

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarEvent {
    pub id: i32,
    pub name: String,
    pub datetime: String,
    pub country: String,
    pub currency: String,
    pub impact: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PairImpact {
    pub symbol: String,
    pub event_volatility: f64,
    pub baseline_volatility: f64,
    pub multiplier: f64,
    pub direction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_data: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventImpactResponse {
    pub event: CalendarEvent,
    pub pair_impacts: Vec<PairImpact>,
    pub observations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventHistoryItem {
    pub event_id: i32,
    pub event_name: String,
    pub datetime: String,
    pub volatility: f64,
    pub baseline: f64,
    pub multiplier: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopEvent {
    pub name: String,
    pub volatility: f64,
    pub multiplier: f64,
    pub datetime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairEventHistoryResponse {
    pub pair_symbol: String,
    pub period: String,
    pub event_history: Vec<EventHistoryItem>,
    pub statistics: HistoryStatistics,
    pub top_events: Vec<TopEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryStatistics {
    pub avg_event_volatility: f64,
    pub avg_baseline_volatility: f64,
    pub avg_multiplier: f64,
    pub max_volatility: f64,
    pub total_events: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeatmapCell {
    pub event_type: String,
    pub pair: String,
    pub avg_volatility: f64,
    pub event_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatmapData {
    pub period: String,
    pub pairs: Vec<String>,
    pub event_types: Vec<EventTypeInfo>,
    pub data: HashMap<String, HashMap<String, f64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTypeInfo {
    pub name: String,
    pub count: i32,
}
