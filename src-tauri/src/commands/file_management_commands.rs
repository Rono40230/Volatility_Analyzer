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
pub struct CalendarImportInfo {
    pub id: i32,
    pub name: String,
    pub filename: String,
    pub event_count: i32,
    pub oldest_event_date: Option<String>,
    pub newest_event_date: Option<String>,
    pub imported_at: String,
    pub is_active: bool,
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
                        if filename_str == "volatility.db" || filename_str == "pairs.db" { continue; }
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
    _pair_state: State<'_, super::pair_data_commands::PairDataState>,
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
        .prepare("SELECT symbol, timeframe, row_count, last_updated, last_imported_file, data_quality_score FROM pair_metadata ORDER BY symbol, timeframe")
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

/// Récupère la liste des calendriers importés depuis volatility.db
#[tauri::command]
pub async fn get_calendar_imports_from_db() -> Result<Vec<CalendarImportInfo>, String> {
    use rusqlite::Connection;
    
    tracing::info!("📊 Getting calendar imports from volatility.db...");
    
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");
    
    let conn = Connection::open(&data_dir)
        .map_err(|e| format!("Failed to open volatility.db: {}", e))?;
    
    let mut stmt = conn
        .prepare("SELECT id, name, filename, event_count, oldest_event_date, newest_event_date, imported_at, is_active FROM calendar_imports ORDER BY imported_at DESC")
        .map_err(|e| format!("Query failed: {}", e))?;
    
    let calendars = stmt
        .query_map([], |row| {
            Ok(CalendarImportInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                filename: row.get(2)?,
                event_count: row.get(3)?,
                oldest_event_date: row.get(4)?,
                newest_event_date: row.get(5)?,
                imported_at: row.get(6)?,
                is_active: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query execution failed: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Result collection failed: {}", e))?;
    
    tracing::info!("✅ Found {} calendar imports in database", calendars.len());
    Ok(calendars)
}

/// Récupère l'ID du calendrier actif
#[tauri::command]
pub async fn get_active_calendar_id() -> Result<Option<i32>, String> {
    use rusqlite::Connection;
    
    tracing::info!("🔍 Getting active calendar ID...");
    
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");
    
    let conn = Connection::open(&data_dir)
        .map_err(|e| format!("Failed to open volatility.db: {}", e))?;
    
    let mut stmt = conn
        .prepare("SELECT id FROM calendar_imports WHERE is_active = 1 LIMIT 1")
        .map_err(|e| format!("Query failed: {}", e))?;
    
    let calendar_id = stmt
        .query_row([], |row| row.get::<_, i32>(0))
        .ok();
    
    tracing::info!("✅ Active calendar ID: {:?}", calendar_id);
    Ok(calendar_id)
}

/// Importe des fichiers de calendrier économique dans volatility.db
#[tauri::command]
pub async fn import_calendar_files(paths: Vec<String>) -> Result<String, String> {
    use rusqlite::Connection;
    use std::fs;
    use csv::ReaderBuilder;
    
    tracing::info!("📥 Starting calendar import for {} file(s)", paths.len());
    
    if paths.is_empty() {
        return Err("Aucun fichier fourni".to_string());
    }
    
    let path = &paths[0]; // Pour l'instant, on n'en traite qu'un
    let file_path = std::path::Path::new(path);
    
    if !file_path.exists() {
        return Err(format!("Fichier non trouvé: {}", path));
    }
    
    // Lire le fichier CSV
    let file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(file);
    
    let mut event_count = 0;
    let mut oldest_date: Option<String> = None;
    let mut newest_date: Option<String> = None;
    
    // Parser les événements du CSV
    let mut events = Vec::new();
    let mut line_count = 0;
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parsing error: {}", e))?;
        line_count += 1;
        
        tracing::debug!("📝 Line {}: {} fields: {:?}", line_count, record.len(), record.iter().collect::<Vec<_>>());
        
        // Format attendu: year,month,day,hour,minute,currency,impact,description,...
        if record.len() < 8 {
            tracing::debug!("   ⏭️ Skipping (< 8 fields)");
            continue; // Ignorer les lignes incomplètes
        }
        
        // Construire la date/heure au format ISO
        let year = record[0].trim();
        let month = record[1].trim();
        let day = record[2].trim();
        let hour = record[3].trim();
        let minute = record[4].trim();
        let symbol = record[5].trim(); // Currency = symbol
        let impact = record[6].trim();
        let description = record[7].trim();
        
        // Format: YYYY-MM-DD HH:MM:00
        let event_time = format!("{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:00", year, month, day, hour, minute);
        
        // Vérifier/mettre à jour les dates min/max
        if oldest_date.is_none() || event_time < *oldest_date.as_ref().unwrap() {
            oldest_date = Some(event_time.clone());
        }
        if newest_date.is_none() || event_time > *newest_date.as_ref().unwrap() {
            newest_date = Some(event_time.clone());
        }
        
        events.push((event_time, symbol.to_string(), impact.to_string(), description.to_string()));
        event_count += 1;
    }
    
    tracing::info!("📊 Parsed {} events from {} lines", event_count, line_count);
    
    if event_count == 0 {
        return Err(format!("Aucun événement trouvé dans le fichier (parsed {} lines)", line_count));
    }
    
    // Ouvrir volatility.db
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");
    
    let conn = Connection::open(&data_dir)
        .map_err(|e| format!("Failed to open volatility.db: {}", e))?;
    
    // Extraire le nom du fichier
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("calendar")
        .to_string();
    
    let calendar_name = filename.trim_end_matches(".csv").to_string();
    
    // Insérer l'enregistrement du calendrier
    let calendar_id: i32 = conn.query_row(
        "INSERT INTO calendar_imports (name, filename, event_count, oldest_event_date, newest_event_date, imported_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING id",
        rusqlite::params![&calendar_name, &filename, event_count, &oldest_date, &newest_date, chrono::Utc::now().to_rfc3339()],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to insert calendar import record: {}", e))?;
    
    tracing::info!("📝 Calendar import record created with ID: {}", calendar_id);
    
    // Insérer les événements
    let mut stmt = conn.prepare(
        "INSERT INTO calendar_events (symbol, event_time, impact, description, created_at) 
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )
    .map_err(|e| format!("Failed to prepare insert statement: {}", e))?;
    
    for (event_time, symbol, impact, description) in events {
        stmt.execute(rusqlite::params![&symbol, &event_time, &impact, &description, chrono::Utc::now().to_rfc3339()])
            .map_err(|e| format!("Failed to insert event: {}", e))?;
    }
    
    tracing::info!("✅ Calendar import complete: {} events imported", event_count);
    Ok(format!("Calendrier importé avec succès: {} événements", event_count))
}

/// Supprime une paire (pair_metadata + tous les candles) de la BD
#[tauri::command]
pub fn delete_pair_from_db(symbol: String, timeframe: String) -> Result<String, String> {
    let db_path = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");

    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open pairs.db: {}", e))?;

    // Démarrer une transaction pour s'assurer que tout est supprimé ou rien
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Supprimer tous les candles de cette paire
    let candles_deleted = tx
        .execute(
            "DELETE FROM candle_data WHERE symbol = ? AND timeframe = ?",
            rusqlite::params![&symbol, &timeframe],
        )
        .map_err(|e| format!("Failed to delete candles: {}", e))?;

    tracing::info!("🗑️  Deleted {} candles for {}/{}", candles_deleted, symbol, timeframe);

    // Supprimer la métadonnée de la paire
    let metadata_deleted = tx
        .execute(
            "DELETE FROM pair_metadata WHERE symbol = ? AND timeframe = ?",
            rusqlite::params![&symbol, &timeframe],
        )
        .map_err(|e| format!("Failed to delete pair metadata: {}", e))?;

    tracing::info!("🗑️  Deleted {} metadata records for {}/{}", metadata_deleted, symbol, timeframe);

    // Commit la transaction
    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(format!("Paire {}/{} supprimée avec succès ({} candles supprimés)", symbol, timeframe, candles_deleted))
}

/// Supprime un calendrier (calendar_imports + tous les événements) de la BD
#[tauri::command]
pub fn delete_calendar_from_db(calendar_id: i32) -> Result<String, String> {
    let db_path = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");

    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open volatility.db: {}", e))?;

    // Démarrer une transaction
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Récupérer le nom du calendrier avant suppression (pour le message)
    let calendar_name: String = tx
        .query_row(
            "SELECT name FROM calendar_imports WHERE id = ?",
            rusqlite::params![calendar_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Calendar not found: {}", e))?;

    // Supprimer tous les événements de ce calendrier
    let events_deleted = tx
        .execute(
            "DELETE FROM calendar_events WHERE symbol IN (SELECT DISTINCT symbol FROM calendar_events) AND symbol LIKE ?",
            rusqlite::params![&calendar_name],
        )
        .map_err(|e| format!("Failed to delete calendar events: {}", e))?;

    tracing::info!("🗑️  Deleted {} events for calendar '{}'", events_deleted, calendar_name);

    // Supprimer l'enregistrement du calendrier
    let metadata_deleted = tx
        .execute(
            "DELETE FROM calendar_imports WHERE id = ?",
            rusqlite::params![calendar_id],
        )
        .map_err(|e| format!("Failed to delete calendar import: {}", e))?;

    tracing::info!("🗑️  Deleted calendar import record for '{}'", calendar_name);

    // Commit la transaction
    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(format!("Calendrier '{}' supprimé avec succès ({} événements supprimés)", calendar_name, events_deleted))
}
