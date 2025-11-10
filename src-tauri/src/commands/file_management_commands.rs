// file_management_commands.rs
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::{DateTime, Utc};
use tauri::State;
use crate::services::pair_data_stats::{
    PairDataSummary, calculate_pair_summary,
    count_csv_lines, extract_date_range_from_path
};
use crate::services::calendar_file_stats::{count_csv_events, extract_calendar_date_range};

#[derive(Debug, Serialize, Deserialize)]
pub struct PairMetadataInfo {
    pub symbol: String,
    pub timeframe: String,
    pub row_count: i32,
    pub last_updated: String,
    pub last_imported_file: String,
    pub quality_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarFileInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub created: String,
    pub modified: String,
    pub event_count: Option<i64>,
    pub date_range: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairFileInfo {
    pub filename: String,
    pub path: String,
    pub pair: Option<String>,
    pub timeframe: Option<String>,
    pub period: Option<String>,
    pub size_bytes: u64,
    pub line_count: Option<usize>,
    pub date_range: Option<String>,
    pub created: String,
    pub modified: String,
}

/// Liste tous les fichiers de calendrier économique
#[tauri::command]
pub async fn list_calendar_files() -> Result<Vec<CalendarFileInfo>, String> {
    tracing::info!("📂 Listing calendar files...");
    let data_dir = dirs::data_local_dir().ok_or("Failed to get data directory")?.join("volatility-analyzer");
    let mut files = Vec::new();
    if data_dir.exists() {
        for entry in fs::read_dir(&data_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "db" || ext_str == "csv" || ext_str == "xlsx" {
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy();
                        if filename_str == "volatility.db" { continue; }
                        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                        let created = metadata.created().ok().and_then(|t| DateTime::<Utc>::from(t).format("%Y-%m-%d %H:%M").to_string().into()).unwrap_or_else(|| "N/A".to_string());
                        let modified = metadata.modified().ok().and_then(|t| DateTime::<Utc>::from(t).format("%Y-%m-%d %H:%M").to_string().into()).unwrap_or_else(|| "N/A".to_string());
                        let date_range = extract_calendar_date_range(&filename_str);
                        let event_count = if ext_str == "csv" { count_csv_events(&path) } else { None };
                        
                        files.push(CalendarFileInfo {
                            filename: filename.to_string_lossy().to_string(),
                            path: path.to_string_lossy().to_string(),
                            size_bytes: metadata.len(),
                            created,
                            modified,
                            event_count,
                            date_range,
                        });
                    }
                }
            }
        }
    }
    
    tracing::info!("✅ Found {} calendar files", files.len());
    Ok(files)
}

/// Liste tous les fichiers CSV de paires
#[tauri::command]
pub async fn list_pair_csv_files() -> Result<Vec<PairFileInfo>, String> {
    tracing::info!("📂 Listing pair CSV files...");
    let csv_dir = dirs::data_local_dir().ok_or("Failed to get data directory")?.join("volatility-analyzer").join("data").join("csv");
    let mut files = Vec::new();
    if csv_dir.exists() {
        for entry in fs::read_dir(&csv_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            
            if path.extension().and_then(|e| e.to_str()) == Some("csv") {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy().to_string();
                    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                    let name_without_ext = filename_str.trim_end_matches(".csv");
                    let pair = name_without_ext.split('_').next().map(|s| s.to_string());
                    let timeframe = if name_without_ext.contains(" Min") {
                        Some(name_without_ext.split('_').nth(1).map(|s| {
                                if s.contains("Min") || name_without_ext.contains(&format!("{} Min", s)) {
                                    format!("{} Min", s)
                                } else {
                                    s.to_string()
                                }
                            }).unwrap_or_else(|| "N/A".to_string()))
                    } else {
                        name_without_ext.split('_').nth(1).map(|s| s.to_string())
                    };
                    let period = if name_without_ext.contains("Bid") {
                        Some("Bid".to_string())
                    } else if name_without_ext.contains("Ask") {
                        Some("Ask".to_string())
                    } else {
                        name_without_ext.split('_').nth(2).map(|s| s.to_string())
                    };
                    let line_count = count_csv_lines(&path);
                    let date_range = extract_date_range_from_path(&path);
                    let created = metadata.created().ok().and_then(|t| DateTime::<Utc>::from(t).format("%Y-%m-%d %H:%M").to_string().into()).unwrap_or_else(|| "N/A".to_string());
                    let modified = metadata.modified().ok().and_then(|t| DateTime::<Utc>::from(t).format("%Y-%m-%d %H:%M").to_string().into()).unwrap_or_else(|| "N/A".to_string());
                    
                    files.push(PairFileInfo {
                        filename: filename.to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        pair,
                        timeframe,
                        period,
                        size_bytes: metadata.len(),
                        line_count,
                        date_range,
                        created,
                        modified,
                    });
                }
            }
        }
    }
    Ok(files)
}

