// services/candle_loader.rs
// Extrait la logique de chargement de CandleIndex

use crate::models::Candle;
use crate::services::{CsvLoader, DatabaseLoader};
use chrono::{DateTime, Utc};

/// Charge une paire spécifique (lazy loading)
/// Utilise DatabaseLoader si disponible, sinon fallback sur CsvLoader
pub fn load_pair_candles_strategy(
    db_loader: Option<&DatabaseLoader>,
    symbol: &str,
) -> Result<Vec<Candle>, String> {
    if let Some(loader) = db_loader {
        // Charger depuis la BD via DatabaseLoader
        // Charger TOUTES les candles disponibles pour ce symbole
        let start_time = DateTime::from_timestamp(0, 0)
            .ok_or_else(|| "Failed to create start datetime".to_string())?;
        let end_time =
            DateTime::from_timestamp(2_000_000_000, 0) // ~2033
                .ok_or_else(|| "Failed to create end datetime".to_string())?;

        loader
            .load_candles_by_pair(symbol, "M1", start_time, end_time)
            .map_err(|e| format!("Failed to load candles for {} from DB: {}", symbol, e))
    } else {
        // Fallback sur CsvLoader si pas de DatabaseLoader
        let loader = CsvLoader::new();
        loader
            .load_candles(symbol)
            .map_err(|e| format!("Failed to load candles for {}: {}", symbol, e))
    }
}

/// Charge une paire spécifique pour une plage de dates donnée
pub fn load_pair_candles_in_range_strategy(
    db_loader: Option<&DatabaseLoader>,
    symbol: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<Candle>, String> {
    if let Some(loader) = db_loader {
        loader
            .load_candles_by_pair(symbol, "M1", start, end)
            .map_err(|e| format!("Failed to load candles for {} from DB: {}", symbol, e))
    } else {
        // Fallback sur CsvLoader (qui charge tout de toute façon)
        // Note: CsvLoader ne supporte pas le range loading natif pour l'instant
        let loader = CsvLoader::new();
        let all_candles = loader
            .load_candles(symbol)
            .map_err(|e| format!("Failed to load candles for {}: {}", symbol, e))?;
            
        // Filtrer manuellement
        Ok(all_candles.into_iter()
            .filter(|c| c.datetime >= start && c.datetime <= end)
            .collect())
    }
}
