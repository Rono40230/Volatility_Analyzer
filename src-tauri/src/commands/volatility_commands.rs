// commands/volatility_commands.rs - Commandes Tauri pour l'analyse de volatilité
// Niveau 2 : Expose les services au frontend Vue.js
// Conforme .clinerules : < 300 lignes, thiserror, pas d'unwrap

use crate::commands::calendar_commands::CalendarState;
use crate::models::{AnalysisResult, HourlyStats};
use crate::services::{CsvLoader, VolatilityAnalyzer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

/// Structure pour les symboles disponibles
#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub file_path: String,
}

/// Résultat d'erreur sérialisable pour le frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandError {
    pub message: String,
    pub error_type: String,
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        CommandError {
            message,
            error_type: "Error".to_string(),
        }
    }
}

impl From<crate::models::VolatilityError> for CommandError {
    fn from(err: crate::models::VolatilityError) -> Self {
        CommandError {
            message: err.to_string(),
            error_type: format!("{:?}", err),
        }
    }
}

/// Liste tous les symboles disponibles (depuis DB pairs ou fallback CSV)
#[tauri::command]
pub async fn load_symbols(
    pair_state: State<'_, super::pair_data_commands::PairDataState>,
) -> Result<Vec<SymbolInfo>, CommandError> {
    info!("Command: load_symbols");

    // Essayer d'abord de charger depuis la DB pairs
    let pool_opt = pair_state.pool.lock().unwrap();
    if let Some(pool) = pool_opt.as_ref() {
        match crate::services::DatabaseLoader::new(pool.clone()).get_all_symbols() {
            Ok(symbols) => {
                if !symbols.is_empty() {
                    let symbol_infos: Vec<SymbolInfo> = symbols
                        .into_iter()
                        .map(|symbol| SymbolInfo {
                            symbol: symbol.clone(),
                            file_path: format!("db://pairs.db/{}", symbol),
                        })
                        .collect();

                    info!("Found {} symbols from DatabaseLoader", symbol_infos.len());
                    return Ok(symbol_infos);
                }
            }
            Err(e) => {
                info!("DatabaseLoader failed: {}, falling back to CsvLoader", e);
            }
        }
    }
    drop(pool_opt);

    // Fallback sur CsvLoader pour les anciennes données
    let loader = CsvLoader::new();

    let symbols = loader
        .list_available_symbols()
        .map_err(|e| {
            error!("Failed to list symbols: {}", e);
            CommandError::from(e)
        })?;

    let symbol_infos: Vec<SymbolInfo> = symbols
        .into_iter()
        .map(|symbol| SymbolInfo {
            symbol: symbol.clone(),
            file_path: format!("data/csv/{}", symbol),
        })
        .collect();

    info!("Found {} symbols from CsvLoader", symbol_infos.len());
    Ok(symbol_infos)
}

/// Analyse complète d'un symbole
#[tauri::command]
pub async fn analyze_symbol(
    symbol: String,
    calendar_id: i32,
    calendar_state: State<'_, CalendarState>,
    pair_state: State<'_, super::pair_data_commands::PairDataState>,
) -> Result<AnalysisResult, CommandError> {
    info!("Command: analyze_symbol({}, calendar_id={})", symbol, calendar_id);

    // Essayer d'abord DatabaseLoader
    let mut candles = Vec::new();
    let pool_opt = pair_state.pool.lock().unwrap();
    if let Some(pool) = pool_opt.as_ref() {
        let db_loader = crate::services::DatabaseLoader::new(pool.clone());
        // Charger tous les candles (période complète depuis 1970 jusqu'à maintenant)
        let start = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
        let end = Utc::now();
        match db_loader.load_candles_by_pair(&symbol, "M1", start, end) {
            Ok(loaded) => {
                candles = loaded;
                info!("Loaded {} candles for {} from DatabaseLoader", candles.len(), symbol);
            }
            Err(e) => {
                info!("DatabaseLoader failed for {}: {}, falling back to CsvLoader", symbol, e);
            }
        }
    }
    drop(pool_opt);

    // Fallback sur CsvLoader
    if candles.is_empty() {
        let loader = CsvLoader::new();
        candles = loader.load_candles(&symbol).map_err(|e| {
            error!("Failed to load candles for {} from both DB and CSV: {}", symbol, e);
            CommandError::from(e)
        })?;
        info!("Loaded {} candles for {} from CsvLoader", candles.len(), symbol);
    }

    info!("Total candles loaded for {}: {}", symbol, candles.len());

    // 2. Récupère le pool DB pour corrélation (optionnel)
    let pool = calendar_state.pool.lock()
        .map_err(|e| format!("Failed to lock calendar pool: {}", e))?
        .clone();

    // 3. Analyse avec VolatilityAnalyzer
    let analyzer = VolatilityAnalyzer::new(candles);
    let result = analyzer.analyze(&symbol, pool).map_err(|e| {
        error!("Failed to analyze {}: {}", symbol, e);
        CommandError::from(e)
    })?;

    info!(
        "Analysis complete for {}: confidence={:.1}, correlated_events={}",
        symbol,
        result.confidence_score,
        result.correlated_events.len()
    );

    Ok(result)
}

/// Récupère les statistiques horaires pour un symbole et une heure
#[tauri::command]
pub async fn get_hourly_stats(
    symbol: String,
    hour: u8,
    calendar_id: i32,
    calendar_state: State<'_, CalendarState>,
    pair_state: State<'_, super::pair_data_commands::PairDataState>,
) -> Result<HourlyStats, CommandError> {
    info!("Command: get_hourly_stats({}, hour={}, calendar_id={})", symbol, hour, calendar_id);

    if hour > 23 {
        return Err(CommandError {
            message: format!("Invalid hour: {}. Must be 0-23", hour),
            error_type: "ValidationError".to_string(),
        });
    }

    // Charge et analyse le symbole
    let result = analyze_symbol(symbol, calendar_id, calendar_state, pair_state).await?;

    // Cherche l'heure demandée
    let stats = result
        .hourly_stats
        .into_iter()
        .find(|s| s.hour == hour)
        .ok_or_else(|| CommandError {
            message: format!("No stats found for hour {}", hour),
            error_type: "NotFound".to_string(),
        })?;

    Ok(stats)
}

/// Récupère uniquement les meilleures heures pour un symbole
#[tauri::command]
pub async fn get_best_hours(
    symbol: String,
    calendar_id: i32,
    calendar_state: State<'_, CalendarState>,
    pair_state: State<'_, super::pair_data_commands::PairDataState>,
) -> Result<Vec<u8>, CommandError> {
    info!("Command: get_best_hours({}, calendar_id={})", symbol, calendar_id);

    let result = analyze_symbol(symbol, calendar_id, calendar_state, pair_state).await?;
    Ok(result.best_hours)
}

/// Ping pour vérifier que le backend est disponible
#[tauri::command]
pub async fn ping() -> String {
    info!("Command: ping");
    "pong".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let response = ping().await;
        assert_eq!(response, "pong");
    }

    #[test]
    fn test_command_error_from_volatility_error() {
        use crate::models::VolatilityError;

        let err = VolatilityError::InsufficientData("Test".to_string());
        let cmd_err = CommandError::from(err);

        assert!(cmd_err.message.contains("Test"));
    }
}
