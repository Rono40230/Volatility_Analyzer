// pair_correlation_helpers.rs - Utilitaires pour calcul volatilité par paire
// Conforme .clinerules: < 150 lignes pour helpers

use rusqlite::Connection;

use super::data_availability::has_candles_for_event;
use super::volatility_helpers::parse_sqlite_datetime;
use crate::services::candle_index::CandleIndex;

/// Calcule la volatilité moyenne observée pour une paire lors d'un événement
/// Retourne: (volatilité_30min_avant, volatilité_30min_après, volatilité_1h_totale, has_data)
pub fn calculate_event_volatility_for_pair(
    conn_vol: &Connection,
    conn_pairs: &Connection,
    symbol: &str,
    event_name: &str,
    calendar_id: Option<i32>,
    candle_index: &CandleIndex,
) -> Result<(f64, f64, f64, bool), String> {
    // Récupérer tous les event_times pour cet événement
    let cal_filter = if let Some(cal_id) = calendar_id {
        format!(" AND calendar_import_id = {}", cal_id)
    } else {
        String::new()
    };

    let event_query = format!(
        "SELECT event_time FROM calendar_events 
         WHERE description = ?{}
         AND impact IN ('HIGH', 'MEDIUM')",
        cal_filter
    );

    let mut stmt_events = conn_vol
        .prepare(&event_query)
        .map_err(|e| format!("Failed to prepare event query: {}", e))?;

    let event_times: Vec<String> = stmt_events
        .query_map([event_name], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query event times: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect event times: {}", e))?;

    if event_times.is_empty() {
        return Ok((0.0, 0.0, 0.0, false));
    }

    let event_count = event_times.len();
    let mut volatilities_before = Vec::new();
    let mut volatilities_after = Vec::new();
    let mut volatilities_total = Vec::new();
    let mut has_data_found = false;

    for event_time in event_times {
        // Parse event_time et vérifier si des candles existent
        let event_dt = parse_sqlite_datetime(&event_time)?;

        if !has_candles_for_event(candle_index, symbol, event_dt) {
            // Skip cet événement, pas de candles disponibles
            continue;
        }

        has_data_found = true;

        // Volatilité 30min AVANT
        let query_before = "SELECT AVG(high - low) as avg_vol 
             FROM candle_data 
             WHERE symbol = ? AND timeframe = 'M1' 
             AND time <= ? AND time > datetime(?, '-30 minutes')
             LIMIT 30";

        let mut stmt_before = conn_pairs
            .prepare(query_before)
            .map_err(|e| format!("Failed to prepare query_before: {}", e))?;

        let vol_before: Option<f64> = stmt_before
            .query_row([symbol, &event_time, &event_time], |row| {
                row.get::<_, Option<f64>>(0)
            })
            .map_err(|e| format!("Failed to query volatility before: {}", e))?;

        if let Some(vol) = vol_before {
            if vol > 0.0 {
                volatilities_before.push(vol * 100.0);
            }
        }

        // Volatilité 30min APRÈS
        let query_after = "SELECT AVG(high - low) as avg_vol 
             FROM candle_data 
             WHERE symbol = ? AND timeframe = 'M1' 
             AND time > ? AND time < datetime(?, '+30 minutes')
             LIMIT 30";

        let mut stmt_after = conn_pairs
            .prepare(query_after)
            .map_err(|e| format!("Failed to prepare query_after: {}", e))?;

        let vol_after: Option<f64> = stmt_after
            .query_row([symbol, &event_time, &event_time], |row| {
                row.get::<_, Option<f64>>(0)
            })
            .map_err(|e| format!("Failed to query volatility after: {}", e))?;

        if let Some(vol) = vol_after {
            if vol > 0.0 {
                volatilities_after.push(vol * 100.0);
            }
        }

        // Volatilité TOTALE 1h
        let query_total = "SELECT AVG(high - low) as avg_vol 
             FROM candle_data 
             WHERE symbol = ? AND timeframe = 'M1' 
             AND time > datetime(?, '-30 minutes') 
             AND time < datetime(?, '+30 minutes')
             LIMIT 60";

        let mut stmt_total = conn_pairs
            .prepare(query_total)
            .map_err(|e| format!("Failed to prepare query_total: {}", e))?;

        let vol_total: Option<f64> = stmt_total
            .query_row([symbol, &event_time, &event_time], |row| {
                row.get::<_, Option<f64>>(0)
            })
            .map_err(|e| format!("Failed to query volatility total: {}", e))?;

        if let Some(vol) = vol_total {
            if vol > 0.0 {
                volatilities_total.push(vol * 100.0);
            }
        }
    }

    let avg_before = if !volatilities_before.is_empty() {
        volatilities_before.iter().sum::<f64>() / volatilities_before.len() as f64
    } else {
        0.0
    };

    let avg_after = if !volatilities_after.is_empty() {
        volatilities_after.iter().sum::<f64>() / volatilities_after.len() as f64
    } else {
        0.0
    };

    let avg_total = if !volatilities_total.is_empty() {
        volatilities_total.iter().sum::<f64>() / volatilities_total.len() as f64
    } else {
        0.0
    };

    tracing::debug!(
        "  📊 Event '{}': {} occ - before={:.2}, after={:.2}, total={:.2} pips (has_data={})",
        event_name,
        event_count,
        avg_before,
        avg_after,
        avg_total,
        has_data_found
    );

    Ok((avg_before, avg_after, avg_total, has_data_found))
}

/// Calcule le score de corrélation (0-100) basé sur les 3 volatilités
pub fn calculate_correlation_score(
    vol_before: f64,
    vol_after: f64,
    vol_total: f64,
    occurrences: i32,
) -> f64 {
    let avg_volatility = (vol_before + vol_after + vol_total) / 3.0;
    let impact = if vol_before > 0.0 {
        ((vol_after - vol_before) / vol_before * 100.0).abs()
    } else {
        0.0
    };

    let vol_factor = (avg_volatility / 10.0).min(60.0);
    let impact_factor = (impact / 10.0).min(25.0);
    let occ_factor = ((occurrences as f64).log2() / 10.0 * 15.0).min(15.0);

    (vol_factor + impact_factor + occ_factor).min(100.0)
}
