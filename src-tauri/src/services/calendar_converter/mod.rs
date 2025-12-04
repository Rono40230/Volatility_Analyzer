mod normalizer;

pub use normalizer::{cell_to_string, normalize_date};

use anyhow::{anyhow, Context, Result};
use calamine::{open_workbook, Reader, Xlsx};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

/// Événement économique parsé
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    pub date: String,
    pub time: String,
    pub currency: String,
    pub event: String,
    pub impact: String,
}

/// Résultat de conversion
#[derive(Debug)]
pub struct ConversionResult {
    pub events: Vec<ParsedEvent>,
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
        let extension = path
            .extension()
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
        let file = File::open(input_path).context("Impossible d'ouvrir le fichier CSV")?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);

        let mut events = Vec::new();
        let mut total_read = 0;

        for result in csv_reader.records() {
            total_read += 1;
            let record = result.context("Erreur de lecture d'une ligne CSV")?;

            if record.len() < 5 {
                continue;
            }

            let date = record.get(0).unwrap_or("").trim();
            let time = record.get(1).unwrap_or("").trim();
            let currency = record.get(2).unwrap_or("").trim();
            let impact = record.get(3).unwrap_or("").trim();
            let event_name = record.get(4).unwrap_or("").trim();

            if impact != "M" && impact != "H" {
                continue;
            }

            let impact_normalized = match impact {
                "M" => "MEDIUM",
                "H" => "HIGH",
                _ => continue,
            };

            let date_normalized = normalize_date(date)?;

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
        let mut workbook: Xlsx<_> =
            open_workbook(input_path).context("Impossible d'ouvrir le fichier Excel")?;

        let sheet_name = workbook
            .sheet_names()
            .first()
            .ok_or_else(|| anyhow!("Le fichier Excel ne contient aucune feuille"))?
            .clone();

        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| anyhow!("Impossible de lire la feuille {}: {}", sheet_name, e))?;

        let mut events = Vec::new();
        let mut total_read = 0;

        for row in range.rows() {
            total_read += 1;

            if row.len() < 5 {
                continue;
            }

            let date = cell_to_string(&row[0]);
            let time = cell_to_string(&row[1]);
            let currency = cell_to_string(&row[2]);
            let impact = cell_to_string(&row[3]);
            let event_name = cell_to_string(&row[4]);

            if impact.trim() != "M" && impact.trim() != "H" {
                continue;
            }

            let impact_normalized = match impact.trim() {
                "M" => "MEDIUM",
                "H" => "HIGH",
                _ => continue,
            };

            let date_normalized = normalize_date(&date)?;

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

    /// Sauvegarde les événements en CSV standardisé
    pub fn save_to_csv(events: &[ParsedEvent], output_path: &str) -> Result<()> {
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent).context("Impossible de créer le dossier de destination")?;
        }

        let mut file =
            File::create(output_path).context("Impossible de créer le fichier de sortie")?;

        writeln!(
            file,
            "Date,Time,Currency,Event,Impact,Actual,Forecast,Previous"
        )?;

        for event in events {
            writeln!(
                file,
                "{},{},{},{},{},,,",
                event.date, event.time, event.currency, event.event, event.impact
            )?;
        }

        Ok(())
    }

    /// Obtient le chemin de sauvegarde avec la période couverte
    pub fn get_standard_save_path(events: &[ParsedEvent]) -> Result<String> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Impossible de trouver le dossier de données local"))?;

        let app_dir = data_dir.join("volatility-analyzer");
        fs::create_dir_all(&app_dir).context("Impossible de créer le dossier de l'application")?;

        let filename = if events.is_empty() {
            "calendar.csv".to_string()
        } else {
            let mut dates: Vec<&str> = events.iter().map(|e| e.date.as_str()).collect();
            dates.sort();

            let first_date = dates.first().unwrap_or(&"unknown");
            let last_date = dates.last().unwrap_or(&"unknown");

            format!("calendar_{}_{}.csv", first_date, last_date)
        };

        let calendar_path = app_dir.join(filename);
        calendar_path
            .to_str()
            .ok_or_else(|| anyhow!("Chemin invalide"))
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_event_structure() {
        let event = ParsedEvent {
            date: "2025-01-15".to_string(),
            time: "14:30".to_string(),
            currency: "EUR".to_string(),
            event: "ECB Interest Rate".to_string(),
            impact: "HIGH".to_string(),
        };
        assert_eq!(event.date, "2025-01-15");
        assert_eq!(event.impact, "HIGH");
    }

    #[test]
    fn test_get_standard_save_path() {
        let path = CalendarConverter::get_standard_save_path(&[]);
        assert!(path
            .as_ref()
            .ok()
            .map(|p| p.contains("calendar.csv"))
            .unwrap_or(false));
    }

    #[test]
    fn test_save_to_csv() {
        let events = vec![ParsedEvent {
            date: "2025-01-15".to_string(),
            time: "14:30".to_string(),
            currency: "EUR".to_string(),
            event: "ECB Rate".to_string(),
            impact: "HIGH".to_string(),
        }];
        let temp_path = "/tmp/test_calendar.csv";
        assert!(CalendarConverter::save_to_csv(&events, temp_path).is_ok());
        let _ = std::fs::remove_file(temp_path);
    }
}
