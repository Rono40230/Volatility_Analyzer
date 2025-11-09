// services/pair_data/metadata.rs - Extraction et génération de métadonnées
// Conforme .clinerules : < 150L, pas d'unwrap()

use super::types::{NormalizedCandle, PairMetadata};
use chrono::{DateTime, Utc};
use std::path::Path;

/// Extracteur de métadonnées de paires
pub(super) struct MetadataExtractor;

impl MetadataExtractor {
    /// Extrait les métadonnées depuis les candles et le nom de fichier
    pub(super) fn extract(
        candles: &[NormalizedCandle],
        original_filename: &str,
    ) -> Result<PairMetadata, String> {
        if candles.is_empty() {
            return Err("Pas de données pour extraire les métadonnées".to_string());
        }
        
        // Extraire la paire depuis le nom de fichier
        let pair = Self::extract_pair_from_filename(original_filename);
        
        // Détecter le timeframe
        let timeframe = Self::detect_timeframe(candles)?;
        
        // Période
        let start_timestamp = candles.first().ok_or("Empty candles vector")?.timestamp;
        let end_timestamp = candles.last().ok_or("Empty candles vector")?.timestamp;
        
        let start_date = DateTime::from_timestamp(start_timestamp, 0)
            .ok_or("Timestamp invalide")?
            .format("%Y-%m-%d")
            .to_string();
        let end_date = DateTime::from_timestamp(end_timestamp, 0)
            .ok_or("Timestamp invalide")?
            .format("%Y-%m-%d")
            .to_string();
        
        // Date d'import
        let import_date = Utc::now().format("%Y%m%d").to_string();
        
        Ok(PairMetadata {
            pair,
            timeframe,
            start_date,
            end_date,
            import_date,
        })
    }
    
    /// Extrait le nom de la paire depuis le nom de fichier
    fn extract_pair_from_filename(filename: &str) -> String {
        // Enlever l'extension
        let name = filename.replace(".csv", "").replace(".CSV", "");
        
        // Extraire les majuscules (paires forex typiquement en majuscules)
        let uppercase: String = name.chars().filter(|c| c.is_uppercase()).collect();
        
        if uppercase.len() >= 6 {
            // Probablement une paire forex (EURUSD, GBPUSD, etc.)
            return uppercase[..6].to_string();
        }
        
        // Sinon prendre le nom du fichier
        name.to_uppercase()
    }
    
    /// Détecte le timeframe en analysant l'écart entre les bougies
    fn detect_timeframe(candles: &[NormalizedCandle]) -> Result<String, String> {
        if candles.len() < 2 {
            return Err("Pas assez de données pour détecter le timeframe".to_string());
        }
        
        // Calculer l'écart moyen entre les 10 premières bougies
        let mut diffs = Vec::new();
        for i in 0..10.min(candles.len() - 1) {
            diffs.push(candles[i + 1].timestamp - candles[i].timestamp);
        }
        
        let avg_diff = diffs.iter().sum::<i64>() / diffs.len() as i64;
        
        // Mapper l'écart au timeframe
        let timeframe = match avg_diff {
            0..=90 => "M1",          // 1 minute
            91..=360 => "M5",        // 5 minutes
            361..=1200 => "M15",     // 15 minutes
            1201..=2400 => "M30",    // 30 minutes
            2401..=4500 => "H1",     // 1 heure
            4501..=18000 => "H4",    // 4 heures
            18001..=100000 => "D1",  // 1 jour
            _ => "W1",               // 1 semaine ou plus
        };
        
        Ok(timeframe.to_string())
    }
    
    /// Génère le nom du fichier normalisé
    #[allow(clippy::format_in_format_args)]
    pub(super) fn generate_filename(metadata: &PairMetadata) -> String {
        format!(
            "{}_{}_{}_{}.csv",
            metadata.pair,
            metadata.timeframe,
            format!("{}-{}", metadata.start_date, metadata.end_date),
            metadata.import_date
        )
    }
    
    /// Sauvegarde les candles normalisées dans un fichier CSV
    pub(super) fn save_to_csv(
        candles: &[NormalizedCandle],
        output_path: &Path,
    ) -> Result<(), String> {
        let mut writer = csv::Writer::from_path(output_path)
            .map_err(|e| format!("Erreur création CSV: {}", e))?;
        
        // Écrire les headers
        writer.write_record(["timestamp", "open", "high", "low", "close", "volume"])
            .map_err(|e| format!("Erreur écriture headers: {}", e))?;
        
        // Écrire les données
        for candle in candles {
            writer.write_record(&[
                candle.timestamp.to_string(),
                candle.open.to_string(),
                candle.high.to_string(),
                candle.low.to_string(),
                candle.close.to_string(),
                candle.volume.to_string(),
            ]).map_err(|e| format!("Erreur écriture ligne: {}", e))?;
        }
        
        writer.flush().map_err(|e| format!("Erreur flush: {}", e))?;
        
        Ok(())
    }
}
