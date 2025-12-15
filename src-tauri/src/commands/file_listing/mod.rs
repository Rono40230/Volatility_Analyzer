mod deletion;
mod types;
pub use deletion::{delete_calendar_file, delete_pair_files};
pub use types::{CalendarFileInfo, PairFileInfo};

use crate::services::pair_data_stats::{
    calculer_resume_paire, count_csv_lines, extract_date_range_from_path, PairDataSummary,
};
use chrono::{DateTime, Utc};
use std::fs;

#[tauri::command]
pub async fn list_calendar_files() -> Result<Vec<CalendarFileInfo>, String> {
    use rusqlite::Connection;

    tracing::info!("📂 Listing calendar files from database...");

    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");

    let db_path = data_dir.join("volatility.db");

    if !db_path.exists() {
        return Err("Database not found".to_string());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, name, filename, event_count, oldest_event_date, newest_event_date, imported_at FROM calendar_imports ORDER BY imported_at DESC")
        .map_err(|e| format!("Query failed: {}", e))?;

    let files = stmt
        .query_map([], |row| {
            let filename: String = row.get(2)?;
            let event_count: i32 = row.get(3)?;
            let oldest_date: Option<String> = row.get(4)?;
            let newest_date: Option<String> = row.get(5)?;
            let imported_at: String = row.get(6)?;

            let date_range = if let (Some(oldest), Some(newest)) = (oldest_date, newest_date) {
                Some(format!(
                    "du {} au {}",
                    oldest.split(' ').next().unwrap_or("?"),
                    newest.split(' ').next().unwrap_or("?")
                ))
            } else {
                None
            };

            Ok(CalendarFileInfo {
                filename,
                path: "<database>".to_string(),
                size_bytes: 0,
                created: imported_at.clone(),
                modified: imported_at,
                event_count: Some(event_count as i64),
                date_range,
            })
        })
        .map_err(|e| format!("Query execution failed: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Result collection failed: {}", e))?;

    tracing::info!("✅ Found {} calendar files in database", files.len());
    Ok(files)
}

#[tauri::command]
pub async fn list_pair_csv_files() -> Result<Vec<PairFileInfo>, String> {
    tracing::info!("📂 Listing pair CSV files...");
    let csv_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("data")
        .join("csv");
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
                        Some(
                            name_without_ext
                                .split('_')
                                .nth(1)
                                .map(|s| {
                                    if s.contains("Min")
                                        || name_without_ext.contains(&format!("{} Min", s))
                                    {
                                        format!("{} Min", s)
                                    } else {
                                        s.to_string()
                                    }
                                })
                                .unwrap_or_else(|| "N/A".to_string()),
                        )
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
                    let created = metadata
                        .created()
                        .ok()
                        .and_then(|t| {
                            DateTime::<Utc>::from(t)
                                .format("%Y-%m-%d %H:%M")
                                .to_string()
                                .into()
                        })
                        .unwrap_or_else(|| "N/A".to_string());
                    let modified = metadata
                        .modified()
                        .ok()
                        .and_then(|t| {
                            DateTime::<Utc>::from(t)
                                .format("%Y-%m-%d %H:%M")
                                .to_string()
                                .into()
                        })
                        .unwrap_or_else(|| "N/A".to_string());

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

#[tauri::command]
pub async fn get_pair_data_summary() -> Result<PairDataSummary, String> {
    tracing::info!("📊 Getting pair data summary...");

    let files = list_pair_csv_files().await?;

    let stats_files: Vec<crate::services::pair_data_stats::PairFileInfo> = files
        .iter()
        .map(|f| crate::services::pair_data_stats::PairFileInfo {
            pair: f.pair.clone(),
            line_count: f.line_count,
            size_bytes: f.size_bytes,
            date_range: f.date_range.clone(),
            modified: f.modified.clone(),
        })
        .collect();

    Ok(calculer_resume_paire(stats_files))
}
