use rusqlite::Connection;
use tauri::State;

use super::heatmap_helpers::{
    calculate_avg_volatility_for_event_pair_optimized, get_event_types, HeatmapData,
};
use crate::commands::candle_index_commands::CandleIndexState;

#[tauri::command]
pub async fn get_correlation_heatmap(
    calendar_id: Option<i32>,
    pairs: Vec<String>,
    state: State<'_, CandleIndexState>,
) -> Result<HeatmapData, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");

    let db_path = data_dir.join("volatility.db");

    if !db_path.exists() {
        return Err("Database not found".to_string());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    if pairs.is_empty() {
        return Err("No pairs provided".to_string());
    }

    let mut event_types = get_event_types(&conn, calendar_id)?;

    if event_types.is_empty() {
        return Ok(HeatmapData {
            period: "Calendrier sélectionné".to_string(),
            pairs,
            event_types: vec![],
            data: std::collections::HashMap::new(),
        });
    }

    let mut data: std::collections::HashMap<String, std::collections::HashMap<String, f64>> =
        std::collections::HashMap::new();

    let mut index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock candle index state: {}", e))?;

    let candle_index = index_state
        .as_mut()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;

    for pair in &pairs {
        candle_index.load_pair_candles(pair)?;
    }

    for pair in &pairs {
        for event_type in &mut event_types {
            let vol_result = calculate_avg_volatility_for_event_pair_optimized(
                &conn,
                &event_type.name,
                pair,
                calendar_id,
                candle_index,
            )?;

            let avg_vol_rounded = (vol_result.value * 10.0).round() / 10.0;

            // Marquer has_data au moins une fois si vrai
            if vol_result.has_data {
                event_type.has_data = Some(true);
            }

            data.entry(event_type.name.clone())
                .or_default()
                .insert(pair.clone(), avg_vol_rounded);
        }
    }

    Ok(HeatmapData {
        period: "Calendrier sélectionné".to_string(),
        pairs,
        event_types,
        data,
    })
}
