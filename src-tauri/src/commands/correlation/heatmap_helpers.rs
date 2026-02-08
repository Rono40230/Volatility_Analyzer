use chrono::NaiveDateTime;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Résultat du calcul de volatilité avec indicateur de disponibilité des données
#[derive(Debug, Clone)]
pub struct VolatilityResult {
    pub value: f64,
    pub has_data: bool,
    /// Nombre d'occurrences (événements avec données) utilisées pour le calcul
    pub sample_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTypeInfo {
    pub name: String,
    pub count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_data: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatmapData {
    pub period: String,
    pub period_start: String,
    pub period_end: String,
    pub pairs: Vec<String>,
    pub event_types: Vec<EventTypeInfo>,
    pub data: HashMap<String, HashMap<String, f64>>,
    /// Nombre d'occurrences par cellule (event_type → pair → count)
    pub counts: HashMap<String, HashMap<String, i32>>,
}

pub fn calculer_volatilite_moyenne_evenement_paire_optimise(
    _conn: &Connection,
    event_name: &str,
    pair: &str,
    _calendar_id: Option<i32>,
    candle_index: &crate::services::candle_index::CandleIndex,
    events_cache: Option<&HashMap<String, Vec<NaiveDateTime>>>,
) -> Result<VolatilityResult, String> {
    use super::volatility_helpers::calculer_volatilites_optimise;
    
    // Si on a un cache, on l'utilise, sinon ... on devrait éviter ce cas
    let empty_vec = Vec::new();
    let events = if let Some(cache) = events_cache {
        cache.get(event_name).unwrap_or(&empty_vec).clone()
    } else {
        // Fallback pour compatibilité (mais lent)
        return Err("Cache events manquant (Optimisation requise)".to_string());
    };

    if events.is_empty() {
        return Ok(VolatilityResult {
            value: 0.0,
            has_data: false,
            sample_count: 0,
        });
    }

    let mut total_score = 0.0;
    let mut valid_count = 0;
    let mut has_data_found = false;

    for event_datetime in events {
        // Vérifier si des candles existent pour cet événement
        // Si pas de candles, SKIPER complètement cet événement
        if !super::data_availability::has_candles_for_event(candle_index, pair, event_datetime) {
            continue;
        }
        
        has_data_found = true;

        let metrics = calculer_volatilites_optimise(
            candle_index,
            pair,
            event_datetime,
            90, // Élargi à 90 min pour capturer les candles H1 qui commencent avant
            7,
            super::utils::get_pip_value(pair),
        )
        .unwrap_or(super::volatility_helpers::VolatilityMetrics {
            event_volatility: 0.0,
            baseline_volatility: 0.0,
            straddle_score: 0.0,
            directionality: 0.0,
            whipsaw_risk: 0.0,
        });

        let score = metrics.straddle_score;

        if score > 0.0 {
            total_score += score;
            valid_count += 1;
        }
    }

    let avg_score = if valid_count == 0 {
        0.0
    } else {
        total_score / valid_count as f64
    };

    Ok(VolatilityResult {
        value: avg_score,
        has_data: has_data_found,
        sample_count: valid_count,
    })
}

