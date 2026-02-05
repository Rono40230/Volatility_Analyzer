use anyhow::{anyhow, Result};
use calamine::Data;
use chrono::NaiveDate;

/// Normalise la date: 2007/01/01 → 2007-01-01
pub fn normalize_date(date: &str) -> Result<String> {
    if date.contains('/') {
        let parts: Vec<&str> = date.split('/').collect();
        if parts.len() == 3 {
            let y = parts[0].parse::<u32>().map_err(|_| anyhow!("Année invalide"))?;
            let m = parts[1].parse::<u32>().map_err(|_| anyhow!("Mois invalide"))?;
            let d = parts[2].parse::<u32>().map_err(|_| anyhow!("Jour invalide"))?;
            
            if NaiveDate::from_ymd_opt(y as i32, m, d).is_some() {
                return Ok(format!("{}-{:02}-{:02}", y, m, d));
            }
        }
    }

    if let Ok(parsed) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        return Ok(parsed.format("%Y-%m-%d").to_string());
    }

    Err(anyhow!("Format de date invalide: {}", date))
}

/// Convertit une cellule Excel en String
pub fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(dt) => dt.clone(),
        Data::DurationIso(d) => d.clone(),
        Data::Error(_) => String::new(),
        Data::Empty => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_date_slash_format() {
        assert_eq!(
            normalize_date("2007/01/01").expect("Failed to normalize"),
            "2007-01-01"
        );
    }

    #[test]
    fn test_normalize_date_dash_format() {
        assert_eq!(
            normalize_date("2007-01-01").expect("Failed to normalize"),
            "2007-01-01"
        );
    }

    #[test]
    fn test_normalize_date_edge_month() {
        assert_eq!(
            normalize_date("2025/12/31").expect("Failed to normalize"),
            "2025-12-31"
        );
    }

    #[test]
    fn test_normalize_date_invalid() {
        let result = normalize_date("invalid-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_date_partial() {
        let result = normalize_date("2007/01");
        assert!(result.is_err());
    }

    #[test]
    fn test_cell_to_string_string() {
        let cell = Data::String("test".to_string());
        assert_eq!(cell_to_string(&cell), "test");
    }

    #[test]
    fn test_cell_to_string_int() {
        let cell = Data::Int(42);
        assert_eq!(cell_to_string(&cell), "42");
    }

    #[test]
    fn test_cell_to_string_float() {
        let cell = Data::Float(std::f64::consts::PI);
        assert!(cell_to_string(&cell).contains("3.14"));
    }

    #[test]
    fn test_cell_to_string_empty() {
        let cell = Data::Empty;
        assert_eq!(cell_to_string(&cell), "");
    }
}
