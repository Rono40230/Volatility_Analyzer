use crate::services::PairDataConverter;
use chrono::Utc;
use rusqlite::Connection;

pub fn insert_pair_metadata(
    symbol: &str,
    timeframe: &str,
    row_count: i32,
    filename: &str,
) -> Result<(), String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");

    let conn =
        Connection::open(&data_dir).map_err(|e| format!("Failed to open pairs.db: {}", e))?;

    let imported_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO pair_metadata (symbol, timeframe, row_count, last_updated, last_imported_file)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(symbol, timeframe) DO UPDATE SET
            row_count = row_count + excluded.row_count,
            last_updated = excluded.last_updated,
            last_imported_file = excluded.last_imported_file",
        rusqlite::params![symbol, timeframe, row_count, &imported_at, filename],
    )
    .map_err(|e| format!("Failed to insert pair metadata: {}", e))?;

    tracing::info!("✅ Métadonnées insérées: {}/{}", symbol, timeframe);
    Ok(())
}

pub fn insert_candles_to_db(
    cleaned_file_path: &str,
    symbol: &str,
    timeframe: &str,
    filename: &str,
) -> Result<(), String> {
    let candles = PairDataConverter::read_and_normalize(cleaned_file_path)?;

    if candles.is_empty() {
        return Err("Aucune donnée valide trouvée".to_string());
    }

    tracing::info!(
        "📊 Insertion de {} candles en BD pour {}/{}",
        candles.len(),
        symbol,
        timeframe
    );

    let db_path = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");

    let mut conn =
        Connection::open(&db_path).map_err(|e| format!("Failed to open pairs.db: {}", e))?;

    let imported_at = Utc::now().to_rfc3339();

    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction begin error: {}", e))?;

    let mut stmt = tx
        .prepare(
            "INSERT INTO candle_data (symbol, timeframe, time, open, high, low, close, volume, imported_at, source_file)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .map_err(|e| format!("Prepare error: {}", e))?;

    for (idx, candle) in candles.iter().enumerate() {
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(candle.timestamp, 0)
            .ok_or(format!("Invalid timestamp: {}", candle.timestamp))?;
        let time_str = dt.format("%Y-%m-%d %H:%M:%S").to_string();

        stmt.execute(rusqlite::params![
            symbol,
            timeframe,
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
            tracing::info!("  ✓ {} candles processed", idx);
        }
    }

    drop(stmt);

    tracing::info!("✅ {} candles insérés en BD", candles.len());

    tx.commit().map_err(|e| format!("Commit error: {}", e))?;

    Ok(())
}
