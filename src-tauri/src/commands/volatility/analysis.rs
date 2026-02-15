use crate::commands::calendar_commands::CalendarState;
use crate::models::{AnalysisResult, VolatilityError};
use crate::services::cache_service::CacheService;
use crate::services::{CsvLoader, VolatilityAnalyzer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

/// État global pour le cache des résultats d'analyse (TTL 5 min, max 50 symboles)
pub struct AnalysisCacheState {
    pub cache: CacheService<AnalysisResult>,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub file_path: String,
}

#[tauri::command]
pub async fn load_symbols(
    pair_state: State<'_, super::super::pair_data::PairDataState>,
) -> Result<Vec<SymbolInfo>, CommandError> {
    info!("Command: load_symbols");

    let pool_opt = pair_state
        .pool
        .lock()
        .map_err(|_| CommandError::from("Failed to acquire database pool lock".to_string()))?;
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

    let loader = CsvLoader::new();
    let symbols = loader.list_available_symbols().map_err(|e| {
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

#[tauri::command]
pub async fn analyze_symbol(
    symbol: String,
    calendar_id: i32,
    date_start: Option<String>,
    date_end: Option<String>,
    calendar_state: State<'_, CalendarState>,
    pair_state: State<'_, super::super::pair_data::PairDataState>,
    analysis_cache: State<'_, AnalysisCacheState>,
) -> Result<AnalysisResult, CommandError> {
    info!(
        "Command: analyze_symbol({}, calendar_id={}, date_start={:?}, date_end={:?})",
        symbol, calendar_id, date_start, date_end
    );

    // Construire une clé de cache unique incluant les dates
    let cache_key = if date_start.is_some() || date_end.is_some() {
        format!("{}:{}:{}:{}", symbol, calendar_id, date_start.as_deref().unwrap_or(""), date_end.as_deref().unwrap_or(""))
    } else {
        format!("{}:{}", symbol, calendar_id)
    };

    // Vérifier le cache d'abord
    if let Some(cached) = analysis_cache.cache.get(&cache_key) {
        info!("Cache hit for {} (calendar_id={})", symbol, calendar_id);
        return Ok(cached);
    }
    info!("Cache miss for {} — computing full analysis", symbol);

    // Extraire les pools (rapide) avant spawn_blocking
    let pair_pool = {
        let pool_opt = pair_state
            .pool
            .lock()
            .map_err(|_| CommandError::from("Failed to acquire database pool lock".to_string()))?;
        pool_opt.clone()
    };

    let cal_pool = calendar_state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock calendar pool: {}", e))?
        .clone();

    // spawn_blocking : tout le travail lourd (chargement DB + analyse)
    let symbol_clone = symbol.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<AnalysisResult, VolatilityError> {
        let mut candles = Vec::new();

        // Parser les dates optionnelles
        let start = if let Some(ds) = date_start {
            // Attendu format: "YYYY-MM-DD"
            DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", ds))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| {
                    info!("Failed to parse date_start: {}, using epoch", ds);
                    DateTime::<Utc>::from_timestamp(0, 0).unwrap()
                })
        } else {
            DateTime::<Utc>::from_timestamp(0, 0).unwrap()
        };

        let end = if let Some(de) = date_end {
            // Attendu format: "YYYY-MM-DD"
            DateTime::parse_from_rfc3339(&format!("{}T23:59:59Z", de))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| {
                    info!("Failed to parse date_end: {}, using now", de);
                    Utc::now()
                })
        } else {
            Utc::now()
        };

        info!("Date range for analysis: {} to {}", start, end);

        if let Some(pool) = pair_pool.as_ref() {
            let db_loader = crate::services::DatabaseLoader::new(pool.clone());
            match db_loader.load_candles_by_pair(&symbol_clone, "M1", start, end) {
                Ok(loaded) => {
                    info!(
                        "Loaded {} candles for {} from DatabaseLoader",
                        loaded.len(),
                        symbol_clone
                    );
                    candles = loaded;
                }
                Err(e) => {
                    info!(
                        "DatabaseLoader failed for {}: {}, falling back to CsvLoader",
                        symbol_clone, e
                    );
                }
            }
        }

        if candles.is_empty() {
            let loader = CsvLoader::new();
            let mut all_candles = loader.load_candles(&symbol_clone).map_err(|e| {
                error!(
                    "Failed to load candles for {} from both DB and CSV: {}",
                    symbol_clone, e
                );
                e
            })?;
            
            // Filtrer par date si nécessaire
            all_candles.retain(|c| c.datetime >= start && c.datetime <= end);
            
            info!(
                "Loaded {} candles for {} from CsvLoader (filtered to date range)",
                all_candles.len(),
                symbol_clone
            );
            candles = all_candles;
        }

        info!("Total candles loaded for {}: {}", symbol_clone, candles.len());

        let analyzer = VolatilityAnalyzer::new(candles);
        analyzer.analyze(&symbol_clone, cal_pool)
    })
    .await
    .map_err(|e| CommandError::from(format!("Task join error: {}", e)))?
    .map_err(|e| {
        error!("Failed to analyze {}: {}", symbol, e);
        CommandError::from(e)
    })?;

    info!(
        "Analysis complete for {}: confidence={:.1}",
        symbol, result.confidence_score
    );

    // Stocker le résultat dans le cache
    analysis_cache.cache.set(cache_key, result.clone());

    Ok(result)
}

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
