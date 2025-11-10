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
            && headers_lower.contains(&"open".to_string()) {
            return CsvFormat::MetaTrader;
        }
        
        // TradingView: time,open,high,low,close,volume
        if headers_lower.len() >= 6 
            && headers_lower[0] == "time" 
            && headers_lower[1] == "open" {
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
        use std::io::{BufRead, BufReader};
        use std::fs::File;
        
        // Lire et normaliser le fichier si format européen
        let file = File::open(path).map_err(|e| format!("Erreur ouverture: {}", e))?;
        let buf_reader = BufReader::new(file);
        let lines: Vec<String> = buf_reader.lines().collect::<Result<_, _>>().map_err(|e| e.to_string())?;
        
        if lines.is_empty() {
            return Err("Fichier vide".to_string());
        }
        
        // Les fichiers sont déjà nettoyés par csv_cleaner avant l'import
        let content = lines.join("\n");
        
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(csv::Trim::All)
            .delimiter(b';')  // Format européen: point-virgule
            .from_reader(content.as_bytes());
        
        // Lire les headers
        let headers: Vec<String> = reader.headers()
            .map_err(|e| format!("Erreur headers: {}", e))?
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        let format = Self::detect_format(&headers);
        println!("🔍 Format détecté: {:?}", format);
        println!("📋 Headers: {:?}", headers);
        
        let mut candles = Vec::new();
        let mut line_number = 1; // Commence à 1 (après header)
        
        for result in reader.records() {
            line_number += 1;
            
            // Ignorer les lignes mal formatées au lieu de faire échouer tout le fichier
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("⚠️ Ligne {} ignorée (erreur CSV): {}", line_number, e);
                    continue;
                }
            };
            
            match FormatParsers::parse_record(&record, &format, &headers) {
                Ok(candle) => candles.push(candle),
                Err(e) => eprintln!("⚠️ Ligne {} ignorée (parsing): {}", line_number, e),
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
