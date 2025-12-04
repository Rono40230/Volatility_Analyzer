// commands/candle_index_commands.rs
// Gère le chargement et l'initialisation du CandleIndex global

use crate::commands::candle_helpers::*;
use crate::commands::pair_data::PairDataState;
use crate::services::candle_index::CandleIndex;
use crate::services::DatabaseLoader;
use std::sync::Mutex;
use tauri::State;
use tracing::info;

pub struct CandleIndexState {
    pub index: Mutex<Option<CandleIndex>>,
}

/// Initialise l'index des candles au démarrage (LAZY LOADING)
#[tauri::command]
pub async fn init_candle_index(
    state: State<'_, CandleIndexState>,
    pair_state: State<'_, PairDataState>,
) -> Result<String, String> {
    let pair_pool = pair_state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pair pool: {}", e))?
        .clone()
        .ok_or("Pair database pool not initialized")?;

    let db_loader = DatabaseLoader::new(pair_pool);
    let index = CandleIndex::with_db_loader(db_loader);

    *state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))? = Some(index);

    Ok("CandleIndex initialized avec DatabaseLoader - paires chargées depuis BD".to_string())
}

/// Charge une paire spécifique dans l'index
#[tauri::command]
pub async fn load_pair_candles(
    symbol: String,
    state: State<'_, CandleIndexState>,
) -> Result<String, String> {
    let mut index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let index = index_state
        .as_mut()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;

    match index.load_pair_candles(&symbol)? {
        true => Ok(format!("Paire {} chargée avec succès", symbol)),
        false => Ok(format!("Paire {} déjà en cache", symbol)),
    }
}

/// Retourne les stats de l'index
#[tauri::command]
pub async fn get_candle_index_stats(state: State<'_, CandleIndexState>) -> Result<String, String> {
    let index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    match &*index_state {
        Some(index) => {
            let stats = index.get_stats();
            let pairs: Vec<String> = index.get_available_pairs();
            Ok(format!("Pairs: {:?}, Stats: {:?}", pairs, stats))
        }
        None => Err("CandleIndex not initialized. Call init_candle_index first.".to_string()),
    }
}

/// Récupère TOUS les candles pour une paire (pour Phase 7 analyses)
#[tauri::command]
pub async fn get_pair_candles(
    symbol: String,
    state: State<'_, CandleIndexState>,
) -> Result<Vec<crate::models::Candle>, String> {
    let mut index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let index = index_state
        .as_mut()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;

    if !index.is_pair_loaded(&symbol) {
        index.load_pair_candles(&symbol)?;
    }

    index
        .get_all_candles(&symbol)
        .ok_or(format!("No candles found for {}", symbol))
}

/// Récupère les 60 candles pour une heure spécifique
#[tauri::command]
pub async fn get_candles_for_hour(
    symbol: String,
    date_str: String,
    hour: u32,
    state: State<'_, CandleIndexState>,
) -> Result<CandlesForHourResponse, String> {
    let date = parse_and_validate_date(&date_str)?;
    validate_hour(hour)?;

    let index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let index = index_state
        .as_ref()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;

    let candles = index
        .get_candles_for_hour(&symbol, date, hour)
        .ok_or(format!(
            "No candles found for {} on {} hour {}",
            symbol, date_str, hour
        ))?;

    if candles.is_empty() {
        return Err(format!(
            "No candles for {} on {} hour {} (empty result)",
            symbol, date_str, hour
        ));
    }

    Ok(CandlesForHourResponse {
        symbol,
        date: date_str,
        hour,
        candle_count: candles.len(),
        candles,
    })
}

/// Récupère les candles filtrées par quarter (15 min)
#[tauri::command]
pub async fn get_candles_for_quarter(
    symbol: String,
    hour: u8,
    quarter: u8,
    state: State<'_, CandleIndexState>,
) -> Result<CandlesForQuarterResponse, String> {
    validate_hour(hour as u32)?;
    validate_quarter(quarter)?;

    info!(
        "🔍 get_candles_for_quarter: {} hour={} quarter={}",
        symbol, hour, quarter
    );

    let mut index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let index = index_state.as_mut().ok_or("CandleIndex not initialized")?;

    if !index.is_pair_loaded(&symbol) {
        info!(
            "📥 Paire {} pas encore chargée - chargement en cours...",
            symbol
        );
        index.load_pair_candles(&symbol)?;
        info!("✅ Paire {} chargée", symbol);
    }

    let all_candles = index
        .get_all_candles(&symbol)
        .ok_or(format!("No candles found for {} after loading", symbol))?;

    info!(
        "📊 Total candles loaded for {}: {}",
        symbol,
        all_candles.len()
    );

    let filtered = filter_by_quarter(all_candles, hour, quarter);
    ensure_not_empty(&filtered, &symbol, hour, quarter)?;

    Ok(CandlesForQuarterResponse {
        symbol,
        hour,
        quarter,
        candle_count: filtered.len(),
        candles: filtered,
    })
}
