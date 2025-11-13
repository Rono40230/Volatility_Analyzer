// event_impact/types.rs - Types pour analyse d'impact événements
// Conforme .clinerules: <100L, structs sérialisables

use serde::{Deserialize, Serialize};

/// Détail d'impact d'un événement sur une paire
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PairImpactDetail {
    pub symbol: String,
    pub event_volatility: f64,                 // valeur brute en pips
    pub baseline_volatility: f64,              // valeur brute en pips
    pub event_volatility_formatted: String,    // formatée à 1 décimale
    pub baseline_volatility_formatted: String, // formatée à 1 décimale
    pub points: f64,                           // event_volatility / 10 (1 point = 1/10 pip)
    pub points_formatted: String,              // formatée à 1 décimale
    pub price: f64,                            // points * pip_value (valeur monétaire approx)
    pub price_formatted: String,               // formatée à 2 décimales
    pub multiplier: f64,
    pub direction: String,
}

/// Résultat complet de l'analyse d'impact
#[derive(Debug, Serialize, Deserialize)]
pub struct EventImpactResult {
    pub event_id: i32,
    pub event_name: String,
    pub datetime: String,      // Première date
    pub last_datetime: String, // Dernière date
    pub country: String,
    pub currency: String,
    pub event_count: i32, // Nombre d'occurrences de cet événement
    pub window_start: String,
    pub window_end: String,
    pub pair_impacts: Vec<PairImpactDetail>,
    pub observations: Vec<String>,
}
