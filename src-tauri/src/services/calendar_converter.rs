//! Service de conversion automatique de calendriers économiques
//! Supporte .csv et .xlsx, filtre les événements M/H, normalise les dates

use calamine::{open_workbook, Reader, Xlsx};
use chrono::NaiveDate;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;
use anyhow::{anyhow, Context, Result};

/// Événement économique parsé
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    pub date: String,      // Format: YYYY-MM-DD
    pub time: String,      // Format: HH:MM
    pub currency: String,  // Ex: EUR, USD
    pub event: String,     // Description
    pub impact: String,    // HIGH ou MEDIUM
}

/// Résultat de conversion
#[derive(Debug)]
pub struct ConversionResult {
    pub events: Vec<ParsedEvent>,
    /// Nombre total d'événements lus du fichier source (avant filtrage)
    /// NOTE: Ce champ n'est pas actuellement utilisé en interne mais reste public
    /// pour les clients qui souhaiteraient l'exploiter
    #[allow(dead_code)]
    pub total_read: usize,
    pub total_filtered: usize,
}

/// Convertisseur de calendrier économique
pub struct CalendarConverter;

impl CalendarConverter {
    /// Convertit un fichier .csv ou .xlsx en événements filtrés
    pub fn convert_file(input_path: &str) -> Result<ConversionResult> {
        let path = Path::new(input_path);
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("Impossible de déterminer l'extension du fichier"))?;

