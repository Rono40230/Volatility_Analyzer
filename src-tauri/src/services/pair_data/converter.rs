// services/pair_data/converter.rs - Convertisseur principal
// Conforme .clinerules : < 150L, pas d'unwrap()

use super::formats::FormatParsers;
use super::metadata::MetadataExtractor;
use super::types::{CsvFormat, NormalizedCandle, PairMetadata};
use std::path::Path;

/// Convertisseur de données de paires
pub struct PairDataConverter;

impl PairDataConverter {
    /// Détecte le format du CSV en analysant les headers
    pub fn detect_format(headers: &[String]) -> CsvFormat {
        let headers_lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();

        // MetaTrader: Date,Time,Open,High,Low,Close,Volume
        if headers_lower.contains(&"date".to_string())
            && headers_lower.contains(&"time".to_string())
            && headers_lower.contains(&"open".to_string())
        {
            return CsvFormat::MetaTrader;
        }

        // TradingView: time,open,high,low,close,volume
        if headers_lower.len() >= 6 && headers_lower[0] == "time" && headers_lower[1] == "open" {
            return CsvFormat::TradingView;
        }

        // Dukascopy: Gmt time,Open,High,Low,Close,Volume
        if headers_lower.iter().any(|h| h.contains("gmt")) {
            return CsvFormat::Dukascopy;
        }

        CsvFormat::Generic
    }

    /// Lit et normalise un fichier CSV  
    pub fn read_and_normalize(path: &str) -> Result<Vec<NormalizedCandle>, String> {
        use std::fs::File;
        use std::io::{BufRead, BufReader, Seek, SeekFrom};

        // 1. Ouvrir le fichier
        let mut file = File::open(path).map_err(|e| format!("Erreur ouverture: {}", e))?;
        let mut buf_reader = BufReader::new(file);

        // 2. Lire la première ligne pour détecter le délimiteur
        let mut first_line = String::new();
        buf_reader.read_line(&mut first_line).map_err(|e| e.to_string())?;

        if first_line.trim().is_empty() {
            return Err("Fichier vide".to_string());
        }

        let delimiter = if first_line.contains(';') {
            b';' // Format européen
        } else {
            b',' // Format standard
        };

        tracing::info!(
            "🔍 Délimiteur détecté: {}",
            if delimiter == b';' { ";" } else { "," }
        );

        // 3. Réinitialiser la position du fichier au début
        file = buf_reader.into_inner(); // Récupérer le fichier sous-jacent
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        
        // Créer un nouveau BufReader pour le parsing
        let buf_reader_csv = BufReader::new(file);

        // 4. Configurer le CSV Reader pour streamer directement depuis le fichier (Memory Safe)
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(csv::Trim::All)
            .delimiter(delimiter)
            .from_reader(buf_reader_csv);

        // Lire les headers
        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| format!("Erreur headers: {}", e))?
            .iter()
            .map(|s| s.to_string())
            .collect();

        let format = Self::detect_format(&headers);
        tracing::debug!("🔍 Format détecté: {:?}", format);
        tracing::debug!("📋 Headers: {:?}", headers);

        // Pré-allouer une capacité raisonnable pour éviter les réallocations fréquentes (ex: 1 million)
        // Un fichier de 400MB contient environ 5-6M de lignes.
        let mut candles = Vec::with_capacity(1_000_000);
        
        let mut line_number = 1; // Commence à 1 (après header)

        for result in reader.records() {
            line_number += 1;

            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    // Loguer seulement 1 erreur sur 1000 pour éviter de spammer les logs
                    if line_number % 1000 == 0 {
                         tracing::warn!("⚠️ Ligne {} ignorée (erreur CSV): {}", line_number, e);
                    }
                    continue;
                }
            };

            match FormatParsers::parse_record(&record, &format, &headers) {
                Ok(candle) => candles.push(candle),
                Err(e) => tracing::warn!("⚠️ Ligne {} ignorée (parsing): {}", line_number, e),
            }
        }

        if candles.is_empty() {
            return Err("Aucune donnée valide trouvée dans le fichier".to_string());
        }

        // Trier par timestamp
        candles.sort_by_key(|c| c.timestamp);

        Ok(candles)
    }

    /// Extrait les métadonnées depuis les candles
    pub fn extract_metadata(
        candles: &[NormalizedCandle],
        original_filename: &str,
    ) -> Result<PairMetadata, String> {
        MetadataExtractor::extract(candles, original_filename)
    }

    /// Génère le nom du fichier normalisé
    pub fn generate_filename(metadata: &PairMetadata) -> String {
        MetadataExtractor::generate_filename(metadata)
    }

    /// Sauvegarde les candles normalisées dans un fichier CSV
    pub fn save_normalized_csv(
        candles: &[NormalizedCandle],
        output_path: &Path,
    ) -> Result<(), String> {
        MetadataExtractor::save_to_csv(candles, output_path)
    }
}
