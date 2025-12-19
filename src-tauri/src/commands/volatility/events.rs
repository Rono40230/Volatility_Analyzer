use crate::commands::calendar_commands::CalendarState;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer, Text};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, QueryableByName)]
pub struct RecurringEvent {
    #[diesel(sql_type = Text)]
    pub time: String, // "HH:MM"
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub impact: String,
    #[diesel(sql_type = Text)]
    pub currency: String,
    #[diesel(sql_type = BigInt)]
    pub frequency: i64,
}

#[tauri::command]
pub async fn get_quarter_events(
    symbol: String,
    hour: u8,
    quarter: u8,
    calendar_state: State<'_, CalendarState>,
) -> Result<Vec<RecurringEvent>, String> {
    info!("Command: get_quarter_events({}, {}:{})", symbol, hour, quarter);

    let pool_opt = calendar_state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock calendar pool: {}", e))?;

    let pool = pool_opt
        .as_ref()
        .ok_or_else(|| "Calendar database not initialized".to_string())?;

    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    // Determine currencies from symbol (e.g., "EURUSD" -> "EUR", "USD")
    let base_currency = if symbol.len() >= 3 {
        &symbol[0..3]
    } else {
        ""
    };
    let quote_currency = if symbol.len() >= 6 {
        &symbol[3..6]
    } else {
        ""
    };

    let start_minute = quarter * 15;
    let end_minute = start_minute + 15;

    info!("Searching events for currencies {}/{} between {}:{:02} and {}:{:02}", 
        base_currency, quote_currency, hour, start_minute, hour, end_minute);

    // Query to find recurring events in this quarter
    // We group by name and time to find "typical" events
    // FIX: Remove currency filter to match EventLoader behavior (Global events)
    // FIX: Filter by High/Medium impact
    // FIX: Use 'symbol' column instead of 'currency' (schema mismatch)
    let query = "
        SELECT 
            strftime('%H:%M', event_time) as time,
            description as name,
            impact,
            symbol as currency,
            COUNT(*) as frequency
        FROM calendar_events
        WHERE 
            (impact = 'H' OR impact = 'HIGH' OR impact = 'M' OR impact = 'MEDIUM')
            AND CAST(strftime('%H', event_time) as INTEGER) = ?
            AND CAST(strftime('%M', event_time) as INTEGER) >= ?
            AND CAST(strftime('%M', event_time) as INTEGER) < ?
        GROUP BY time, name, impact, symbol
        ORDER BY time ASC, frequency DESC
    ";

    let events = diesel::sql_query(query)
        .bind::<Integer, _>(hour as i32)
        .bind::<Integer, _>(start_minute as i32)
        .bind::<Integer, _>(end_minute as i32)
        .load::<RecurringEvent>(&mut conn)
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    info!(
        "Found {} recurring events for {}:{}",
        events.len(),
        hour,
        quarter
    );
    
    // Normalize impact strings (H -> HIGH, M -> MEDIUM) for frontend
    let normalized_events: Vec<RecurringEvent> = events.into_iter().map(|mut e| {
        e.impact = match e.impact.to_uppercase().as_str() {
            "H" | "HIGH" => "HIGH".to_string(),
            "M" | "MEDIUM" => "MEDIUM".to_string(),
            _ => e.impact,
        };
        e
    }).collect();

    for event in &normalized_events {
        info!("  -> Event: {} at {} ({}) [{}]", event.name, event.time, event.currency, event.impact);
    }

    Ok(normalized_events)
}
