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

    tracing::info!(
        "🗑️  Deleted {} candles for {}/{}",
        candles_deleted,
        symbol,
        timeframe
    );

    // Supprimer la métadonnée de la paire
    let metadata_deleted = tx
        .execute(
            "DELETE FROM pair_metadata WHERE symbol = ? AND timeframe = ?",
            rusqlite::params![&symbol, &timeframe],
        )
        .map_err(|e| format!("Failed to delete pair metadata: {}", e))?;

    tracing::info!(
        "🗑️  Deleted {} metadata records for {}/{}",
        metadata_deleted,
        symbol,
        timeframe
    );

    // Commit la transaction
    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(format!(
        "Paire {}/{} supprimée avec succès ({} candles supprimés)",
        symbol, timeframe, candles_deleted
    ))
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
            "DELETE FROM calendar_events WHERE calendar_import_id = ?",
            rusqlite::params![calendar_id],
        )
        .map_err(|e| format!("Failed to delete calendar events: {}", e))?;

    tracing::info!(
        "🗑️  Deleted {} events for calendar '{}'",
        events_deleted,
        calendar_name
    );

    // Supprimer l'enregistrement du calendrier
    let _metadata_deleted = tx
        .execute(
            "DELETE FROM calendar_imports WHERE id = ?",
            rusqlite::params![calendar_id],
        )
        .map_err(|e| format!("Failed to delete calendar import: {}", e))?;

    tracing::info!("🗑️  Deleted calendar import record for '{}'", calendar_name);

    // Commit la transaction
    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(format!(
        "Calendrier '{}' supprimé avec succès ({} événements supprimés)",
        calendar_name, events_deleted
    ))
}
