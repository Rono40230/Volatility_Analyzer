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
pub async fn process_forex_factory_csv(csv_content: String) -> Result<String, String> {
    if csv_content.trim().starts_with("<!DOCTYPE") || csv_content.trim().starts_with("<html") {
        return Err("Le contenu semble être une page HTML (Blocage Cloudflare).".to_string());
    }

    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(csv_content.as_bytes());

    let mut events = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parsing error: {}", e))?;
        if let Some((event_time, symbol_val, impact_val, description_val, actual, forecast, previous)) = parse_record(&record) {
            events.push((
                event_time,
                symbol_val,
                impact_val,
                description_val,
                actual,
                forecast,
                previous,
            ));
        }
    }

    // Récupération de la connection DB
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");
    let conn = Connection::open(&data_dir).map_err(|e| format!("Failed to open volatility.db: {}", e))?;

    let filename = "ff_calendar_thisweek.csv";
    let calendar_name = "Forex Factory Week";

    // Suppression de l'ancien si existant
    use crate::commands::calendar_db_helper::delete_calendar_import_by_name;
    let _ = delete_calendar_import_by_name(&conn, calendar_name);

    save_calendar_import(&conn, calendar_name, filename, &events).map_err(|e| format!("Save error: {}", e))?;
    
    Ok("Import réussi via le frontend".to_string())
}

#[tauri::command]
pub async fn clean_old_calendar_files_from_downloads() -> Result<(), String> {
    let download_dir = dirs::download_dir().ok_or("Impossible de trouver le dossier Téléchargements")?;
    
    if let Ok(entries) = fs::read_dir(&download_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("ff_calendar_thisweek") && name.ends_with(".csv") {
                     let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn check_download_folder_for_forex_factory() -> Result<String, String> {
    let download_dir = dirs::download_dir().ok_or("Impossible de trouver le dossier Téléchargements")?;
    
    // Find the newest file matching ff_calendar_thisweek*.csv
    let mut newest_file: Option<(std::path::PathBuf, std::time::SystemTime)> = None;

    if let Ok(entries) = fs::read_dir(&download_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("ff_calendar_thisweek") && name.ends_with(".csv") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            match newest_file {
                                Some((_, ref last_mod)) => {
                                    if modified > *last_mod {
                                        newest_file = Some((path, modified));
                                    }
                                }
                                None => {
                                    newest_file = Some((path, modified));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some((path, _)) = newest_file {
        return process_downloaded_file(path);
    }
    
    Err("Fichier non trouvé (en attente du téléchargement...)".to_string())
}

fn process_downloaded_file(path: std::path::PathBuf) -> Result<String, String> {
    tracing::info!("📂 file found at {:?}", path);
    
    // Attendre que le fichier soit complet (taille stable ou lock)
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Read content
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Parse using existing logic (reuse process_forex_factory_csv logic essentially)
    if content.trim().starts_with("<!DOCTYPE") || content.trim().starts_with("<html") {
         let _ = fs::remove_file(&path);
         return Err("Le fichier téléchargé est une page HTML (Blocage Cloudflare probable).".to_string());
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
                symbol_val,
                impact_val,
                description_val,
                actual,
                forecast,
                previous,
            ));
        }
    }

    if events.is_empty() {
         let _ = fs::remove_file(&path);
         return Err("Aucun événement trouvé dans le CSV".to_string());
    }

    // Save to DB
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");
    let conn = Connection::open(&data_dir).map_err(|e| format!("Failed to open volatility.db: {}", e))?;

    let filename = "ff_calendar_thisweek.csv";
    let calendar_name = "Forex Factory Week";

    use crate::commands::calendar_db_helper::delete_calendar_import_by_name;
    let _ = delete_calendar_import_by_name(&conn, calendar_name);

    let res = save_calendar_import(&conn, calendar_name, filename, &events);
    
    if res.is_ok() {
        // Cleanup file after successful import
        let _ = fs::remove_file(&path);
        Ok(format!("Import réussi: {} événements", events.len()))
    } else {
        Err(format!("Erreur sauvegarde DB: {:?}", res.err()))
    }
}

#[tauri::command]
pub async fn sync_forex_factory_week() -> Result<String, String> {
    tracing::info!("🔄 Starting Forex Factory sync (Internal Request)...");

    let url = "https://nfs.faireconomy.media/ff_calendar_thisweek.csv";
    
    // Setup headers to mimic a real Linux Browser EXACTLY to bypass simple filters
    let mut headers = reqwest::header::HeaderMap::new();
    // Chrome on Linux User Agent
    headers.insert("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
    headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
    headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap()); // Important
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "none".parse().unwrap()); // Direct type in address bar behavior
    headers.insert("Sec-Fetch-User", "?1".parse().unwrap());

    // Build client
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        if status.as_u16() == 429 {
             return Err("Erreur 429: Forex Factory limite les requêtes. Attendez quelques minutes.".to_string());
        }
        if status.as_u16() == 403 {
             return Err("Erreur 403: Accès bloqué par Cloudflare. Le téléchargement via l'application est restreint.".to_string());
        }
        return Err(format!("Erreur Forex Factory: Status {}", status));
    }

    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read content: {}", e))?;

    // Cloudflare HTML check
    if content.trim().starts_with("<!DOCTYPE") || content.trim().starts_with("<html") {
        return Err("Blocage Cloudflare détecté (Réponse HTML au lieu de CSV).".to_string());
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
        return Err("Fichier téléchargé vide ou format invalide".to_string());
    }

    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");

    let conn = Connection::open(&data_dir).map_err(|e| format!("DB Error: {}", e))?;

    // Supprimer les anciens plannings synchronisés
    conn.execute(
        "DELETE FROM calendar_imports WHERE name LIKE 'ForexFactory_Sync_%'",
        [],
    )
    .map_err(|e| format!("Delete error: {}", e))?;

    // Create unique import name
    let calendar_name = format!("ForexFactory_Sync_{}", chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S"));
    let filename = "ff_calendar_thisweek.csv";

    // Save
    save_calendar_import(&conn, &calendar_name, filename, &events).map_err(|e| format!("Save error: {}", e))?;

    Ok(format!("Import réussi ({} événements).", events.len()))
}
