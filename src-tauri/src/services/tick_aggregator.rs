// services/tick_aggregator.rs
// Agrège des ticks Dukascopy (bid+ask dans un même fichier) en bougies M1 enrichies.
//
// Format Dukascopy attendu :
//   Séparateur : `;`
//   Décimal : `,` (format UE)
//   Header : Time (EET);Ask;Bid;AskVolume;BidVolume
//   Exemple : 2025.08.01 00:00:00.005;1,14172;1,14126;0,9;0,9

use chrono::{DateTime, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{info, warn};

/// Tick brut parsé depuis le CSV Dukascopy (ou converti depuis bi5)
#[derive(Debug, Clone)]
pub struct RawTick {
    pub datetime_utc: DateTime<Utc>,
    pub bid: f64,
    pub ask: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
}

/// Bougie M1 enrichie avec données de spread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedM1 {
    pub datetime_utc: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub spread_open: f64,
    pub spread_high: f64,
    pub spread_low: f64,
    pub spread_close: f64,
    pub spread_mean: f64,
    pub tick_count: i32,
}

/// Résultat de l'agrégation complète
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub symbol: String,
    pub candles: Vec<EnrichedM1>,
    pub total_ticks: usize,
    pub date_start: String,
    pub date_end: String,
    pub avg_spread: f64,
    pub avg_ticks_per_minute: f64,
}

/// Parse un fichier tick Dukascopy et agrège en M1 enrichies.
///
/// Le fichier contient bid ET ask sur chaque ligne.
/// Timezone : EET (UTC+2 hiver / UTC+3 été via Europe/Athens).
pub fn aggregate_ticks_to_m1(file_path: &str) -> Result<AggregationResult, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("Fichier introuvable : {}", file_path));
    }

    let symbol = extract_symbol_from_filename(file_path)?;

    info!("🔄 Lecture ticks Dukascopy : {}", file_path);
    let ticks = parse_dukascopy_file(file_path)?;
    let total_ticks = ticks.len();

    if ticks.is_empty() {
        return Err("Aucun tick valide trouvé dans le fichier".to_string());
    }

    info!("📊 {} ticks parsés, agrégation en M1…", total_ticks);
    let candles = aggregate_to_m1(&ticks);

    let avg_spread = if !candles.is_empty() {
        candles.iter().map(|c| c.spread_mean).sum::<f64>() / candles.len() as f64
    } else {
        0.0
    };

    let avg_ticks_per_min = if !candles.is_empty() {
        total_ticks as f64 / candles.len() as f64
    } else {
        0.0
    };

    let date_start = candles
        .first()
        .map(|c| c.datetime_utc.to_rfc3339())
        .unwrap_or_default();
    let date_end = candles
        .last()
        .map(|c| c.datetime_utc.to_rfc3339())
        .unwrap_or_default();

    info!(
        "✅ {} M1 enrichies générées ({} → {}), spread moyen = {:.5}, ~{:.0} ticks/min",
        candles.len(),
        date_start,
        date_end,
        avg_spread,
        avg_ticks_per_min,
    );

    Ok(AggregationResult {
        symbol,
        candles,
        total_ticks,
        date_start,
        date_end,
        avg_spread,
        avg_ticks_per_minute: avg_ticks_per_min,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Parseur Dukascopy
// ─────────────────────────────────────────────────────────────────────────────

fn parse_dukascopy_file(file_path: &str) -> Result<Vec<RawTick>, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Erreur ouverture fichier : {}", e))?;
    let reader = BufReader::with_capacity(256 * 1024, file);

    // EET = Europe/Athens (UTC+2 hiver, UTC+3 été, identique à EET)
    let eet: Tz = "Europe/Athens"
        .parse()
        .map_err(|e| format!("Erreur timezone : {}", e))?;

    let mut ticks = Vec::with_capacity(500_000);
    let mut line_count = 0usize;
    let mut error_count = 0usize;

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("Erreur lecture ligne : {}", e))?;
        line_count += 1;

        // Skip header
        if line_count == 1 && line.contains("Time") {
            continue;
        }

        match parse_dukascopy_line(&line, &eet) {
            Ok(tick) => ticks.push(tick),
            Err(_) => {
                error_count += 1;
                if error_count <= 5 {
                    warn!("⚠ Ligne {} ignorée : {}", line_count, &line[..line.len().min(80)]);
                }
            }
        }
    }

    if error_count > 5 {
        warn!("⚠ {} lignes ignorées au total", error_count);
    }

    Ok(ticks)
}

