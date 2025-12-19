// services/csv_cleaner.rs - Nettoyeur de CSV européens
// Conforme .clinerules : < 150L, pas d'unwrap()

use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleaningReport {
    pub original_file: String,
    pub cleaned_file: String,
    pub status: String,
    pub lines_processed: usize,
    pub lines_cleaned: usize,
    pub errors: usize,
    pub warnings: Vec<String>,
}

/// Nettoie un fichier CSV avec format européen (virgules décimales)
pub fn clean_european_csv(input_path: &str, output_dir: &Path) -> Result<CleaningReport, String> {
    let input = Path::new(input_path);
    let filename = input
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Nom invalide")?;
    let output_filename = filename.replace(".csv", "_cleaned.csv");
    let output_path = output_dir.join(&output_filename);

    tracing::debug!("🧹 Nettoyage: {} → {}", input_path, output_path.display());

    let input_file = File::open(input_path).map_err(|e| format!("Ouverture: {}", e))?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(&output_path).map_err(|e| format!("Création: {}", e))?;

    let mut report = CleaningReport {
        original_file: filename.to_string(),
        cleaned_file: output_path.to_string_lossy().to_string(),
        status: "success".to_string(),
        lines_processed: 0,
        lines_cleaned: 0,
        errors: 0,
        warnings: Vec::new(),
    };

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                report.errors += 1;
                report.warnings.push(format!("L{}: {}", line_num + 1, e));
                continue;
            }
        };
        report.lines_processed += 1;

        if line_num == 0 {
            writeln!(output_file, "timestamp,open,high,low,close,volume")
                .map_err(|e| format!("Header: {}", e))?;
            continue;
        }

        match clean_line(&line) {
            Ok(cleaned) => {
                writeln!(output_file, "{}", cleaned)
                    .map_err(|e| format!("L{}: {}", line_num + 1, e))?;
                report.lines_cleaned += 1;
            }
            Err(e) => {
                report.errors += 1;
                report.warnings.push(format!("L{}: {}", line_num + 1, e));
            }
        }
    }

    // Calculer taux d'erreur : < 1% = success, sinon partial
    let error_rate = if report.lines_processed > 0 {
        (report.errors as f64 / report.lines_processed as f64) * 100.0
    } else {
        0.0
    };

    if error_rate >= 1.0 {
        report.status = "partial".to_string();
        tracing::warn!(
            "⚠️ {} lignes nettoyées ({} erreurs = {:.2}%)",
            report.lines_cleaned, report.errors, error_rate
        );
    } else if report.errors > 0 {
        tracing::info!(
            "✅ {} lignes nettoyées ({} erreurs = {:.2}% < 1%)",
            report.lines_cleaned, report.errors, error_rate
        );
    } else {
        tracing::info!("✅ {} lignes nettoyées (0 erreur)", report.lines_cleaned);
    }

    Ok(report)
}

/// Nettoie une ligne individuelle
fn clean_line(line: &str) -> Result<String, String> {
    // Détecter le délimiteur : point-virgule (nouveau format) ou virgule (ancien)
    let delimiter = if line.contains(';') { ';' } else { ',' };

    let parts: Vec<&str> = line.split(delimiter).collect();
    if parts.len() < 2 {
        return Err("Format invalide".to_string());
    }

    // Format point-virgule (nouveau) : 6 colonnes directes
    if delimiter == ';' {
        if parts.len() != 6 {
            return Err(format!("Attendu 6 colonnes, trouvé {}", parts.len()));
        }

        // Time (UTC);Open;High;Low;Close;Volume
        let timestamp = parts[0];
        let open = parts[1].replace(',', ".");
        let high = parts[2].replace(',', ".");
        let low = parts[3].replace(',', ".");
        let close = parts[4].replace(',', ".");
        let volume = parts[5].replace(',', ".");

        return Ok(format!(
            "{},{},{},{},{},{}",
            timestamp, open, high, low, close, volume
        ));
    }

    // Format virgule (ancien) : parsing complexe
    // Le timestamp est "YYYY.MM.DD HH:MM:SS" et les données OHLCV commencent après
    // Exemple: 2024.10.10 13:33:00,108.114,108.131,108.102,108.113,278.52
    // Les parts sont: ["2024", "10", "10 13:33:00", "108", "114", "108", "131", "108", "102", "108", "113", "278", "52"]

    // Chercher le timestamp : les parts qui commencent par des chiffres et contiennent ":" ou "-"
    let mut timestamp_parts = Vec::new();
    let mut data_start = 0;

    for (i, part) in parts.iter().enumerate() {
        // Le timestamp se termine quand on atteint une vraie valeur numérique (pas de ":")
        if timestamp_parts.is_empty() {
            timestamp_parts.push(*part);
        } else if part.contains(':') || (i < 3 && part.len() <= 2) {
            // Continuer le timestamp (heure, minute ou jour)
            timestamp_parts.push(*part);
        } else {
            // On a atteint les données OHLCV
            data_start = i;
            break;
        }
    }

    if timestamp_parts.is_empty() || timestamp_parts.len() < 3 {
        return Err(format!("Pas de timestamp valide dans: {}", line));
    }

    let timestamp = timestamp_parts.join(".");
    let values: Vec<&str> = parts[data_start..].to_vec();
    let ohlcv = reconstruct_ohlcv(&values)?;

    Ok(format!(
        "{},{},{},{},{},{}",
        timestamp, ohlcv[0], ohlcv[1], ohlcv[2], ohlcv[3], ohlcv[4]
    ))
}

