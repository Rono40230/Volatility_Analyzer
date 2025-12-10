use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Résultat du calcul de volatilité avec indicateur de disponibilité des données
#[derive(Debug, Clone)]
pub struct VolatilityResult {
    pub value: f64,
    pub has_data: bool,
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
}

pub fn get_event_types(
    conn: &Connection,
    calendar_id: Option<i32>,
) -> Result<Vec<EventTypeInfo>, String> {
    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT description, COUNT(DISTINCT event_time) as count 
             FROM calendar_events 
             WHERE calendar_import_id = {} 
             GROUP BY description 
             HAVING count >= 1
             ORDER BY count DESC, description",
            cal_id
        )
    } else {
        "SELECT description, COUNT(DISTINCT event_time) as count 
         FROM calendar_events 
         GROUP BY description 
         HAVING count >= 1
         ORDER BY count DESC, description"
            .to_string()
    };

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let event_types = stmt
        .query_map([], |row| {
            Ok(EventTypeInfo {
                name: row.get(0)?,
                count: row.get(1)?,
                has_data: None,
            })
        })
        .map_err(|e| format!("Failed to query event types: {}", e))?
        .collect::<SqliteResult<Vec<EventTypeInfo>>>()
        .map_err(|e| format!("Failed to collect event types: {}", e))?;

    Ok(event_types)
}

pub fn calculate_avg_volatility_for_event_pair_optimized(
    conn: &Connection,
    event_name: &str,
    pair: &str,
    calendar_id: Option<i32>,
    candle_index: &crate::services::candle_index::CandleIndex,
) -> Result<VolatilityResult, String> {
    use super::volatility_helpers::{calculer_volatilites_optimise, parse_sqlite_datetime};

    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT datetime(event_time) 
             FROM calendar_events 
             WHERE description = '{}' AND calendar_import_id = {} 
             ORDER BY event_time",
            event_name.replace("'", "''"),
            cal_id
        )
    } else {
        format!(
            "SELECT datetime(event_time) 
             FROM calendar_events 
             WHERE description = '{}' 
             ORDER BY event_time",
            event_name.replace("'", "''")
        )
    };

    let mut event_stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare event statement: {}", e))?;

    let events: Vec<String> = event_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query events: {}", e))?
        .collect::<SqliteResult<Vec<String>>>()
        .map_err(|e| format!("Failed to collect events: {}", e))?;

    if events.is_empty() {
        return Ok(VolatilityResult {
            value: 0.0,
            has_data: false,
        });
    }

    let mut total_volatility = 0.0;
    let mut valid_count = 0;
    let mut has_data_found = false;

    for datetime_str in &events {
        let event_datetime = parse_sqlite_datetime(datetime_str)?;

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
            30,
            7,
            super::volatility_helpers::get_pip_value(pair),
        )
        .unwrap_or(super::volatility_helpers::VolatilityMetrics {
            event_volatility: 0.0,
            baseline_volatility: 0.0,
        });

        let event_volatility = metrics.event_volatility;

        if event_volatility > 0.0 {
            total_volatility += event_volatility;
            valid_count += 1;
        }
    }

    let avg_vol = if valid_count == 0 {
        0.0
    } else {
        total_volatility / valid_count as f64
    };

    Ok(VolatilityResult {
        value: avg_vol,
        has_data: has_data_found,
    })
}
