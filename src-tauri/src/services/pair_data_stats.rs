// services/pair_data_stats.rs - Calcul des statistiques pour les données de paires
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct PairDataSummary {
    pub total_pairs: usize,
    pub total_files: usize,
    pub total_lines: usize,
    pub total_size_bytes: u64,
    pub date_range_start: Option<String>,
    pub date_range_end: Option<String>,
    pub last_import_date: Option<String>,
}

#[derive(Debug)]
pub struct PairFileInfo {
    pub pair: Option<String>,
    pub line_count: Option<usize>,
    pub size_bytes: u64,
    pub date_range: Option<String>,
    pub modified: String,
}

/// Compte le nombre de lignes dans un fichier CSV
pub fn count_csv_lines(path: &PathBuf) -> Option<usize> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    Some(reader.lines().count().saturating_sub(1)) // -1 pour enlever l'en-tête
}

/// Extrait la plage de dates depuis le nom de fichier
pub fn extract_date_range_from_filename(filename: &str) -> Option<String> {
    // Format attendu: SYMBOL_M1_YYYY-MM-DD-YYYY-MM-DD_YYYYMMDD.csv
    let parts: Vec<&str> = filename.split('_').collect();

    for part in parts {
        if part.contains('-') && part.len() >= 10 {
            // Vérifie si c'est une plage de dates (YYYY-MM-DD-YYYY-MM-DD)
            let date_parts: Vec<&str> = part.split('-').collect();
            if date_parts.len() >= 6 {
                let start = format!("{}-{}-{}", date_parts[0], date_parts[1], date_parts[2]);
                let end = format!("{}-{}-{}", date_parts[3], date_parts[4], date_parts[5]);
                return Some(format!("{} → {}", start, end));
            }
        }
    }

    None
}

/// Extrait la plage de dates depuis le chemin complet du fichier et retourne une chaîne formatée
pub fn extract_date_range_from_path(file_path: &Path) -> Option<String> {
    if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
        extract_date_range_from_filename(filename)
    } else {
        None
    }
}

/// Calcule les statistiques globales des données de paires
pub fn calculer_resume_paire(files: Vec<PairFileInfo>) -> PairDataSummary {
    if files.is_empty() {
        return PairDataSummary {
            total_pairs: 0,
            total_files: 0,
            total_lines: 0,
            total_size_bytes: 0,
            date_range_start: None,
            date_range_end: None,
            last_import_date: None,
        };
    }

    let mut unique_pairs = HashSet::new();
    let mut total_lines = 0;
    let mut total_size = 0;
    let mut all_dates = Vec::new();
    let mut latest_modified: Option<String> = None;

    for file in &files {
        if let Some(pair) = &file.pair {
            unique_pairs.insert(pair.clone());
        }

        if let Some(count) = file.line_count {
            total_lines += count;
        }

        total_size += file.size_bytes;

        // Extraire les dates de la plage
        if let Some(range) = &file.date_range {
            let dates: Vec<&str> = range.split(" → ").collect();
            if let Some(start) = dates.first() {
                all_dates.push(start.to_string());
            }
            if let Some(end) = dates.get(1) {
                all_dates.push(end.to_string());
            }
        }

        // Trouver le dernier fichier modifié
        if latest_modified.is_none() || file.modified > latest_modified.clone().unwrap_or_default()
        {
            latest_modified = Some(file.modified.clone());
        }
    }

    // Trier les dates pour trouver min/max
    all_dates.sort();
    let date_range_start = all_dates.first().cloned();
    let date_range_end = all_dates.last().cloned();

    PairDataSummary {
        total_pairs: unique_pairs.len(),
        total_files: files.len(),
        total_lines,
        total_size_bytes: total_size,
        date_range_start,
        date_range_end,
        last_import_date: latest_modified,
    }
}
