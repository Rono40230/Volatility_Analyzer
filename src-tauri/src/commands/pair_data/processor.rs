use crate::db::DbPool;
use crate::services::PairDataConverter;
use chrono::Utc;
use std::path::Path;
use tracing::info;

pub fn process_single_file(
    source_path: &str,
    _pool: &DbPool,
) -> Result<(String, String, usize), String> {
    info!("🔄 Normalisation: {}", source_path);
    let candles = PairDataConverter::read_and_normalize(source_path)?;

    if candles.is_empty() {
        return Err("Aucune donnée valide trouvée".to_string());
    }

    let row_count = candles.len();

    let filename = Path::new(source_path)
        .file_name()
        .ok_or("Nom de fichier invalide")?
        .to_str()
        .ok_or("Nom de fichier non-UTF8")?;

    info!("📊 Extraction métadonnées de: {}", filename);
    let metadata = PairDataConverter::extract_metadata(&candles, filename)?;

    info!("   Paire: {}", metadata.pair);
    info!("   Timeframe: {}", metadata.timeframe);
    info!(
        "   Période: {} → {} ({} candles)",
        metadata.start_date, metadata.end_date, row_count
    );

    let db_path = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");

    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open pairs.db: {}", e))?;

    // Set busy timeout and WAL mode
    conn.busy_timeout(std::time::Duration::from_millis(5000))
        .map_err(|e| format!("Failed to set busy_timeout: {}", e))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("Failed to set WAL mode: {}", e))?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| format!("Failed to set synchronous mode: {}", e))?;

    let imported_at = Utc::now().to_rfc3339();

    info!(
        "💾 Insertion en BD: {}/{} ({} lignes)",
        metadata.pair, metadata.timeframe, row_count
    );

    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction begin error: {}", e))?;

    let mut stmt = tx
        .prepare(
            "INSERT INTO candle_data (symbol, timeframe, time, open, high, low, close, volume, imported_at, source_file)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(symbol, timeframe, time) DO UPDATE SET
                open = excluded.open,
                high = excluded.high,
                low = excluded.low,
                close = excluded.close,
                volume = excluded.volume,
                imported_at = excluded.imported_at,
                source_file = excluded.source_file"
        )
        .map_err(|e| format!("Prepare error: {}", e))?;

    info!("📋 Prepared INSERT statement for candle_data (with UPSERT)");

    for (idx, candle) in candles.iter().enumerate() {
        let dt = chrono::DateTime::<Utc>::from_timestamp(candle.timestamp, 0)
            .ok_or(format!("Invalid timestamp: {}", candle.timestamp))?;
        let time_str = dt.to_rfc3339();

        stmt.execute(rusqlite::params![
            &metadata.pair,
            &metadata.timeframe,
            &time_str,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume,
            &imported_at,
            filename,
        ])
        .map_err(|e| format!("INSERT candle_data error at row {}: {}", idx, e))?;

        if idx % 50000 == 0 && idx > 0 {
            info!("  ✓ {} candles processed", idx);
        }
    }

    drop(stmt);

    info!("✅ {} candles traités (insérés ou mis à jour)", row_count);

    // Recalculer le nombre réel de lignes pour cette paire/timeframe pour garantir l'exactitude
    let actual_count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM candle_data WHERE symbol = ? AND timeframe = ?",
        rusqlite::params![&metadata.pair, &metadata.timeframe],
        |row| row.get(0),
    ).map_err(|e| format!("COUNT error: {}", e))?;

    tx.execute(
        "INSERT INTO pair_metadata (symbol, timeframe, row_count, last_updated, last_imported_file)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(symbol, timeframe) DO UPDATE SET
            row_count = excluded.row_count,
            last_updated = excluded.last_updated,
            last_imported_file = excluded.last_imported_file",
        rusqlite::params![
            &metadata.pair,
            &metadata.timeframe,
            actual_count,
            &imported_at,
            filename,
        ],
    )
    .map_err(|e| format!("UPDATE pair_metadata error: {}", e))?;

    info!("✅ Métadonnées mises à jour");

    info!("📋 INSERT import_log entry");
    tx.execute(
        "INSERT INTO import_log (filename, symbol, timeframe, row_count, expected_row_count, status, imported_at)
         VALUES (?, ?, ?, ?, ?, 'success', ?)",
        rusqlite::params![
            filename,
            &metadata.pair,
            &metadata.timeframe,
            row_count as i32,
            row_count as i32,
            &imported_at,
        ]
    )
    .map_err(|e| format!("INSERT import_log error: {}", e))?;

    info!("✅ Import loggé");

    tx.commit()
        .map_err(|e| format!("Transaction commit error: {}", e))?;

    // REINDEX après import pour éviter la corruption d'index (rusqlite/Diesel dual-access)
    conn.execute_batch("REINDEX;")
        .map_err(|e| format!("REINDEX error: {}", e))?;

    // Conservation du fichier source (Modification demandée : ne pas supprimer)
    info!("✅ Fichier source conservé: {}", source_path);
    // fs::remove_file(source_path).map_err(...) // Suppression désactivée
    info!(
        "🎉 Import réussi: {}/{} ({} candles)",
        metadata.pair, metadata.timeframe, row_count
    );

    Ok((metadata.pair, metadata.timeframe, row_count))
}
