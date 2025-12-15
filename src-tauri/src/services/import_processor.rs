// services/import_processor.rs - Logique de traitement pour l'import unifié
// Conforme .clinerules : < 300L, pas d'unwrap()

use crate::services::{clean_european_csv, PairDataConverter};
use std::path::{Path, PathBuf};

/// Résultat du traitement d'un fichier
pub struct ProcessResult {
    pub pair: String,
    pub timeframe: String,
    pub lines_cleaned: usize,
    pub errors: usize,
    pub error_rate: f64,
    pub cleaned_file_path: String, // Chemin du fichier nettoyé
}

/// Gère les doublons en ajoutant un suffixe de version (fonction utilitaire réutilisable)
pub fn handle_duplicate(output_dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let (base, ext) = if let Some(pos) = filename.rfind('.') {
        (&filename[..pos], &filename[pos..])
    } else {
        (filename, "")
    };

    for version in 2..=100 {
        let new_filename = format!("{}_v{}{}", base, version, ext);
        let new_path = output_dir.join(&new_filename);

        if !new_path.exists() {
            tracing::warn!("⚠️  Doublon détecté, sauvegarde comme: {}", new_filename);
            return Ok(new_path);
        }
    }

    Err("Trop de versions du même fichier".to_string())
}

/// Traite un fichier : nettoyage + import + suppression du temporaire
pub fn process_file_with_cleaning(
    source_path: &str,
    temp_dir: &Path,
    data_dir: &Path,
) -> Result<ProcessResult, String> {
    tracing::info!("🧹 Début du nettoyage du fichier");
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

    tracing::info!(
        "✅ {} lignes nettoyées ({} erreurs = {:.2}%)",
        cleaning_report.lines_cleaned,
        cleaning_report.errors,
        error_rate
    );

    tracing::info!("📥 Début de l'import du fichier");
    let (pair, timeframe) = import_cleaned_file(cleaned_path, data_dir)?;

    tracing::info!("✅ Importé: {} ({})", pair, timeframe);

    // NOTE: Ne pas supprimer le fichier nettoyé ici
    // Il sera inséré en BD par import_clean_commands.rs, puis supprimé après

    Ok(ProcessResult {
        pair,
        timeframe,
        lines_cleaned: cleaning_report.lines_cleaned,
        errors: cleaning_report.errors,
        error_rate,
        cleaned_file_path: cleaned_path.to_string(), // Retourner le chemin du fichier nettoyé
    })
}

/// Importe un fichier CSV nettoyé (réutilise la logique de pair_data_commands)
fn import_cleaned_file(cleaned_path: &str, output_dir: &Path) -> Result<(String, String), String> {
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
