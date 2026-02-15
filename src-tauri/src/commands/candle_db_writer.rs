// commands/candle_db_writer.rs
// Utilitaire partagé pour écrire des bougies M1 enrichies en BD (pairs.db).
// Utilisé par tick_import_commands et dukascopy_commands.

use crate::services::tick_aggregator::EnrichedM1;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Résultat de la sauvegarde en BD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResult {
    pub symbol: String,
    pub candles_saved: usize,
    pub actual_count: i64,
}

/// Ouvre pairs.db avec les options optimisées (WAL, busy_timeout).
pub fn open_pairs_db() -> Result<rusqlite::Connection, String> {
    let db_path = dirs::data_local_dir()
        .ok_or("Impossible de trouver le répertoire de données")?
        .join("volatility-analyzer")
        .join("pairs.db");

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Erreur ouverture pairs.db : {}", e))?;

    conn.busy_timeout(std::time::Duration::from_millis(5000))
        .map_err(|e| format!("Erreur busy_timeout : {}", e))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("Erreur WAL : {}", e))?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| format!("Erreur synchronous : {}", e))?;

    Ok(conn)
}

/// Sauvegarde des bougies M1 enrichies en BD avec UPSERT.
pub fn save_enriched_candles(
    conn: &rusqlite::Connection,
    symbol: &str,
    candles: &[EnrichedM1],
    source_label: &str,
) -> Result<SaveResult, String> {
    let imported_at = chrono::Utc::now().to_rfc3339();
    let timeframe = "M1";

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Erreur début transaction : {}", e))?;

    let mut stmt = tx
        .prepare(
            "INSERT INTO candle_data (symbol, timeframe, time, open, high, low, close, volume,
                spread_open, spread_high, spread_low, spread_close, spread_mean, tick_count,
                imported_at, source_file)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(symbol, timeframe, time) DO UPDATE SET
                open=excluded.open, high=excluded.high, low=excluded.low,
                close=excluded.close, volume=excluded.volume,
                spread_open=excluded.spread_open, spread_high=excluded.spread_high,
                spread_low=excluded.spread_low, spread_close=excluded.spread_close,
                spread_mean=excluded.spread_mean, tick_count=excluded.tick_count,
                imported_at=excluded.imported_at, source_file=excluded.source_file",
        )
        .map_err(|e| format!("Erreur prepare : {}", e))?;

    for (idx, candle) in candles.iter().enumerate() {
        stmt.execute(rusqlite::params![
            symbol,
            timeframe,
            candle.datetime_utc.to_rfc3339(),
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume,
            candle.spread_open,
            candle.spread_high,
            candle.spread_low,
            candle.spread_close,
            candle.spread_mean,
            candle.tick_count,
            &imported_at,
            source_label,
        ])
        .map_err(|e| format!("Erreur INSERT ligne {} : {}", idx, e))?;

        if idx % 50_000 == 0 && idx > 0 {
            info!("  ✓ {} M1 insérées", idx);
        }
    }
    drop(stmt);

    // Count réel en BD
    let actual_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM candle_data WHERE symbol = ? AND timeframe = ?",
            rusqlite::params![symbol, timeframe],
            |row| row.get(0),
        )
        .map_err(|e| format!("Erreur COUNT : {}", e))?;

    // Mise à jour pair_metadata
    tx.execute(
        "INSERT INTO pair_metadata (symbol, timeframe, row_count, last_updated, last_imported_file)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(symbol, timeframe) DO UPDATE SET
            row_count=excluded.row_count, last_updated=excluded.last_updated,
            last_imported_file=excluded.last_imported_file",
        rusqlite::params![symbol, timeframe, actual_count, &imported_at, source_label],
    )
    .map_err(|e| format!("Erreur pair_metadata : {}", e))?;

    // Log d'import
    tx.execute(
        "INSERT INTO import_log (filename, symbol, timeframe, row_count, expected_row_count, status, imported_at)
         VALUES (?, ?, ?, ?, ?, 'success', ?)",
        rusqlite::params![source_label, symbol, timeframe, candles.len() as i32, candles.len() as i32, &imported_at],
    )
    .map_err(|e| format!("Erreur import_log : {}", e))?;

    tx.commit().map_err(|e| format!("Erreur commit : {}", e))?;

    conn.execute_batch("REINDEX;")
        .map_err(|e| format!("Erreur REINDEX : {}", e))?;

    info!("✅ {} M1 sauvegardées pour {} (total BD : {})", candles.len(), symbol, actual_count);

    Ok(SaveResult {
        symbol: symbol.to_string(),
        candles_saved: candles.len(),
        actual_count,
    })
}