/// Parse une seule ligne Dukascopy.
/// Format : `2025.08.01 00:00:00.005;1,14172;1,14126;0,9;0,9`
fn parse_dukascopy_line(line: &str, eet: &Tz) -> Result<RawTick, String> {
    let parts: Vec<&str> = line.split(';').collect();
    if parts.len() < 5 {
        return Err("Pas assez de colonnes".to_string());
    }

    let datetime_utc = parse_eet_datetime(parts[0], eet)?;
    let ask = parse_eu_decimal(parts[1])?;
    let bid = parse_eu_decimal(parts[2])?;
    let ask_volume = parse_eu_decimal(parts[3])?;
    let bid_volume = parse_eu_decimal(parts[4])?;

    if bid <= 0.0 || ask <= 0.0 || ask < bid {
        return Err(format!("Prix invalides : bid={}, ask={}", bid, ask));
    }

    Ok(RawTick {
        datetime_utc,
        bid,
        ask,
        bid_volume,
        ask_volume,
    })
}

/// Parse un timestamp EET en UTC.
/// Formats supportés :
///   `2025.08.01 00:00:00.005` (avec ms)
///   `2025.08.01 00:00:00` (sans ms)
fn parse_eet_datetime(s: &str, eet: &Tz) -> Result<DateTime<Utc>, String> {
    // Tronquer les millisecondes (NaiveDateTime ne les gère pas facilement)
    let base = if let Some(dot_pos) = s.rfind('.') {
        // Vérifier que c'est bien le séparateur ms (après les secondes)
        // et pas le séparateur de la date (2025.08.01)
        let after_dot = &s[dot_pos + 1..];
        if after_dot.len() <= 3 && after_dot.chars().all(|c| c.is_ascii_digit()) {
            &s[..dot_pos]
        } else {
            s
        }
    } else {
        s
    };

    let naive = NaiveDateTime::parse_from_str(base, "%Y.%m.%d %H:%M:%S")
        .map_err(|e| format!("Format datetime invalide '{}' : {}", s, e))?;

    // Convertir EET → UTC. Utilise `earliest()` pour gérer l'ambiguïté DST.
    let local = eet
        .from_local_datetime(&naive)
        .earliest()
        .ok_or_else(|| format!("Conversion EET→UTC impossible : {}", s))?;

    Ok(local.with_timezone(&Utc))
}

/// Parse un nombre avec virgule décimale européenne.
/// `1,14172` → `1.14172`
fn parse_eu_decimal(s: &str) -> Result<f64, String> {
    let cleaned = s.trim().replace(',', ".");
    cleaned
        .parse::<f64>()
        .map_err(|e| format!("Nombre invalide '{}' : {}", s, e))
}

// ─────────────────────────────────────────────────────────────────────────────
// Agrégation ticks → M1 enrichies
// ─────────────────────────────────────────────────────────────────────────────

