use crate::commands::pair_data::PairDataState;
use diesel::prelude::*;
use tauri::State;

/// Supprime une paire (pair_metadata + tous les candles) de la BD
#[tauri::command]
pub async fn delete_pair_from_db(
    state: State<'_, PairDataState>,
    symbol: String,
    timeframe: String,
) -> Result<String, String> {
    let symbol_clean = symbol.trim().to_string();
    let timeframe_clean = timeframe.trim().to_string();
    
    tracing::info!(
        "🗑️ [Delete Pair] Request received for '{}'/'{}'. (Hex Sym: {:?}, Hex TF: {:?})", 
        symbol_clean, 
        timeframe_clean,
        symbol_clean.as_bytes(),
        timeframe_clean.as_bytes()
    );

    // Utiliser le pool Diesel existant au lieu d'ouvrir une nouvelle connexion Rusqlite
    tracing::debug!("🗑️ [Delete Pair] Acquiring pool lock...");
    let pool = state
        .pool
        .lock()
        .map_err(|_| "Failed to lock pool mutex".to_string())?
        .clone()
        .ok_or("Database pool not initialized".to_string())?;
    tracing::debug!("🗑️ [Delete Pair] Pool lock acquired.");

    // Exécuter dans un thread bloquant pour ne pas bloquer l'executor async
    tokio::task::spawn_blocking(move || {
        tracing::debug!("🗑️ [Delete Pair] Inside spawn_blocking, requesting connection...");
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get connection from pool: {}", e))?;
        tracing::debug!("🗑️ [Delete Pair] Connection acquired from pool.");

        // Exécuter la suppression via SQL brut avec Diesel
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            tracing::debug!("🗑️ [Delete Pair] Transaction started.");
            
            // 1. Supprimer les candles
            tracing::debug!("🗑️ [Delete Pair] Deleting candles...");
            let candles_deleted = diesel::sql_query("DELETE FROM candle_data WHERE symbol = ? AND timeframe = ?")
                .bind::<diesel::sql_types::Text, _>(&symbol_clean)
                .bind::<diesel::sql_types::Text, _>(&timeframe_clean)
                .execute(conn)?;

            tracing::info!(
                "🗑️  Deleted {} candles for {}/{}",
                candles_deleted,
                symbol_clean,
                timeframe_clean
            );

            // 2. Supprimer les métadonnées
            tracing::debug!("🗑️ [Delete Pair] Deleting metadata...");
            let metadata_deleted = diesel::sql_query("DELETE FROM pair_metadata WHERE symbol = ? AND timeframe = ?")
                .bind::<diesel::sql_types::Text, _>(&symbol_clean)
                .bind::<diesel::sql_types::Text, _>(&timeframe_clean)
                .execute(conn)?;

            tracing::info!(
                "🗑️  Deleted {} metadata records for {}/{}",
                metadata_deleted,
                symbol_clean,
                timeframe_clean
            );

            if metadata_deleted == 0 && candles_deleted == 0 {
                // Si rien n'a été supprimé, cela peut être dû à une incohérence.
                // Essayons une suppression par LIKE pour être plus permissif si le strict a échoué ?
                // Non, restons stricts pour l'instant mais renvoyons une erreur précise.
                return Err(diesel::result::Error::NotFound);
            }

            Ok(())
        })
        .map_err(|e| format!("Erreur suppression: {} (Cible: {}/{})", e, symbol_clean, timeframe_clean))?;

        tracing::debug!("🗑️ [Delete Pair] Transaction committed successfully.");
        Ok(format!(
            "Paire {}/{} supprimée avec succès",
            symbol_clean, timeframe_clean
        ))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

use crate::commands::calendar_commands::CalendarState;

/// Supprime un calendrier (calendar_imports + tous les événements) de la BD
#[tauri::command]
pub async fn delete_calendar_from_db(
    state: State<'_, CalendarState>,
    calendar_id: i32,
) -> Result<String, String> {
    tracing::info!("🗑️ [Delete Calendar] Request received for ID {}", calendar_id);

    // Utiliser le pool Diesel existant
    tracing::debug!("🗑️ [Delete Calendar] Acquiring pool lock...");
    let pool = state
        .pool
        .lock()
        .map_err(|_| "Failed to lock pool mutex".to_string())?
        .clone()
        .ok_or("Database pool not initialized".to_string())?;
    tracing::debug!("🗑️ [Delete Calendar] Pool lock acquired.");

    tokio::task::spawn_blocking(move || {
        tracing::debug!("🗑️ [Delete Calendar] Inside spawn_blocking, requesting connection...");
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get connection from pool: {}", e))?;
        tracing::debug!("🗑️ [Delete Calendar] Connection acquired from pool.");

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            tracing::debug!("🗑️ [Delete Calendar] Transaction started.");

            // 1. Supprimer les événements
            tracing::debug!("🗑️ [Delete Calendar] Deleting events...");
            let events_deleted = diesel::sql_query("DELETE FROM calendar_events WHERE calendar_import_id = ?")
                .bind::<diesel::sql_types::Integer, _>(calendar_id)
                .execute(conn)?;

            tracing::info!(
                "🗑️  Deleted {} events for calendar ID {}",
                events_deleted,
                calendar_id
            );

            // 2. Supprimer l'import
            tracing::debug!("🗑️ [Delete Calendar] Deleting import record...");
            let _import_deleted = diesel::sql_query("DELETE FROM calendar_imports WHERE id = ?")
                .bind::<diesel::sql_types::Integer, _>(calendar_id)
                .execute(conn)?;

            tracing::info!("🗑️  Deleted calendar import record ID {}", calendar_id);

            Ok(())
        })
        .map_err(|e| format!("Transaction failed: {}", e))?;

        tracing::debug!("🗑️ [Delete Calendar] Transaction committed successfully.");
        Ok(format!(
            "Calendrier ID {} supprimé avec succès",
            calendar_id
        ))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
