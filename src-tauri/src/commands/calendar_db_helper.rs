use rusqlite::Connection;

pub fn save_calendar_import(
    conn: &Connection,
    name: &str,
    filename: &str,
    events: &[(String, String, String, String)],
) -> Result<i32, String> {
    if events.is_empty() {
        return Err("Aucun événement à sauvegarder".to_string());
    }

    let mut oldest_date: Option<String> = None;
    let mut newest_date: Option<String> = None;

    for (event_time, _, _, _) in events {
        if oldest_date
            .as_ref()
            .map(|o| event_time < o)
            .unwrap_or(true)
        {
            oldest_date = Some(event_time.clone());
        }
        if newest_date
            .as_ref()
            .map(|n| event_time > n)
            .unwrap_or(true)
        {
            newest_date = Some(event_time.clone());
        }
    }

    // Insérer l'enregistrement du calendrier
    let calendar_id: i32 = conn.query_row(
        "INSERT INTO calendar_imports (name, filename, event_count, oldest_event_date, newest_event_date, imported_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING id",
        rusqlite::params![name, filename, events.len(), &oldest_date, &newest_date, chrono::Utc::now().to_rfc3339()],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to insert calendar import record: {}", e))?;

    // Insérer les événements
    let mut stmt = conn
        .prepare(
            "INSERT INTO calendar_events (symbol, event_time, impact, description, calendar_import_id, created_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(|e| format!("Failed to prepare insert statement: {}", e))?;

    for (event_time, symbol, impact, description) in events {
        stmt.execute(rusqlite::params![
            symbol,
            event_time,
            impact,
            description,
            calendar_id,
            chrono::Utc::now().to_rfc3339()
        ])
        .map_err(|e| format!("Failed to insert event: {}", e))?;
    }

    Ok(calendar_id)
}
