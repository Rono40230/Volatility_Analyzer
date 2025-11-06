// commands/import_clean_commands.rs - Import avec nettoyage automatique
// Commande unifiée qui nettoie ET importe en une seule opération

use crate::services::{clean_european_csv, create_cleaned_dir, CleaningReport};
use crate::commands::pair_data_commands::ImportSummary;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Rapport combiné de nettoyage + import
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportCleanReport {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub pairs_updated: Vec<String>,
    pub timeframes: Vec<String>,
    pub cleaning_stats: Vec<FileCleaningStats>,
    pub errors: Vec<String>,
}

/// Statistiques de nettoyage pour un fichier
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileCleaningStats {
    pub original_file: String,
    pub pair: String,
    pub timeframe: String,
    pub lines_cleaned: usize,
    pub errors: usize,
    pub error_rate: f64,
    pub status: String, // "success", "partial", "failed"
}

/// Commande Tauri : Nettoie ET importe des fichiers CSV en une seule opération
#[tauri::command]
pub async fn import_and_clean_files(paths: Vec<String>) -> Result<ImportCleanReport, String> {
    println!("📥 Import avec nettoyage automatique de {} fichiers", paths.len());
    
    let mut report = ImportCleanReport {
        total_files: paths.len(),
        successful: 0,
        failed: 0,
        pairs_updated: Vec::new(),
        timeframes: Vec::new(),
        cleaning_stats: Vec::new(),
        errors: Vec::new(),
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
        
        match process_file_with_cleaning(path, &temp_dir, &data_dir) {
            Ok((pair, timeframe, cleaning_stats)) => {
                report.successful += 1;
                
                if !report.pairs_updated.contains(&pair) {
                    report.pairs_updated.push(pair.clone());
                }
                
                if !report.timeframes.contains(&timeframe) {
                    report.timeframes.push(timeframe.clone());
                }
                
                report.cleaning_stats.push(cleaning_stats);
                
                println!("✅ Fichier importé avec succès");
            }
            Err(e) => {
                report.failed += 1;
                let file_name = Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let error_msg = format!("{}: {}", file_name, e);
                report.errors.push(error_msg);
                eprintln!("❌ Erreur: {}", e);
            }
        }
    }
    
    println!("\n📊 Import terminé: {} succès, {} échecs", report.successful, report.failed);
    
    Ok(report)
}

/// Traite un fichier : nettoyage + import + suppression du temporaire
fn process_file_with_cleaning(
    source_path: &str,
    temp_dir: &Path,
    data_dir: &Path,
) -> Result<(String, String, FileCleaningStats), String> {
    // Étape 1 : Nettoyer le fichier
    println!("  🧹 Nettoyage...");
    let cleaning_report = clean_european_csv(source_path, temp_dir)?;
    
    let cleaned_path = &cleaning_report.cleaned_file;
    
    // Calculer le taux d'erreur
    let error_rate = if cleaning_report.lines_processed > 0 {
        (cleaning_report.errors as f64 / cleaning_report.lines_processed as f64) * 100.0
    } else {
        0.0
    };
    
    // Vérifier si le nettoyage a trop d'erreurs
    if error_rate >= 5.0 {
        return Err(format!(
            "Trop d'erreurs de nettoyage ({:.2}%), fichier peut être corrompu",
            error_rate
        ));
    }
    
    println!("  ✅ {} lignes nettoyées ({} erreurs = {:.2}%)", 
             cleaning_report.lines_cleaned, cleaning_report.errors, error_rate);
    
    // Étape 2 : Importer le fichier nettoyé
    println!("  📥 Import...");
    let (pair, timeframe) = import_cleaned_file(cleaned_path, data_dir)?;
    
    println!("  ✅ Importé: {} ({})", pair, timeframe);
    
    // Étape 3 : Supprimer le fichier temporaire nettoyé
    if let Err(e) = fs::remove_file(cleaned_path) {
        eprintln!("  ⚠️  Impossible de supprimer le fichier temporaire: {}", e);
    }
    
    // Créer les stats de nettoyage
    let stats = FileCleaningStats {
        original_file: Path::new(source_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        pair: pair.clone(),
        timeframe: timeframe.clone(),
        lines_cleaned: cleaning_report.lines_cleaned,
        errors: cleaning_report.errors,
        error_rate,
        status: if error_rate >= 1.0 { "partial".to_string() } else { "success".to_string() },
    };
    
    Ok((pair, timeframe, stats))
}

/// Importe un fichier CSV nettoyé (réutilise la logique de pair_data_commands)
fn import_cleaned_file(
    cleaned_path: &str,
    output_dir: &Path,
) -> Result<(String, String), String> {
    use crate::services::PairDataConverter;
    
    // 1. Lire et normaliser le CSV
    let candles = PairDataConverter::read_and_normalize(cleaned_path)?;
    
    if candles.is_empty() {
        return Err("Aucune donnée valide trouvée".to_string());
    }
    
    // 2. Extraire les métadonnées
    let filename = Path::new(cleaned_path)
        .file_name()
        .ok_or("Nom de fichier invalide")?
        .to_str()
        .ok_or("Nom de fichier non-UTF8")?;
    
    let metadata = PairDataConverter::extract_metadata(&candles, filename)?;
    
    // 3. Générer le nom du fichier normalisé
    let output_filename = PairDataConverter::generate_filename(&metadata);
    let mut output_path = output_dir.join(&output_filename);
    
    // 4. Gérer les doublons (versioning)
    if output_path.exists() {
        output_path = handle_duplicate(output_dir, &output_filename)?;
    }
    
    // 5. Sauvegarder le CSV normalisé
    PairDataConverter::save_normalized_csv(&candles, &output_path)?;
    
    Ok((metadata.pair, metadata.timeframe))
}

/// Gère les doublons en ajoutant un suffixe de version
fn handle_duplicate(output_dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let (base, ext) = if let Some(pos) = filename.rfind('.') {
        (&filename[..pos], &filename[pos..])
    } else {
        (filename, "")
    };
    
    for version in 2..=100 {
        let new_filename = format!("{}_v{}{}", base, version, ext);
        let new_path = output_dir.join(&new_filename);
        
        if !new_path.exists() {
            println!("  ⚠️  Doublon détecté, sauvegarde comme: {}", new_filename);
            return Ok(new_path);
        }
    }
    
    Err("Trop de versions du même fichier".to_string())
}
