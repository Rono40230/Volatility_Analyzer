use chrono::{DateTime, NaiveDateTime};

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
    match symbol {
        "ADAUSD" => 0.0001,
        "BTCUSD" => 1.00,
        "CADJPY" => 0.01,
        "CHFJPY" => 0.01,
        "ETHUSD" => 0.01,
        "GBPJPY" => 0.01,
        "LINKUSD" => 0.001,
        "LTCUSD" => 0.01,
        "UNIUSD" => 0.001,
        "USDCAD" => 0.0001,
        "USDJPY" => 0.01,
        "AUDJPY" => 0.01,
        "NZDJPY" => 0.01,
        "EURJPY" => 0.01,
        "XAGUSD" => 0.001,
        "XAUUSD" => 0.01,
        "XLMUSD" => 0.00001,
        "EURUSD" => 0.0001,
        "GBPUSD" => 0.0001,
        _ => 0.0001, // valeur par défaut
    }
}
