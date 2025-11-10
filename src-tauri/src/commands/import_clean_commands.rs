// commands/import_clean_commands.rs - Import avec nettoyage automatique
// Commande unifiée qui nettoie ET importe en une seule opération

use crate::services::{create_cleaned_dir, process_file_with_cleaning, ProcessResult, PairDataConverter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::Utc;
use rusqlite::Connection;

/// Rapport combiné de nettoyage + import
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportCleanReport {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<ImportCleanResult>,
}

/// Résultat pour un fichier individuel
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportCleanResult {
    pub original_file: String,
    pub import_status: String,
    pub lines_imported: usize,
    pub cleaning_stats: Option<FileCleaningStats>,
    pub error_message: Option<String>,
}

/// Statistiques de nettoyage pour un fichier
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileCleaningStats {
    pub lines_processed: usize,
    pub lines_cleaned: usize,
    pub errors: usize,
    pub warnings: Vec<String>,
}

/// Commande Tauri : Nettoie ET importe des fichiers CSV en une seule opération
#[tauri::command]
pub async fn import_and_clean_files(paths: Vec<String>) -> Result<ImportCleanReport, String> {
    println!("📥 Import avec nettoyage automatique de {} fichiers", paths.len());
    
    let mut report = ImportCleanReport {
        total_files: paths.len(),
        successful: 0,
        failed: 0,
        results: Vec::new(),
    };
    
    // Créer le dossier de destination pour les fichiers importés
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("data")
        .join("csv");
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Erreur création répertoire: {}", e))?;
    }
    
    // Créer un dossier temporaire pour les fichiers nettoyés
    let temp_dir = create_cleaned_dir()?;
    
    println!("📂 Dossier de destination: {}", data_dir.display());
    println!("🧹 Dossier temporaire: {}", temp_dir.display());
    
    // Traiter chaque fichier
    for (index, path) in paths.iter().enumerate() {
        println!("\n[{}/{}] Traitement: {}", index + 1, paths.len(), path);
        
        let result = process_single_file(path, &temp_dir, &data_dir);
        
        match &result.import_status as &str {
            "success" => report.successful += 1,
            _ => report.failed += 1,
        }
        
        report.results.push(result);
    }
    
    println!("\n📊 Import terminé: {} succès, {} échecs", report.successful, report.failed);
    
    Ok(report)
}

/// Traite un fichier individuel et retourne un résultat structuré
fn process_single_file(
    source_path: &str,
    temp_dir: &Path,
    data_dir: &Path,
) -> ImportCleanResult {
    let file_name = Path::new(source_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    match process_file_with_cleaning(source_path, temp_dir, data_dir) {
        Ok(ProcessResult { pair, timeframe, lines_cleaned, errors, error_rate, cleaned_file_path }) => {
            println!("✅ Fichier importé avec succès: {} ({})", pair, timeframe);
            
            // 1. Insérer les candles en BD
            if let Err(e) = insert_candles_to_db(&cleaned_file_path, &pair, &timeframe, &file_name) {
                eprintln!("⚠️  Erreur insertion candles: {}", e);
                // Continue anyway - metadata is still useful
            }
            
            // 2. Insérer les métadonnées dans pair_metadata
            if let Err(e) = insert_pair_metadata(&pair, &timeframe, lines_cleaned as i32, &file_name) {
                eprintln!("⚠️  Erreur insertion métadonnées: {}", e);
            }
            
            // 3. Supprimer le fichier temporaire nettoyé après insertion
            if let Err(e) = fs::remove_file(&cleaned_file_path) {
                eprintln!("  ⚠️  Impossible de supprimer le fichier temporaire: {}", e);
            }
            
            ImportCleanResult {
                original_file: file_name,
                import_status: if error_rate >= 1.0 { "partial".to_string() } else { "success".to_string() },
                lines_imported: lines_cleaned,
                cleaning_stats: Some(FileCleaningStats {
                    lines_processed: lines_cleaned + errors,
                    lines_cleaned,
                    errors,
                    warnings: Vec::new(),
                }),
                error_message: None,
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur: {}", e);
            
            ImportCleanResult {
                original_file: file_name,
                import_status: "failed".to_string(),
                lines_imported: 0,
                cleaning_stats: None,
                error_message: Some(e),
            }
        }
    }
}

/// Insère ou met à jour les métadonnées dans pair_metadata
fn insert_pair_metadata(symbol: &str, timeframe: &str, row_count: i32, filename: &str) -> Result<(), String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");
    
    let conn = Connection::open(&data_dir)
        .map_err(|e| format!("Failed to open pairs.db: {}", e))?;
    
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
    
    println!("✅ Métadonnées insérées: {}/{}", symbol, timeframe);
    Ok(())
}

/// Insère les candles dans candle_data table
fn insert_candles_to_db(cleaned_file_path: &str, symbol: &str, timeframe: &str, filename: &str) -> Result<(), String> {
    // 1. Parser les candles du fichier nettoyé
    let candles = PairDataConverter::read_and_normalize(cleaned_file_path)?;
    
    if candles.is_empty() {
        return Err("Aucune donnée valide trouvée".to_string());
    }
    
    println!("📊 Insertion de {} candles en BD pour {}/{}", candles.len(), symbol, timeframe);
    
    // 2. Ouvrir la BD pairs.db
    let db_path = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("pairs.db");
    
    let mut conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open pairs.db: {}", e))?;
    
    let imported_at = Utc::now().to_rfc3339();
    
    // 3. Démarrer une transaction pour performance
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
    
    println!("📋 Prepared INSERT statement for candle_data");
    
    // 4. Insérer chaque candle
    for (idx, candle) in candles.iter().enumerate() {
        // Convertir timestamp Unix en DateTime RFC3339
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(candle.timestamp, 0)
            .ok_or(format!("Invalid timestamp: {}", candle.timestamp))?;
        let time_str = dt.to_rfc3339();
        
        let res = stmt.execute(rusqlite::params![
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
        ]);
        
        if let Err(e) = res {
            eprintln!("❌ INSERT candle_data error at row {}: {}", idx, e);
            return Err(format!("INSERT candle_data error at row {}: {}", idx, e));
        }
        
        if idx % 50000 == 0 && idx > 0 {
            println!("  ✓ {} candles processed", idx);
        }
    }
    
    drop(stmt); // Libérer le statement avant de continuer
    
    println!("✅ {} candles insérés en BD", candles.len());
    
    // 5. Commit la transaction
    tx.commit()
        .map_err(|e| format!("Commit error: {}", e))?;
    
    Ok(())
}

