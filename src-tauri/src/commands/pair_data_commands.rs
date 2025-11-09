use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::services::{PairDataConverter, handle_duplicate};

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
#[tauri::command]
pub async fn import_pair_data(paths: Vec<String>) -> Result<ImportSummary, String> {
    println!("📥 Import de {} fichiers de paires", paths.len());
    
    let mut summary = ImportSummary {
        total_files: paths.len(),
        successful: 0,
        failed: 0,
        pairs_updated: Vec::new(),
        timeframes: Vec::new(),
        errors: Vec::new(),
    };
    
    // Créer le répertoire de destination dans le dossier de données utilisateur
    // Cela évite le hot-reload de Tauri pendant l'import
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("data")
        .join("csv");
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Erreur création répertoire: {}", e))?;
    }
    
    println!("📂 Dossier d'import: {}", data_dir.display());
    
    for path in paths {
        match process_single_file(&path, &data_dir) {
            Ok((pair, timeframe)) => {
                summary.successful += 1;
                
                if !summary.pairs_updated.contains(&pair) {
                    summary.pairs_updated.push(pair);
                }
                
                if !summary.timeframes.contains(&timeframe) {
                    summary.timeframes.push(timeframe);
                }
                
                println!("✅ Fichier importé: {}", path);
            }
            Err(e) => {
                summary.failed += 1;
                let file_name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let error_msg = format!("{}: {}", file_name, e);
                summary.errors.push(error_msg);
                eprintln!("❌ Erreur import {}: {}", path, e);
            }
        }
    }
    
    println!("📊 Import terminé: {} succès, {} échecs", summary.successful, summary.failed);
    
    Ok(summary)
}

/// Traite un fichier individuel
fn process_single_file(
    source_path: &str,
    output_dir: &Path,
) -> Result<(String, String), String> {
    // 1. Lire et normaliser le CSV
    println!("🔄 Normalisation: {}", source_path);
    let candles = PairDataConverter::read_and_normalize(source_path)?;
    
    if candles.is_empty() {
        return Err("Aucune donnée valide trouvée".to_string());
    }
    
    // 2. Extraire les métadonnées
    let filename = Path::new(source_path)
        .file_name()
        .ok_or("Nom de fichier invalide")?
        .to_str()
        .ok_or("Nom de fichier non-UTF8")?;
    
    println!("📊 Extraction métadonnées de: {}", filename);
    let metadata = PairDataConverter::extract_metadata(&candles, filename)?;
    
    println!("   Paire: {}", metadata.pair);
    println!("   Timeframe: {}", metadata.timeframe);
    println!("   Période: {} → {}", metadata.start_date, metadata.end_date);
    
    // 3. Générer le nom du fichier normalisé
    let output_filename = PairDataConverter::generate_filename(&metadata);
    let mut output_path = output_dir.join(&output_filename);
    
    // 4. Gérer les doublons (versioning)
    if output_path.exists() {
        output_path = handle_duplicate(output_dir, &output_filename)?;
    }
    
    println!("💾 Sauvegarde: {}", output_path.display());
    
    // 5. Sauvegarder le CSV normalisé
    PairDataConverter::save_normalized_csv(&candles, &output_path)?;
    
    // 6. Supprimer le fichier source
    println!("🗑️  Suppression source: {}", source_path);
    fs::remove_file(source_path)
        .map_err(|e| format!("Erreur suppression fichier source: {}", e))?;
    
    Ok((metadata.pair, metadata.timeframe))
}
