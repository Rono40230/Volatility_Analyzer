use crate::commands::calendar_db_helper::save_calendar_import;
use crate::commands::calendar_parser::parse_record;
use csv::ReaderBuilder;
use rusqlite::Connection;
use std::fs;

#[tauri::command]
pub async fn import_calendar_files(paths: Vec<String>) -> Result<String, String> {
    tracing::info!("📥 Starting calendar import for {} file(s)", paths.len());

    if paths.is_empty() {
        return Err("Aucun fichier fourni".to_string());
    }

    let path = &paths[0];
    let file_path = std::path::Path::new(path);

    if !file_path.exists() {
        return Err(format!("Fichier non trouvé: {}", path));
    }

    let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = ReaderBuilder::new().delimiter(b',').from_reader(file);

    let mut events = Vec::new();
    let mut line_count = 0;

    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parsing error: {}", e))?;
        line_count += 1;

        if let Some((event_time, symbol_val, impact_val, description_val, actual, forecast, previous)) = parse_record(&record) {
            events.push((
                event_time,
                symbol_val.to_string(),
                impact_val.to_string(),
                description_val.to_string(),
                actual,
                forecast,
                previous
            ));
        } else if line_count <= 5 {
            tracing::warn!("⚠️ Rejected line {}: {:?}", line_count, record);
        }
    }

    tracing::info!("📊 Parsed {} events from {} lines", events.len(), line_count);

    if events.is_empty() {
        return Err(format!(
            "Aucun événement trouvé dans le fichier (parsed {} lines)",
            line_count
        ));
    }

    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");

    let conn =
        Connection::open(&data_dir).map_err(|e| format!("Failed to open volatility.db: {}", e))?;

    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?
        .to_string();

    let calendar_name = filename.trim_end_matches(".csv").to_string();

    // On supprime toujours l'ancien calendrier du même nom s'il existe (remplacement automatique)
    use crate::commands::calendar_db_helper::delete_calendar_import_by_name;
    let _ = delete_calendar_import_by_name(&conn, &calendar_name);

    let calendar_id = save_calendar_import(&conn, &calendar_name, &filename, &events)?;

    tracing::info!("📝 Calendar import record created with ID: {}", calendar_id);
    tracing::info!(
        "✅ Calendar import complete: {} events imported",
        events.len()
    );
    Ok(format!(
        "Calendrier importé avec succès: {} événements",
        events.len()
    ))
}

#[tauri::command]
pub async fn sync_forex_factory_week() -> Result<String, String> {
    tracing::info!("🔄 Starting Forex Factory sync...");

    let url = "https://nfs.faireconomy.media/ff_calendar_thisweek.csv";
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download calendar: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Erreur de téléchargement Forex Factory: Status {}",
            response.status()
        ));
    }

    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {}", e))?;

    if content.trim().starts_with("<!DOCTYPE") || content.trim().starts_with("<html") {
        return Err("Forex Factory a bloqué la requête (Rate Limit). Veuillez réessayer plus tard.".to_string());
    }

    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(content.as_bytes());

    let mut events = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parsing error: {}", e))?;
        if let Some((event_time, symbol_val, impact_val, description_val, actual, forecast, previous)) = parse_record(&record) {
            events.push((
                event_time,
                symbol_val.to_string(),
                impact_val.to_string(),
                description_val.to_string(),
                actual,
                forecast,
                previous
            ));
        }
    }

    if events.is_empty() {
        return Err("Aucun événement trouvé dans le fichier téléchargé".to_string());
    }

    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");

    let conn =
        Connection::open(&data_dir).map_err(|e| format!("Failed to open volatility.db: {}", e))?;

    // Supprimer les anciens plannings synchronisés pour éviter les doublons
    conn.execute(
        "DELETE FROM calendar_imports WHERE name LIKE 'ForexFactory_Sync_%'",
        [],
    )
    .map_err(|e| format!("Failed to delete old syncs: {}", e))?;

    let calendar_name = format!("ForexFactory_Sync_{}", chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S"));
    let filename = "ff_calendar_thisweek.csv";

    save_calendar_import(&conn, &calendar_name, &filename, &events)?;

    tracing::info!(
        "✅ Forex Factory sync complete: {} events imported",
        events.len()
    );
    Ok(format!(
        "Synchronisation réussie: {} événements ajoutés",
        events.len()
    ))
}
