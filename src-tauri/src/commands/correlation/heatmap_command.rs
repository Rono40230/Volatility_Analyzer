use chrono::{Duration, Utc};
use rusqlite::Connection;
use tauri::State;

use super::heatmap_helpers::{calculer_volatilite_moyenne_evenement_paire_optimise, HeatmapData};
use super::heatmap_queries::{get_all_events_grouped, get_event_types};
use super::utils::{format_date_fr, parse_db_date};
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

    // Récupérer la plage de dates
    let range_query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT MIN(event_time), MAX(event_time) 
             FROM calendar_events 
             WHERE calendar_import_id = {} 
             AND impact IN ('H', 'M')",
            cal_id
        )
    } else {
        "SELECT MIN(event_time), MAX(event_time) 
         FROM calendar_events 
         WHERE impact IN ('H', 'M')"
            .to_string()
    };

    let mut range_stmt = conn
        .prepare(&range_query)
        .map_err(|e| format!("Failed to prepare range query: {}", e))?;
    let (start_str_opt, end_str_opt) = range_stmt
        .query_row([], |row| {
            let start: Option<String> = row.get(0)?;
            let end: Option<String> = row.get(1)?;
            Ok((start, end))
        })
        .unwrap_or((None, None));

    // Déterminer la plage de chargement pour l'index
    let mut load_start = Utc::now() - Duration::days(365 * 5);
    let mut load_end = Utc::now();

    if let Some(ref s) = start_str_opt {
        if let Some(dt) = parse_db_date(s) {
            load_start = dt;
        }
    }
    if let Some(ref s) = end_str_opt {
        if let Some(dt) = parse_db_date(s) {
            load_end = dt;
        }
    }

    let period_start = start_str_opt
        .as_ref()
        .map(|s| format_date_fr(s))
        .unwrap_or_else(|| "N/A".to_string());
    let period_end = end_str_opt
        .as_ref()
        .map(|s| format_date_fr(s))
        .unwrap_or_else(|| "N/A".to_string());

    let mut event_types = get_event_types(&conn, calendar_id)?;

    if event_types.is_empty() {
        return Ok(HeatmapData {
            period: "Calendrier sélectionné".to_string(),
            period_start,
            period_end,
            pairs,
            event_types: vec![],
            data: std::collections::HashMap::new(),
            counts: std::collections::HashMap::new(),
        });
    }

    let mut data: std::collections::HashMap<String, std::collections::HashMap<String, f64>> =
        std::collections::HashMap::new();
    let mut counts: std::collections::HashMap<String, std::collections::HashMap<String, i32>> =
        std::collections::HashMap::new();

    let mut index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock candle index state: {}", e))?;

    let candle_index = index_state
        .as_mut()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;

    // Optimisation : ajouter une marge de sécurité (ex: 5 jours avant/après)
    let buffer = Duration::days(5);
    let effective_start = load_start - buffer;
    let effective_end = load_end + buffer;

    for pair in &pairs {
        candle_index.load_pair_candles_in_range(pair, effective_start, effective_end)?;
    }

    // Précachage des événements pour éviter les requêtes DB répétitives
    let events_cache = get_all_events_grouped(&conn, calendar_id)?;

    for pair in &pairs {
        for event_type in &mut event_types {
            let vol_result = calculer_volatilite_moyenne_evenement_paire_optimise(
                &conn,
                &event_type.name,
                pair,
                calendar_id,
                candle_index,
                Some(&events_cache),
            )?;

            let avg_vol_rounded = if vol_result.has_data {
                (vol_result.value * 10.0).round() / 10.0
            } else {
                -1.0 // Indicateur de "Pas de données"
            };

            // Marquer has_data au moins une fois si vrai
            if vol_result.has_data {
                event_type.has_data = Some(true);
            }

            data.entry(event_type.name.clone())
                .or_default()
                .insert(pair.clone(), avg_vol_rounded);

            counts.entry(event_type.name.clone())
                .or_default()
                .insert(pair.clone(), vol_result.sample_count);
        }
    }

    Ok(HeatmapData {
        period: "Calendrier sélectionné".to_string(),
        period_start,
        period_end,
        pairs,
        event_types,
        data,
        counts,
    })
}