pub fn aggregate_to_m1(ticks: &[RawTick]) -> Vec<EnrichedM1> {
    // Grouper les ticks par minute (clé = timestamp tronqué à la minute)
    let mut minute_buckets: BTreeMap<DateTime<Utc>, Vec<&RawTick>> = BTreeMap::new();

    for tick in ticks {
        let minute_key = tick.datetime_utc.date_naive().and_hms_opt(
            tick.datetime_utc.time().hour(),
            tick.datetime_utc.time().minute(),
            0,
        );
        if let Some(naive) = minute_key {
            let key = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
            minute_buckets.entry(key).or_default().push(tick);
        }
    }

    let mut candles = Vec::with_capacity(minute_buckets.len());

    for (minute_dt, bucket) in &minute_buckets {
        if bucket.is_empty() {
            continue;
        }

        // Prix mid = (bid + ask) / 2
        let first = bucket[0];
        let last = bucket[bucket.len() - 1];

        let first_mid = (first.bid + first.ask) / 2.0;
        let last_mid = (last.bid + last.ask) / 2.0;

        let mut high_mid = f64::NEG_INFINITY;
        let mut low_mid = f64::INFINITY;
        let mut total_volume = 0.0;
        let mut spread_high = f64::NEG_INFINITY;
        let mut spread_low = f64::INFINITY;
        let mut spread_sum = 0.0;

        for tick in bucket {
            let mid = (tick.bid + tick.ask) / 2.0;
            let spread = tick.ask - tick.bid;

            if mid > high_mid {
                high_mid = mid;
            }
            if mid < low_mid {
                low_mid = mid;
            }
            if spread > spread_high {
                spread_high = spread;
            }
            if spread < spread_low {
                spread_low = spread;
            }
            spread_sum += spread;
            total_volume += tick.bid_volume + tick.ask_volume;
        }

        let tick_count = bucket.len() as i32;
        let spread_mean = spread_sum / tick_count as f64;
        let spread_open = first.ask - first.bid;
        let spread_close = last.ask - last.bid;

        candles.push(EnrichedM1 {
            datetime_utc: *minute_dt,
            open: first_mid,
            high: high_mid,
            low: low_mid,
            close: last_mid,
            volume: total_volume,
            spread_open,
            spread_high,
            spread_low,
            spread_close,
            spread_mean,
            tick_count,
        });
    }

    candles
}

// ─────────────────────────────────────────────────────────────────────────────
// Utilitaires
// ─────────────────────────────────────────────────────────────────────────────

