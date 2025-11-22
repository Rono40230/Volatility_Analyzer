// pair_correlation.rs - Corrélation événements pour une paire donnée
// Conforme .clinerules: < 200 lignes (appelle helpers)

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use chrono::Datelike;

use super::pair_correlation_helpers::{
    calculate_correlation_score, calculate_event_volatility_for_pair,
};
use crate::services::candle_index::CandleIndex;

fn format_date_fr(date_str: &str) -> String {
    let months = ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];
    
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        let m = months.get((dt.month() as usize).saturating_sub(1)).unwrap_or(&"?");
        return format!("{} {} {}", dt.day(), m, dt.year());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        let m = months.get((dt.month() as usize).saturating_sub(1)).unwrap_or(&"?");
        return format!("{} {} {}", dt.day(), m, dt.year());
    }
    date_str.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairEventCorrelation {
    pub name: String,
    pub count: i32,
    pub volatility_before: f64, // 30min avant
    pub volatility_after: f64,  // 30min après
    pub volatility_total: f64,  // 1h complète
    pub volatility_before_fmt: String,
    pub volatility_after_fmt: String,
    pub volatility_total_fmt: String,
    pub correlation_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_data: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairCorrelationResult {
    pub pair: String,
    pub period_start: String,
    pub period_end: String,
    pub events: Vec<PairEventCorrelation>,
}

/// Récupère la corrélation de TOUS les événements pour une paire donnée
pub fn calculate_pair_event_correlation(
    symbol: &str,
    calendar_id: Option<i32>,
    candle_index: &CandleIndex,
) -> Result<PairCorrelationResult, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");

    let volatility_db = data_dir.join("volatility.db");
    let pairs_db = data_dir.join("pairs.db");

    let conn_vol = Connection::open(&volatility_db)
        .map_err(|e| format!("Failed to open volatility.db: {}", e))?;
    let conn_pairs =
        Connection::open(&pairs_db).map_err(|e| format!("Failed to open pairs.db: {}", e))?;

    // 1. Récupérer la plage de dates des événements
    let range_query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT MIN(event_time), MAX(event_time) 
             FROM calendar_events 
             WHERE calendar_import_id = {} 
             AND impact IN ('HIGH', 'MEDIUM')",
            cal_id
        )
    } else {
        "SELECT MIN(event_time), MAX(event_time) 
         FROM calendar_events 
         WHERE impact IN ('HIGH', 'MEDIUM')"
            .to_string()
    };

    let mut range_stmt = conn_vol.prepare(&range_query).map_err(|e| format!("Failed to prepare range query: {}", e))?;
    let (start_str, end_str) = range_stmt.query_row([], |row| {
        let start: Option<String> = row.get(0)?;
        let end: Option<String> = row.get(1)?;
        Ok((start, end))
    }).unwrap_or((None, None));

    let period_start = start_str.map(|s| format_date_fr(&s)).unwrap_or_else(|| "N/A".to_string());
    let period_end = end_str.map(|s| format_date_fr(&s)).unwrap_or_else(|| "N/A".to_string());

    // Requête pour récupérer les événements regroupés
    // IMPORTANT: COUNT(DISTINCT event_time) déduplique les événements qui se produisent le même jour
    // mais sous différentes devises (ex: Bank Holiday JPY + Bank Holiday USD = 1 événement, pas 2)
    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT description, COUNT(DISTINCT event_time) as count 
             FROM calendar_events 
             WHERE calendar_import_id = {} 
             AND impact IN ('HIGH', 'MEDIUM')
             GROUP BY description
             ORDER BY count DESC",
            cal_id
        )
    } else {
        "SELECT description, COUNT(DISTINCT event_time) as count 
         FROM calendar_events 
         WHERE impact IN ('HIGH', 'MEDIUM')
         GROUP BY description
         ORDER BY count DESC"
            .to_string()
    };

    tracing::info!("🔍 pair_correlation: query = {}", query);

    let mut stmt = conn_vol
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let mut events = Vec::new();

    let event_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
        .map_err(|e| format!("Failed to query events: {}", e))?;

    for result in event_iter {
        let (event_name, count) = result.map_err(|e| format!("Row error: {}", e))?;

        let (vol_before, vol_after, vol_total, has_data) = calculate_event_volatility_for_pair(
            &conn_vol,
            &conn_pairs,
            symbol,
            &event_name,
            calendar_id,
            candle_index,
        )?;

        let correlation_score =
            calculate_correlation_score(vol_before, vol_after, vol_total, count);

        tracing::info!(
            "  📌 Event: {} (count={}, before={:.2}, after={:.2}, total={:.2}, has_data={})",
            event_name,
            count,
            vol_before,
            vol_after,
            vol_total,
            has_data
        );

        events.push(PairEventCorrelation {
            name: event_name,
            count,
            volatility_before: vol_before,
            volatility_after: vol_after,
            volatility_total: vol_total,
            volatility_before_fmt: format!("{:.2}", vol_before),
            volatility_after_fmt: format!("{:.2}", vol_after),
            volatility_total_fmt: format!("{:.2}", vol_total),
            correlation_score,
            has_data: Some(has_data),
        });
    }

    // Trier par volatilité totale réelle (décroissant)
    events.sort_by(|a, b| {
        b.volatility_total
            .partial_cmp(&a.volatility_total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    tracing::info!(
        "✅ pair_correlation: Found {} events for pair {}, sorted by volatility",
        events.len(),
        symbol
    );

    Ok(PairCorrelationResult {
        pair: symbol.to_string(),
        period_start,
        period_end,
        events,
    })
}