/// Supprime un fichier de calendrier
#[tauri::command]
pub async fn delete_calendar_file(file_path: String) -> Result<(), String> {
    tracing::info!("🗑️  Deleting calendar file: {}", file_path);
    fs::remove_file(&file_path).map_err(|e| format!("Failed to delete file: {}", e))?;
    tracing::info!("✅ File deleted successfully");
    Ok(())
}

/// Supprime des fichiers de paires CSV
#[tauri::command]
pub async fn delete_pair_files(file_paths: Vec<String>) -> Result<usize, String> {
    tracing::info!("🗑️  Deleting {} pair files", file_paths.len());
    let mut deleted = 0;
    for path in file_paths {
        if let Err(e) = fs::remove_file(&path) {
            tracing::warn!("Failed to delete {}: {}", path, e);
        } else {
            deleted += 1;
        }
    }
    tracing::info!("✅ Deleted {} files", deleted);
    Ok(deleted)
}

/// Récupère les statistiques globales des données de paires
#[tauri::command]
pub async fn get_pair_data_summary() -> Result<PairDataSummary, String> {
    tracing::info!("📊 Getting pair data summary...");
    
    let files = list_pair_csv_files().await?;
    
    // Convertir les PairFileInfo vers le format attendu par le service
    let stats_files: Vec<crate::services::pair_data_stats::PairFileInfo> = files.iter().map(|f| {
        crate::services::pair_data_stats::PairFileInfo {
            pair: f.pair.clone(),
            line_count: f.line_count,
            size_bytes: f.size_bytes,
            date_range: f.date_range.clone(),
            modified: f.modified.clone(),
        }
    }).collect();
    
    Ok(calculate_pair_summary(stats_files))
}

/// Récupère les métadonnées des paires depuis pairs.db
#[tauri::command]
pub async fn get_pair_metadata_from_db(
    pair_state: State<'_, super::pair_data_commands::PairDataState>,
) -> Result<Vec<PairMetadataInfo>, String> {
    use rusqlite::Connection;
    
    tracing::info!("📊 Getting pair metadata from pairs.db...");
    
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");
    
    let conn = Connection::open(&data_dir)
        .map_err(|e| format!("Failed to open pairs.db: {}", e))?;
    
    let mut stmt = conn
        .prepare("SELECT symbol, timeframe, row_count, last_updated, last_imported_file, quality_score FROM pair_metadata ORDER BY symbol, timeframe")
        .map_err(|e| format!("Query failed: {}", e))?;
    
    let pairs = stmt
        .query_map([], |row| {
            Ok(PairMetadataInfo {
                symbol: row.get(0)?,
                timeframe: row.get(1)?,
                row_count: row.get(2)?,
                last_updated: row.get(3)?,
                last_imported_file: row.get(4)?,
                quality_score: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query execution failed: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Result collection failed: {}", e))?;
    
    tracing::info!("✅ Found {} pairs in database", pairs.len());
    Ok(pairs)
}
