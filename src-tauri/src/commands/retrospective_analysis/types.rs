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

/// Detailed decay profile with ATR timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayProfileDetailedResult {
    pub peak_atr: f64,
    pub decay_rate_pips_per_minute: f64,
    pub decay_speed: String,
    pub recommended_timeout_minutes: i16,
    pub event_count: usize,
    pub event_type: String,
    pub atr_timeline: Vec<f64>,      // ATR par minute (tous les 1min)
    pub volatility_mean: Vec<f64>,   // Moyenne de volatilité par minute
    pub volatility_std: Vec<f64>,    // Écart-type par minute
    pub peak_minute: u16,            // Minute où ATR = max
    pub total_minutes_analyzed: u16, // Durée totale en minutes
}

/// Volatility profile for retrospective analysis (T-30 to T+90)
/// Y-axis: ATR5 (volatilité réelle mesurée sur fenêtre 5-min glissante)
/// X-axis: Temps en minutes depuis événement (-30 = T-30, 0 = T0, 90 = T+90)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityProfileResult {
    pub atr5_timeline: Vec<f64>, // ATR5 glissant (121 points = T-30 à T+90)
    pub peak_minute: u16,        // Minute où ATR5 atteint le max (0 = T0, 30 = T-30, 120 = T+90)
    pub peak_atr5: f64,          // Valeur max d'ATR5
    pub mean_atr5: f64,          // Moyenne d'ATR5 sur la période
    pub std_atr5: f64,           // Écart-type d'ATR5
    pub event_count: usize,      // Nombre d'occurrences analysées
    pub event_type: String,
    pub pair: String,
}

/// Event impact analysis: volatility comparison before/after event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventImpactResult {
    pub atr_timeline_before: Vec<f64>, // ATR moyen T-30 à T0 (30 points)
    pub atr_timeline_after: Vec<f64>,  // ATR moyen T0 à T+90 (90 points)
    pub body_timeline_before: Vec<f64>, // Body% moyen T-30 à T0 (30 points)
    pub body_timeline_after: Vec<f64>, // Body% moyen T0 à T+90 (90 points)
    pub noise_ratio_before: f64,       // Noise Ratio moyen avant événement
    pub noise_ratio_during: f64,       // Noise Ratio au moment de l'événement
    pub noise_ratio_after: f64,        // Noise Ratio moyen après événement
    pub volatility_increase_percent: f64, // % d'augmentation ATR (après vs avant)
    pub event_count: usize,            // Nombre d'occurrences analysées
    pub event_type: String,
    pub pair: String,
    pub event_datetime: String,  // ISO 8601: heure moyenne de l'événement
    pub timezone_offset: String, // Ex: "UTC+0" ou "UTC-5"

    // === PARAMÈTRES BIDI POUR STRADDLE (DIRECTIONNEL) ===
    pub meilleur_moment: f64, // Offset optimal en minutes avant événement (T0 - meilleur_moment)
    pub stop_loss: f64,       // Stop Loss en pips (basé sur ATR moyen)
    pub trailing_stop: f64,   // Trailing Stop coefficient (ajusté selon noise)
    pub timeout: i32,         // Timeout recommandé en minutes (basé sur decay de volatilité)
    pub offset: f64,          // Offset d'entrée en points (distance du prix)
    
    // === PARAMÈTRES BIDI POUR STRADDLE (SIMULTANÉ) ===
    pub stop_loss_simultaneous: f64,       // SL spécifique pour mode Simultané (souvent plus large)
    pub trailing_stop_simultaneous: f64,   // TS spécifique pour mode Simultané
    pub offset_simultaneous: f64,          // Offset spécifique pour mode Simultané
    pub stop_loss_recovery_simultaneous: f64, // SL Recovery spécifique pour mode Simultané
    pub stop_loss_recovery: f64,           // SL Recovery (Directionnel)

    pub point_value: f64,     // Valeur d'un point pour normalisation (ex: 0.001 pour JPY)
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
