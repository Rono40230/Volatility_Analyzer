use csv::ReaderBuilder;
use std::fs;

/// Importe des fichiers de calendrier économique dans volatility.db
#[tauri::command]
pub async fn import_calendar_files(paths: Vec<String>) -> Result<String, String> {
    use rusqlite::Connection;

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
    let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut reader = ReaderBuilder::new().delimiter(b',').from_reader(file);

    let mut event_count = 0;
    let mut oldest_date: Option<String> = None;
    let mut newest_date: Option<String> = None;

    // Parser les événements du CSV
    let mut events = Vec::new();
    let mut line_count = 0;
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parsing error: {}", e))?;
        line_count += 1;

        tracing::debug!(
            "📝 Line {}: {} fields: {:?}",
            line_count,
            record.len(),
            record.iter().collect::<Vec<_>>()
        );

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
        let event_time = format!(
            "{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:00",
            year, month, day, hour, minute
        );

        // Vérifier/mettre à jour les dates min/max
        if oldest_date
            .as_ref()
            .map(|o| event_time < *o)
            .unwrap_or(true)
        {
            oldest_date = Some(event_time.clone());
        }
        if newest_date
            .as_ref()
            .map(|n| event_time > *n)
            .unwrap_or(true)
        {
            newest_date = Some(event_time.clone());
        }

        events.push((
            event_time,
            symbol.to_string(),
            impact.to_string(),
            description.to_string(),
        ));
        event_count += 1;
    }

    tracing::info!("📊 Parsed {} events from {} lines", event_count, line_count);

    if event_count == 0 {
        return Err(format!(
            "Aucun événement trouvé dans le fichier (parsed {} lines)",
            line_count
        ));
    }

    // Ouvrir volatility.db
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db");

    let conn =
        Connection::open(&data_dir).map_err(|e| format!("Failed to open volatility.db: {}", e))?;

    // Extraire le nom du fichier
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?
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
    let mut stmt = conn
        .prepare(
            "INSERT INTO calendar_events (symbol, event_time, impact, description, calendar_import_id, created_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(|e| format!("Failed to prepare insert statement: {}", e))?;

    for (event_time, symbol, impact, description) in events {
        stmt.execute(rusqlite::params![
            &symbol,
            &event_time,
            &impact,
            &description,
            calendar_id,
            chrono::Utc::now().to_rfc3339()
        ])
        .map_err(|e| format!("Failed to insert event: {}", e))?;
    }

    tracing::info!(
        "✅ Calendar import complete: {} events imported",
        event_count
    );
    Ok(format!(
        "Calendrier importé avec succès: {} événements",
        event_count
    ))
}
