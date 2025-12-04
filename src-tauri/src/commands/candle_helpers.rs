// commands/candle_helpers.rs
// Utilitaires pour candle_index_commands

use crate::models::Candle;
use chrono::{NaiveDate, Timelike};
use serde::Serialize;
use tracing::{debug, info, warn};

#[derive(Serialize)]
pub struct CandlesForHourResponse {
    pub symbol: String,
    pub date: String,
    pub hour: u32,
    pub candle_count: usize,
    pub candles: Vec<Candle>,
}

#[derive(Serialize)]
pub struct CandlesForQuarterResponse {
    pub symbol: String,
    pub hour: u8,
    pub quarter: u8,
    pub candle_count: usize,
    pub candles: Vec<Candle>,
}

/// Valide et parse une date au format "YYYY-MM-DD"
pub fn parse_and_validate_date(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format '{}': {}", date_str, e))
}

/// Valide une heure (0-23)
pub fn validate_hour(hour: u32) -> Result<(), String> {
    if hour > 23 {
        Err(format!("Invalid hour: {} (must be 0-23)", hour))
    } else {
        Ok(())
    }
}

/// Valide un quarter (0-3)
pub fn validate_quarter(quarter: u8) -> Result<(), String> {
    if quarter > 3 {
        Err(format!("Invalid quarter: {} (must be 0-3)", quarter))
    } else {
        Ok(())
    }
}

/// Filtre les candles par quarter (15 min)
/// Paris timezone: UTC+1, groupé par (hour, quarter)
pub fn filter_by_quarter(all_candles: Vec<Candle>, hour: u8, quarter: u8) -> Vec<Candle> {
    const PARIS_OFFSET: i32 = 1;
    let mut debug_count = 0;

    all_candles
        .into_iter()
        .filter(|candle| {
            let utc_hour = candle.hour_utc() as i32;
            let utc_minute = candle.datetime.minute() as i32;

            let paris_hour = (utc_hour + PARIS_OFFSET) % 24;
            let paris_minute = utc_minute;

            let q = (paris_minute / 15) as u8;
            let matches = paris_hour == hour as i32 && q == quarter;

            if debug_count < 5 {
                debug!(
                    "  Candle: {} UTC {}:{} → Paris {}:{} q={} → match={}",
                    candle.datetime, utc_hour, utc_minute, paris_hour, paris_minute, q, matches
                );
                debug_count += 1;
            }

            matches
        })
        .collect()
}

/// Retourne un message d'erreur si les candles filtrées sont vides
pub fn ensure_not_empty(
    filtered: &[Candle],
    symbol: &str,
    hour: u8,
    quarter: u8,
) -> Result<(), String> {
    if filtered.is_empty() {
        warn!(
            "❌ No candles found for {} hour={}:{} quarter={}",
            symbol,
            hour,
            quarter * 15,
            quarter
        );
        Err(format!(
            "No candles for {} hour {}:{} (quarter {})",
            symbol,
            hour,
            quarter * 15,
            quarter
        ))
    } else {
        info!(
            "✅ Filtered {} candles for {}:{}(q{})",
            filtered.len(),
            hour,
            quarter * 15,
            quarter
        );
        Ok(())
    }
}
