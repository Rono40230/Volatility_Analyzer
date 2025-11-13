// event_impact/mod.rs - Point d'entrée du module event_impact
// Exporte la commande Tauri et les types publics

mod calculator;
mod helpers;
mod types;

pub use calculator::{calculate_pair_impacts, generate_observations};
pub use helpers::{currency_to_country, get_available_pairs};
pub use types::EventImpactResult;

use super::volatility_helpers::parse_sqlite_datetime;
use crate::commands::candle_index_commands::CandleIndexState;
use crate::commands::pair_data::PairDataState;
use crate::services::DatabaseLoader;
use chrono::{DateTime, Duration, TimeZone, Utc};
use rusqlite::Connection;
use tauri::State;

#[tauri::command]
pub async fn get_event_impact_by_pair(
    event_type: String,
    event_count: i32,
    _state: State<'_, CandleIndexState>,
    pair_state: State<'_, PairDataState>,
) -> Result<EventImpactResult, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");

    let db_path = data_dir.join("volatility.db");

    if !db_path.exists() {
        return Err("Database not found".to_string());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    // Récupérer TOUTES les occurrences de cet événement depuis 2024-01-01
    let mut event_stmt = conn
        .prepare("SELECT id, datetime(event_time), symbol FROM calendar_events WHERE description = ?1 AND event_time >= '2024-01-01' ORDER BY event_time")
        .map_err(|e| format!("Failed to prepare event query: {}", e))?;

    let events: Vec<(i32, String, String)> = event_stmt
        .query_map([&event_type], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Failed to query events: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect events: {}", e))?;

    if events.is_empty() {
        return Err(format!("No events found for type: {}", event_type));
    }

    // Récupérer info de la première occurrence
    let (first_id, first_datetime, currency) = &events[0];
    let country = currency_to_country(currency);

    let first_event_datetime = parse_sqlite_datetime(first_datetime)?;
    let window_start = first_event_datetime.format("%Y-%m-%d %H:%M").to_string();
    let window_end = (first_event_datetime + Duration::minutes(120))
        .format("%Y-%m-%d %H:%M")
        .to_string();

    let (_, last_datetime, _) = &events[events.len() - 1];
    let _last_event_datetime = parse_sqlite_datetime(last_datetime)?;
    let last_datetime_formatted = last_datetime.clone();

    // Créer le DatabaseLoader depuis le pair_data pool
    let pair_pool = pair_state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pair pool: {}", e))?
        .clone()
        .ok_or("Pair database pool not initialized")?;

    let db_loader = DatabaseLoader::new(pair_pool);

    // Créer un CandleIndex qui utilise le DatabaseLoader pour charger les paires
    let mut candle_index =
        crate::services::candle_index::CandleIndex::with_db_loader(db_loader.clone());

    // Obtenir toutes les paires disponibles depuis la BD
    let pairs = get_available_pairs(&db_loader)?;

    // Préparer les datetimes des événements
    let event_datetimes: Result<Vec<DateTime<Utc>>, String> = events
        .iter()
        .map(|(_, datetime_str, _)| {
            let naive_dt = parse_sqlite_datetime(datetime_str)?;
            Ok(Utc.from_utc_datetime(&naive_dt))
        })
        .collect();
    let event_datetimes = event_datetimes?;

    // Charger les paires à la demande
    for pair in &pairs {
        candle_index.load_pair_candles(pair)?;
    }

    let pair_impacts = calculate_pair_impacts(&pairs, &event_datetimes[0], &candle_index)?;
    let observations = generate_observations(&pair_impacts);

    Ok(EventImpactResult {
        event_id: *first_id,
        event_name: event_type.clone(),
        datetime: first_datetime.clone(),
        last_datetime: last_datetime_formatted,
        country,
        currency: currency.clone(),
        event_count,
        window_start,
        window_end,
        pair_impacts,
        observations,
    })
}
