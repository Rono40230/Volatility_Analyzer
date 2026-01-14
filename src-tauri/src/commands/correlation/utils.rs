use chrono::{Datelike, DateTime, NaiveDateTime, Utc};
use crate::models::AssetProperties;

/// Helper pour afficher une date en français
pub fn format_date_fr(date_str: &str) -> String {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        let day = dt.day();
        let month = match dt.month() {
            1 => "janvier",
            2 => "février",
            3 => "mars",
            4 => "avril",
            5 => "mai",
            6 => "juin",
            7 => "juillet",
            8 => "août",
            9 => "septembre",
            10 => "octobre",
            11 => "novembre",
            12 => "décembre",
            _ => "?",
        };
        let year = dt.year();
        return format!("{} {} {}", day, month, year);
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        let day = dt.day();
        let month = match dt.month() {
            1 => "janvier",
            2 => "février",
            3 => "mars",
            4 => "avril",
            5 => "mai",
            6 => "juin",
            7 => "juillet",
            8 => "août",
            9 => "septembre",
            10 => "octobre",
            11 => "novembre",
            12 => "décembre",
            _ => "?",
        };
        let year = dt.year();
        return format!("{} {} {}", day, month, year);
    }
    date_str.to_string()
}

/// Helper pour parser les dates de la BD vers DateTime<Utc>
pub fn parse_db_date(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    None
}

/// Parse une datetime depuis SQLite qui peut être soit un string formaté, soit un timestamp Unix
pub fn parse_sqlite_datetime(s: &str) -> Result<NaiveDateTime, String> {
    // Essayer d'abord comme timestamp Unix (nombre de secondes)
    if let Ok(timestamp) = s.parse::<i64>() {
        return DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.naive_utc())
            .ok_or_else(|| format!("Invalid Unix timestamp: {}", s));
    }

    // Sinon, essayer les formats de date classiques
    let formats = vec![
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d",
    ];

    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
            return Ok(dt);
        }
    }

    Err(format!(
        "Cannot parse datetime from any known format: '{}'",
        s
    ))
}

/// Retourne la valeur d'1 pip pour une paire donnée
pub fn get_pip_value(symbol: &str) -> f64 {
    let props = AssetProperties::from_symbol(symbol);
    props.pip_value
}