        match extension.to_lowercase().as_str() {
            "csv" => Self::convert_csv(input_path),
            "xlsx" | "xls" => Self::convert_excel(input_path),
            _ => Err(anyhow!("Format de fichier non supporté: {}", extension)),
        }
    }

    /// Convertit un fichier CSV
    fn convert_csv(input_path: &str) -> Result<ConversionResult> {
        let file = File::open(input_path)
            .context("Impossible d'ouvrir le fichier CSV")?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);

        let mut events = Vec::new();
        let mut total_read = 0;

        for result in csv_reader.records() {
            total_read += 1;
            let record = result.context("Erreur de lecture d'une ligne CSV")?;

            // Colonnes attendues: Date, Time, Currency, Impact, Event (0-4)
            if record.len() < 5 {
                continue; // Ligne invalide
            }

            let date = record.get(0).unwrap_or("").trim();
            let time = record.get(1).unwrap_or("").trim();
            let currency = record.get(2).unwrap_or("").trim();
            let impact = record.get(3).unwrap_or("").trim();
            let event_name = record.get(4).unwrap_or("").trim();

            // Filtrer uniquement M (Medium) et H (High)
            if impact != "M" && impact != "H" {
                continue;
            }

            // Convertir impact: M → MEDIUM, H → HIGH
            let impact_normalized = match impact {
                "M" => "MEDIUM",
                "H" => "HIGH",
                _ => continue,
            };

            // Normaliser la date: 2007/01/01 → 2007-01-01
            let date_normalized = Self::normalize_date(date)?;

            events.push(ParsedEvent {
                date: date_normalized,
                time: time.to_string(),
                currency: currency.to_string(),
                event: event_name.to_string(),
                impact: impact_normalized.to_string(),
            });
        }

        Ok(ConversionResult {
            total_filtered: events.len(),
            total_read,
            events,
        })
    }

    /// Convertit un fichier Excel (.xlsx)
    fn convert_excel(input_path: &str) -> Result<ConversionResult> {
        let mut workbook: Xlsx<_> = open_workbook(input_path)
            .context("Impossible d'ouvrir le fichier Excel")?;

        // Prendre la première feuille
        let sheet_name = workbook.sheet_names()
            .first()
            .ok_or_else(|| anyhow!("Le fichier Excel ne contient aucune feuille"))?
            .clone();

        let range = workbook.worksheet_range(&sheet_name)
            .map_err(|e| anyhow!("Impossible de lire la feuille {}: {}", sheet_name, e))?;

        let mut events = Vec::new();
        let mut total_read = 0;

        for row in range.rows() {
            total_read += 1;

            // Colonnes: A=Date, B=Time, C=Currency, D=Impact, E=Event
            if row.len() < 5 {
                continue;
            }

            let date = Self::cell_to_string(&row[0]);
            let time = Self::cell_to_string(&row[1]);
            let currency = Self::cell_to_string(&row[2]);
            let impact = Self::cell_to_string(&row[3]);
            let event_name = Self::cell_to_string(&row[4]);

            // Filtrer uniquement M et H
            if impact.trim() != "M" && impact.trim() != "H" {
                continue;
            }

            let impact_normalized = match impact.trim() {
                "M" => "MEDIUM",
                "H" => "HIGH",
                _ => continue,
            };

            let date_normalized = Self::normalize_date(&date)?;

            events.push(ParsedEvent {
                date: date_normalized,
                time: time.trim().to_string(),
                currency: currency.trim().to_string(),
                event: event_name.trim().to_string(),
                impact: impact_normalized.to_string(),
            });
        }

        Ok(ConversionResult {
            total_filtered: events.len(),
            total_read,
            events,
        })
    }

    /// Convertit une cellule Excel en String
    fn cell_to_string(cell: &calamine::Data) -> String {
        match cell {
            calamine::Data::String(s) => s.clone(),
            calamine::Data::Int(i) => i.to_string(),
            calamine::Data::Float(f) => f.to_string(),
            calamine::Data::Bool(b) => b.to_string(),
            calamine::Data::DateTime(dt) => dt.to_string(),
            calamine::Data::DateTimeIso(dt) => dt.clone(),
            calamine::Data::DurationIso(d) => d.clone(),
            calamine::Data::Error(_) => String::new(),
            calamine::Data::Empty => String::new(),
        }
    }

    /// Normalise la date: 2007/01/01 → 2007-01-01
    fn normalize_date(date: &str) -> Result<String> {
        if date.contains('/') {
            // Format: 2007/01/01
            let parts: Vec<&str> = date.split('/').collect();
            if parts.len() == 3 {
                return Ok(format!("{}-{:02}-{:02}", 
                    parts[0], 
                    parts[1].parse::<u32>().unwrap_or(1),
                    parts[2].parse::<u32>().unwrap_or(1)
                ));
            }
        } else if date.contains('-') {
            // Déjà au bon format
            return Ok(date.to_string());
        }

        // Tenter un parsing avec chrono
        if let Ok(parsed) = NaiveDate::parse_from_str(date, "%Y/%m/%d") {
            return Ok(parsed.format("%Y-%m-%d").to_string());
        }
        if let Ok(parsed) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            return Ok(parsed.format("%Y-%m-%d").to_string());
        }

        Err(anyhow!("Format de date invalide: {}", date))
    }

    /// Sauvegarde les événements en CSV standardisé
    pub fn save_to_csv(events: &[ParsedEvent], output_path: &str) -> Result<()> {
        // Créer les dossiers parents si nécessaire
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent)
                .context("Impossible de créer le dossier de destination")?;
        }

        let mut file = File::create(output_path)
            .context("Impossible de créer le fichier de sortie")?;

        // En-tête CSV
        writeln!(file, "Date,Time,Currency,Event,Impact,Actual,Forecast,Previous")?;

        // Écrire les événements
        for event in events {
            writeln!(
                file,
                "{},{},{},{},{},,,",
                event.date,
                event.time,
                event.currency,
                event.event,
                event.impact
            )?;
        }

        Ok(())
    }

    /// Obtient le chemin de sauvegarde avec la période couverte
    pub fn get_standard_save_path(events: &[ParsedEvent]) -> Result<String> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Impossible de trouver le dossier de données local"))?;

        let app_dir = data_dir.join("volatility-analyzer");
        fs::create_dir_all(&app_dir)
            .context("Impossible de créer le dossier de l'application")?;

        // Déterminer la période couverte (première et dernière date)
        let filename = if events.is_empty() {
            "calendar.csv".to_string()
        } else {
            // Trouver min et max des dates
            let mut dates: Vec<&str> = events.iter().map(|e| e.date.as_str()).collect();
            dates.sort();
            
            let first_date = dates.first().unwrap_or(&"unknown");
            let last_date = dates.last().unwrap_or(&"unknown");
            
            format!("calendar_{}_{}.csv", first_date, last_date)
        };

        let calendar_path = app_dir.join(filename);
        calendar_path.to_str()
            .ok_or_else(|| anyhow!("Chemin invalide"))
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_date() {
        assert_eq!(
            CalendarConverter::normalize_date("2007/01/01").unwrap(),
            "2007-01-01"
        );
        assert_eq!(
            CalendarConverter::normalize_date("2025/12/31").unwrap(),
            "2025-12-31"
        );
        assert_eq!(
            CalendarConverter::normalize_date("2007-01-01").unwrap(),
            "2007-01-01"
        );
    }
}
