use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTypeInfo {
    pub name: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatmapData {
    pub period: String,
    pub pairs: Vec<String>,
    pub event_types: Vec<EventTypeInfo>,
    pub data: HashMap<String, HashMap<String, f64>>,
}

pub fn get_available_pairs(_conn: &Connection) -> Result<Vec<String>, String> {
    use crate::services::csv_loader::CsvLoader;
    let loader = CsvLoader::new();
    let symbols = loader
        .list_available_symbols()
        .map_err(|e| format!("Failed to list symbols: {}", e))?;
    Ok(symbols)
}

pub fn get_event_types(conn: &Connection, calendar_id: Option<i32>) -> Result<Vec<EventTypeInfo>, String> {
    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT description, COUNT(*) as count 
             FROM calendar_events 
             WHERE calendar_import_id = {} 
             GROUP BY description 
             HAVING count >= 1
             ORDER BY count DESC, description",
            cal_id
        )
    } else {
        "SELECT description, COUNT(*) as count 
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
) -> Result<f64, String> {
    use super::volatility_helpers::{calculate_volatilities_optimized, parse_sqlite_datetime};

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
        return Ok(0.0);
    }

    let mut total_volatility = 0.0;
    let mut valid_count = 0;

    for datetime_str in &events {
        let event_datetime = parse_sqlite_datetime(datetime_str)?;

        let metrics = calculate_volatilities_optimized(
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

    if valid_count == 0 {
        Ok(0.0)
    } else {
        Ok(total_volatility / valid_count as f64)
    }
}
