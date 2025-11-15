use std::path::Path;
use std::fs;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::services::PairDataConverter;
use crate::db::DbPool;
use chrono::Utc;
use tracing::{info, error};

/// État Tauri pour la DB paires (stockage des données de trading)
pub struct PairDataState {
    #[allow(dead_code)]
    pub pool: Mutex<Option<DbPool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub pairs_updated: Vec<String>,
    pub timeframes: Vec<String>,
    pub errors: Vec<String>,
}

/// Commande d'import multi-fichiers de données de paires
/// NOUVEAU: Au lieu de sauvegarder CSV, insère directement dans pairs.db
#[tauri::command]
pub async fn import_pair_data(
    state: tauri::State<'_, PairDataState>,
    paths: Vec<String>,
) -> Result<ImportSummary, String> {
    info!("📥 ========== DÉBUT IMPORT PAIR DATA ==========");
    info!("📥 Import de {} fichiers de paires vers BD", paths.len());
    for (idx, path) in paths.iter().enumerate() {
        info!("   [{}] {}", idx + 1, path);
    }
    
    let mut summary = ImportSummary {
        total_files: paths.len(),
        successful: 0,
        failed: 0,
        pairs_updated: Vec::new(),
        timeframes: Vec::new(),
        errors: Vec::new(),
    };
    
    // Obtenir le pool de la DB paires
    info!("🔐 Tentative d'accès au pool DB paires...");
    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")?;
        match pool_opt.clone() {
            Some(p) => {
                info!("✅ Pool DB obtenu avec succès");
                p
            },
            None => {
                error!("❌ ERREUR CRITIQUE: Pool DB non initialisé!");
                return Err("DB pool not initialized".to_string());
            }
        }
    };
    
    for (file_idx, path) in paths.into_iter().enumerate() {
        info!("🔄 Traitement fichier [{}/{}]: {}", file_idx + 1, summary.total_files, path);
        
        match process_single_file(&path, &pool) {
            Ok((pair, timeframe, row_count)) => {
                summary.successful += 1;
                
                if !summary.pairs_updated.contains(&pair) {
                    summary.pairs_updated.push(pair.clone());
                }
                
                if !summary.timeframes.contains(&timeframe) {
                    summary.timeframes.push(timeframe.clone());
                }
                
                info!("✅ [{}/{}] Fichier importé avec succès: {} ({} lignes)", file_idx + 1, summary.total_files, path, row_count);
            }
            Err(e) => {
                summary.failed += 1;
                let file_name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let error_msg = format!("{}: {}", file_name, e);
                summary.errors.push(error_msg);
                error!("❌ [{}/{}] Erreur import {}: {}", file_idx + 1, summary.total_files, path, e);
            }
        }
    }
    
    info!("📊 ========== IMPORT TERMINÉ ==========");
    info!("📊 Résumé final: {} succès, {} échecs sur {} fichiers", summary.successful, summary.failed, summary.total_files);
    info!("📊 Paires mises à jour: {:?}", summary.pairs_updated);
    info!("📊 Timeframes: {:?}", summary.timeframes);
    if !summary.errors.is_empty() {
        error!("📊 Erreurs rencontrées:");
        for (idx, err) in summary.errors.iter().enumerate() {
            error!("   [{}] {}", idx + 1, err);
        }
    }
    
    Ok(summary)
}

/// Traite un fichier individuel: parse CSV → INSERT en DB → supprime CSV source
fn process_single_file(
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
    info!("   Période: {} → {} ({} candles)", metadata.start_date, metadata.end_date, row_count);
    
    // 3. Ouvrir une connexion rusqlite directe au fichier pairs.db
    let db_path = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");
    
    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open pairs.db: {}", e))?;
    
    let imported_at = Utc::now().to_rfc3339();
    
    // 4. INSERT les candles en BD (bulk insert pour performance)
    info!("💾 Insertion en BD: {}/{} ({} lignes)", metadata.pair, metadata.timeframe, row_count);
    
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
    info!("📝 INSERT/UPDATE pair_metadata for {}/{}", metadata.pair, metadata.timeframe);
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
        ]
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
    
    info!("🎉 Import réussi: {}/{} ({} candles)", metadata.pair, metadata.timeframe, row_count);
    
    Ok((metadata.pair, metadata.timeframe, row_count))
}
