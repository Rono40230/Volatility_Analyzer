use serde::{Deserialize, Serialize};

/// Peak delay analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakDelayResult {
    pub peak_delay_minutes: i16,
    pub peak_atr: f64,
    pub event_minute: u8,
    pub confidence: f64,
    pub event_count: usize,
    pub event_type: String,
    pub optimal_entry_seconds_before: i32,
    pub event_date_min: String, // ISO 8601: 1er événement analysé
    pub event_date_max: String, // ISO 8601: dernier événement analysé
}

/// Decay profile analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayProfileResult {
    pub peak_atr: f64,
    pub decay_rate_pips_per_minute: f64,
    pub decay_speed: String,
    pub recommended_timeout_minutes: i16,
    pub event_count: usize,
    pub event_type: String,
}

/// Available event types with count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeList {
    pub types: Vec<EventType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventType {
    pub name: String,
    pub count: i32,
}
