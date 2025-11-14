// calendar_parser.rs - Parses calendar files in both formats (NEW and OLD)
// Handles dual-format detection and event normalization

use csv::ReaderBuilder;
use std::fs;
use std::io::BufRead;

pub struct ParsedEvent {
    pub date: String,
    pub time: String,
    pub currency: String,
    pub impact: String,
    pub description: String,
}

/// Détecte et analyse un fichier de calendrier
pub fn parse_calendar_file(path: &str) -> Result<Vec<ParsedEvent>, String> {
    let file_path = std::path::Path::new(path);
    if !file_path.exists() {
        return Err(format!("Fichier non trouvé: {}", path));
    }

    // Lire la première ligne pour détecter le format
    let first_line = {
        let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = std::io::BufReader::new(file);
        if let Some(Ok(line)) = reader.lines().next() {
            line
        } else {
            return Err("CSV file is empty".to_string());
        }
    };

    let fields: Vec<&str> = first_line.split(',').collect();

    let is_format_new = fields.len() >= 8
        && fields[0].parse::<i32>().is_ok()
        && fields[0].len() == 4
        && fields[1].parse::<i32>().is_ok();

    let is_format_old = fields.len() >= 5 && fields[0].contains("/") && fields[1].contains(":");

    if is_format_new {
        parse_format_new(path)
    } else if is_format_old {
        parse_format_old(path)
    } else {
        Err("Format de CSV non reconnu".to_string())
    }
}

fn parse_format_new(path: &str) -> Result<Vec<ParsedEvent>, String> {
    let file = fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = std::io::BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(false).from_reader(reader);

    let mut events = Vec::new();

    for result in csv_reader.records().skip(1) {
        let record = result.map_err(|e| format!("CSV error: {}", e))?;
        if record.len() < 8 {
            continue;
        }

        let year = record.get(0).unwrap_or("").parse::<i32>().unwrap_or(0);
        let month = record.get(1).unwrap_or("").parse::<i32>().unwrap_or(0);
        let day = record.get(2).unwrap_or("").parse::<i32>().unwrap_or(0);
        let hour = record.get(3).unwrap_or("").parse::<i32>().unwrap_or(0);
        let minute = record.get(4).unwrap_or("").parse::<i32>().unwrap_or(0);
        let currency = record.get(5).unwrap_or("").trim();
        let impact_raw = record.get(6).unwrap_or("").trim();
        let description = record.get(7).unwrap_or("").trim();

        // FILTER: Exclude Bank Holidays (not economic events, just market closures)
        if description.eq_ignore_ascii_case("Bank Holiday") {
            continue;
        }

        if year < 2000 || month < 1 || month > 12 || day < 1 || day > 31 {
            continue;
        }

        let date = format!("{:04}-{:02}-{:02}", year, month, day);
        let time = format!("{:02}:{:02}", hour, minute);
        let impact = normalize_impact(impact_raw);

        if !matches!(impact.as_str(), "HIGH" | "MEDIUM" | "LOW") {
            continue;
        }

        events.push(ParsedEvent {
            date,
            time,
            currency: currency.to_string(),
            impact,
            description: description.to_string(),
        });
    }

    Ok(events)
}

fn parse_format_old(path: &str) -> Result<Vec<ParsedEvent>, String> {
    let file = fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = std::io::BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(false).from_reader(reader);

    let mut events = Vec::new();

    for result in csv_reader.records().skip(1) {
        let record = result.map_err(|e| format!("CSV error: {}", e))?;
        if record.len() < 5 {
            continue;
        }

        let date_str = record.get(0).unwrap_or("").trim();
        let time_str = record.get(1).unwrap_or("").trim();
        let currency = record.get(2).unwrap_or("").trim();
        let impact_raw = record.get(3).unwrap_or("").trim();
        let description = record.get(4).unwrap_or("").trim();

        // FILTER: Exclude Bank Holidays (not economic events, just market closures)
        if description.eq_ignore_ascii_case("Bank Holiday") {
            continue;
        }

        // Valider la date (YYYY/MM/DD → YYYY-MM-DD)
        let date_parts: Vec<&str> = date_str.split('/').collect();
        if date_parts.len() != 3 {
            continue;
        }

        if let (Ok(y), Ok(m), Ok(d)) = (
            date_parts[0].parse::<i32>(),
            date_parts[1].parse::<i32>(),
            date_parts[2].parse::<i32>(),
        ) {
            if y < 2000 || m < 1 || m > 12 || d < 1 || d > 31 {
                continue;
            }

            let date = format!("{:04}-{:02}-{:02}", y, m, d);
            let impact = normalize_impact(impact_raw);

            if !matches!(impact.as_str(), "HIGH" | "MEDIUM" | "LOW") {
                continue;
            }

            events.push(ParsedEvent {
                date,
                time: time_str.to_string(),
                currency: currency.to_string(),
                impact,
                description: description.to_string(),
            });
        }
    }

    Ok(events)
}

fn normalize_impact(raw: &str) -> String {
    match raw.to_uppercase().as_str() {
        "H" | "HIGH" => "HIGH".to_string(),
        "M" | "MEDIUM" | "MID" | "N" => "MEDIUM".to_string(),
        "L" | "LOW" => "LOW".to_string(),
        _ => "MEDIUM".to_string(),
    }
}
