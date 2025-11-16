// commands/pair_importer.rs - Logique d'import des données de paires
// Module séparé pour respecter la limite de taille (pair_data_commands.rs < 200L)

use crate::db::DbPool;
use crate::services::PairDataConverter;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::{error, info};

/// Traite un fichier individuel: parse CSV → INSERT en DB → supprime CSV source
#[allow(dead_code)]
pub fn process_single_file(
    source_path: &str,
    _pool: &DbPool,
) -> Result<(String, String, usize), String> {
    // 1. Lire et normaliser le CSV
    info!("🔄 Normalisation: {}", source_path);
    let candles = PairDataConverter::read_and_normalize(source_path)?;

    if candles.is_empty() {
        return Err("Aucune donnée valide trouvée".to_string());
    }

    let row_count = candles.len();

    // 2. Extraire les métadonnées
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

    // 3. Ouvrir une connexion rusqlite directe au fichier pairs.db
    let db_path = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");

    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open pairs.db: {}", e))?;

    let imported_at = Utc::now().to_rfc3339();

    // 4. INSERT les candles en BD (bulk insert pour performance)
    info!(
        "💾 Insertion en BD: {}/{} ({} lignes)",
        metadata.pair, metadata.timeframe, row_count
    );

    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction begin error: {}", e))?;

    // Préparer le statement une fois au lieu de pour chaque ligne
    let mut stmt = tx
        .prepare(
            "INSERT INTO candle_data (symbol, timeframe, time, open, high, low, close, volume, imported_at, source_file)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .map_err(|e| format!("Prepare error: {}", e))?;

    info!("📋 Prepared INSERT statement for candle_data");

    for (idx, candle) in candles.iter().enumerate() {
        // Convertir timestamp Unix en DateTime RFC3339
        let dt = chrono::DateTime::<Utc>::from_timestamp(candle.timestamp, 0)
            .ok_or(format!("Invalid timestamp: {}", candle.timestamp))?;
        let time_str = dt.to_rfc3339();

        let res = stmt.execute(rusqlite::params![
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
        ]);

        if let Err(e) = res {
            error!("❌ INSERT candle_data error at row {}: {}", idx, e);
            return Err(format!("INSERT candle_data error at row {}: {}", idx, e));
        }

        if idx % 50000 == 0 && idx > 0 {
            info!("  ✓ {} candles processed", idx);
        }
    }

    drop(stmt); // Libérer le statement avant de continuer

    info!("✅ {} candles insérés en BD", row_count);

    // 5. Mettre à jour pair_metadata
    info!(
        "📝 INSERT/UPDATE pair_metadata for {}/{}",
        metadata.pair, metadata.timeframe
    );
    let metadata_res = tx.execute(
        "INSERT INTO pair_metadata (symbol, timeframe, row_count, last_updated, last_imported_file)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(symbol, timeframe) DO UPDATE SET
            row_count = row_count + excluded.row_count,
            last_updated = excluded.last_updated,
            last_imported_file = excluded.last_imported_file",
        rusqlite::params![
            &metadata.pair,
            &metadata.timeframe,
            row_count as i32,
            &imported_at,
            filename,
        ],
    );

    match metadata_res {
        Ok(affected) => info!("✅ Métadonnées mises à jour ({} rows affected)", affected),
        Err(e) => {
            error!("❌ UPDATE pair_metadata error: {}", e);
            return Err(format!("UPDATE pair_metadata error: {}", e));
        }
    }

    // 6. Logger l'import
    info!("📋 INSERT import_log entry");
    let log_res = tx.execute(
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
    );

    match log_res {
        Ok(affected) => info!("✅ Import loggé ({} rows affected)", affected),
        Err(e) => {
            error!("❌ INSERT import_log error: {}", e);
            return Err(format!("INSERT import_log error: {}", e));
        }
    }

    // Commit transaction
    info!("🔄 Committing transaction...");
    match tx.commit() {
        Ok(()) => info!("✅ Transaction committed successfully"),
        Err(e) => {
            error!("❌ Transaction commit error: {}", e);
            return Err(format!("Transaction commit error: {}", e));
        }
    }

    // 7. Supprimer le fichier source
    info!("🗑️  Tentative suppression: {}", source_path);
    match fs::remove_file(source_path) {
        Ok(()) => {
            info!("✅ Fichier source supprimé avec succès");
        }
        Err(e) => {
            error!("❌ Erreur suppression fichier source: {}", e);
            return Err(format!("Erreur suppression fichier source: {}", e));
        }
    }

    info!(
        "🎉 Import réussi: {}/{} ({} candles)",
        metadata.pair, metadata.timeframe, row_count
    );

    Ok((metadata.pair, metadata.timeframe, row_count))
}