/// Extrait le symbole depuis le nom du fichier.
/// Ex : `EURUSD_Ticks_2025.08.01_2026.01.31.csv` → `EURUSD`
fn extract_symbol_from_filename(path: &str) -> Result<String, String> {
    let filename = Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or("Nom de fichier invalide")?;

    // Le symbole est la partie avant le premier '_'
    let symbol = filename
        .split('_')
        .next()
        .ok_or("Format de fichier non reconnu (attendu: SYMBOL_Ticks_...)")?;

    if symbol.len() < 3 || symbol.len() > 10 {
        return Err(format!(
            "Symbole extrait invalide '{}' (attendu 3-10 caractères)",
            symbol
        ));
    }

    Ok(symbol.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_eu_decimal() {
        assert!((parse_eu_decimal("1,14172").unwrap() - 1.14172).abs() < 1e-10);
        assert!((parse_eu_decimal("0,9").unwrap() - 0.9).abs() < 1e-10);
        assert!(parse_eu_decimal("abc").is_err());
    }

    #[test]
    fn test_extract_symbol() {
        assert_eq!(
            extract_symbol_from_filename("/tmp/EURUSD_Ticks_2025.csv").unwrap(),
            "EURUSD"
        );
        assert_eq!(
            extract_symbol_from_filename("XAUUSD_Ticks_2025.08.01_2026.01.31.csv").unwrap(),
            "XAUUSD"
        );
    }

    #[test]
    fn test_parse_eet_datetime() {
        let eet: Tz = "Europe/Athens".parse().unwrap();
        // Hiver (EET = UTC+2) : 2025.01.15 10:30:00 EET → 08:30:00 UTC
        let dt = parse_eet_datetime("2025.01.15 10:30:00.123", &eet).unwrap();
        assert_eq!(dt.hour(), 8);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_dukascopy_line() {
        let eet: Tz = "Europe/Athens".parse().unwrap();
        let line = "2025.08.01 00:00:00.005;1,14172;1,14126;0,9;0,9";
        let tick = parse_dukascopy_line(line, &eet).unwrap();
        assert!((tick.ask - 1.14172).abs() < 1e-10);
        assert!((tick.bid - 1.14126).abs() < 1e-10);
        assert!((tick.ask_volume - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_aggregate_single_minute() {
        let eet: Tz = "Europe/Athens".parse().unwrap();
        let lines = vec![
            "2025.08.01 00:00:00.005;1,14172;1,14126;0,9;0,9",
            "2025.08.01 00:00:17.117;1,14191;1,14130;0,54;0,9",
            "2025.08.01 00:00:56.088;1,14191;1,14135;0,54;0,67",
        ];
        let ticks: Vec<RawTick> = lines
            .iter()
            .map(|l| parse_dukascopy_line(l, &eet).unwrap())
            .collect();

        let candles = aggregate_to_m1(&ticks);
        assert_eq!(candles.len(), 1);

        let c = &candles[0];
        assert_eq!(c.tick_count, 3);
        // open mid = (1.14126+1.14172)/2 = 1.14149
        assert!((c.open - 1.14149).abs() < 1e-4);
        // spread_open = 1.14172 - 1.14126 = 0.00046
        assert!((c.spread_open - 0.00046).abs() < 1e-6);
    }

    #[test]
    fn test_aggregate_multi_minutes_produces_multiple_candles() {
        let eet: Tz = "Europe/Athens".parse().unwrap();
        // Tick à minute 0
        let line0 = "2025.08.01 00:00:30.000;1,14172;1,14126;0,9;0,9";
        // Tick à minute 1
        let line1 = "2025.08.01 00:01:15.000;1,14200;1,14150;0,5;0,5";
        // Tick à minute 3 (minute 2 manquante → pas de candle pour min 2)
        let line3 = "2025.08.01 00:03:05.000;1,14210;1,14160;0,8;0,8";

        let ticks: Vec<RawTick> = vec![line0, line1, line3]
            .iter()
            .map(|l| parse_dukascopy_line(l, &eet).unwrap())
            .collect();

        let candles = aggregate_to_m1(&ticks);
        // 3 ticks dans 3 minutes distinctes → 3 candles
        assert_eq!(candles.len(), 3);

        // Vérifier que les timestamps sont triés (BTreeMap)
        for i in 1..candles.len() {
            assert!(candles[i].datetime_utc > candles[i - 1].datetime_utc);
        }
        // Chaque candle a tick_count=1 (un seul tick par minute)
        for c in &candles {
            assert_eq!(c.tick_count, 1);
        }
    }

    #[test]
    fn test_empty_ticks_no_candles() {
        let candles = aggregate_to_m1(&[]);
        assert!(candles.is_empty());
    }

    #[test]
    fn test_spread_mean_exact_value() {
        let eet: Tz = "Europe/Athens".parse().unwrap();
        // 2 ticks dans la même minute avec spreads différents
        let lines = vec![
            "2025.01.15 10:00:10.000;1,10050;1,10000;1,0;1,0", // spread = 0.00050
            "2025.01.15 10:00:40.000;1,10080;1,10020;1,0;1,0", // spread = 0.00060
        ];
        let ticks: Vec<RawTick> = lines
            .iter()
            .map(|l| parse_dukascopy_line(l, &eet).unwrap())
            .collect();

        let candles = aggregate_to_m1(&ticks);
        assert_eq!(candles.len(), 1);
        let c = &candles[0];
        assert_eq!(c.tick_count, 2);
        // spread_mean = (0.00050 + 0.00060) / 2 = 0.00055
        assert!((c.spread_mean - 0.00055).abs() < 1e-8);
        // spread_high = 0.00060
        assert!((c.spread_high - 0.00060).abs() < 1e-8);
        // spread_low = 0.00050
        assert!((c.spread_low - 0.00050).abs() < 1e-8);
        // volume = (1+1) + (1+1) = 4
        assert!((c.volume - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_single_tick_produces_valid_candle() {
        let eet: Tz = "Europe/Athens".parse().unwrap();
        let line = "2025.01.15 10:00:00.000;1,10050;1,10000;1,0;1,0";
        let tick = parse_dukascopy_line(line, &eet).unwrap();

        let candles = aggregate_to_m1(&[tick]);
        assert_eq!(candles.len(), 1);
        let c = &candles[0];
        assert_eq!(c.tick_count, 1);
        // open == close == mid = (1.10000 + 1.10050) / 2 = 1.10025
        assert!((c.open - c.close).abs() < 1e-10);
        assert!((c.open - 1.10025).abs() < 1e-6);
        // high == low == open pour un seul tick
        assert!((c.high - c.low).abs() < 1e-10);
    }

    #[test]
    fn test_invalid_line_rejected() {
        let eet: Tz = "Europe/Athens".parse().unwrap();
        // ask < bid → invalide
        let result = parse_dukascopy_line("2025.01.15 10:00:00.000;1,10000;1,10050;1,0;1,0", &eet);
        assert!(result.is_err());
    }
}