/// Reconstruit les 5 valeurs OHLCV depuis un tableau de valeurs séparées
/// Gère le format européen où les virgules décimales créent des colonnes supplémentaires
fn reconstruct_ohlcv(values: &[&str]) -> Result<Vec<String>, String> {
    if values.is_empty() {
        return Err("Aucune valeur".to_string());
    }

    // Format standard : exactement 5 valeurs (déjà au bon format)
    if values.len() == 5 {
        return Ok(values.iter().map(|s| s.to_string()).collect());
    }

    // Format EU avec virgules décimales : 9, 10 ou 11 valeurs
    // 11 valeurs = OHLCV avec toutes les décimales (ex: 0,996,1,003,0,996,1,003,0,072)
    // 10 valeurs = volume entier sans décimale (ex: 106,401,106,401,106,401,106,401,0)
    // 9 valeurs  = high ET volume entiers (ex: 106,993,107,106,977,106,978,134,52)

    match values.len() {
        11 => {
            // Fusion systématique par paires : [0,1], [2,3], [4,5], [6,7], [8,9], reste 10
            Ok(vec![
                format!("{}.{}", values[0], values[1]),
                format!("{}.{}", values[2], values[3]),
                format!("{}.{}", values[4], values[5]),
                format!("{}.{}", values[6], values[7]),
                format!("{}.{}", values[8], values[9]),
            ])
        }
        10 => {
            // Fusion des 4 premières paires + volume entier
            Ok(vec![
                format!("{}.{}", values[0], values[1]),
                format!("{}.{}", values[2], values[3]),
                format!("{}.{}", values[4], values[5]),
                format!("{}.{}", values[6], values[7]),
                values[8].to_string(), // Volume entier (ex: "0")
            ])
        }
        9 => {
            // Cas spécial : détecter quelle valeur est entière
            // Si values[2] est un seul chiffre (1-9), c'est le high entier
            // Sinon, fusion normale
            if values[2].len() <= 2 && values[2].parse::<u32>().is_ok() {
                // High entier détecté (ex: "107")
                Ok(vec![
                    format!("{}.{}", values[0], values[1]), // Open
                    values[2].to_string(),                  // High entier
                    format!("{}.{}", values[3], values[4]), // Low
                    format!("{}.{}", values[5], values[6]), // Close
                    format!("{}.{}", values[7], values[8]), // Volume
                ])
            } else {
                // Fusion normale par paires + dernier entier
                Ok(vec![
                    format!("{}.{}", values[0], values[1]),
                    format!("{}.{}", values[2], values[3]),
                    format!("{}.{}", values[4], values[5]),
                    format!("{}.{}", values[6], values[7]),
                    values[8].to_string(),
                ])
            }
        }
        8 => {
            // 4 paires exactes (rare mais possible)
            Ok(vec![
                format!("{}.{}", values[0], values[1]),
                format!("{}.{}", values[2], values[3]),
                format!("{}.{}", values[4], values[5]),
                format!("{}.{}", values[6], values[7]),
                "0".to_string(), // Volume = 0
            ])
        }
        _ => Err(format!(
            "Format invalide: {} valeurs (attendu 5, 8-11)",
            values.len()
        )),
    }
}

/// Crée le répertoire de sortie pour les fichiers nettoyés
pub fn create_cleaned_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_local_dir()
        .ok_or("Impossible d'obtenir le dossier de données")?
        .join("volatility-analyzer")
        .join("cleaned");

    create_dir_all(&dir).map_err(|e| format!("Erreur création dossier: {}", e))?;

    Ok(dir)
}
