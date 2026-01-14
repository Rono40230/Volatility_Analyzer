use rusqlite::Connection;
use std::collections::HashMap;
use chrono::NaiveDateTime;
use super::heatmap_helpers::EventTypeInfo;

/// Récupère tous les types d'événements disponibles
pub fn get_event_types(
    conn: &Connection,
    calendar_id: Option<i32>,
) -> Result<Vec<EventTypeInfo>, String> {
    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT description, COUNT(DISTINCT event_time) as count 
             FROM calendar_events 
             WHERE calendar_import_id = {} 
             GROUP BY description 
             HAVING count >= 1
             ORDER BY count DESC, description",
            cal_id
        )
    } else {
        "SELECT description, COUNT(DISTINCT event_time) as count 
         FROM calendar_events 
         GROUP BY description 
         HAVING count >= 1
         ORDER BY count DESC, description"
            .to_string()
    };

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let event_types = stmt
        .query_map([], |row| {
            Ok(EventTypeInfo {
                name: row.get(0)?,
                count: row.get(1)?,
                has_data: None,
            })
        })
        .map_err(|e| format!("Failed to query event types: {}", e))?
        .collect::<Result<Vec<EventTypeInfo>, _>>()
        .map_err(|e| format!("Failed to collect event types: {}", e))?;

    Ok(event_types)
}

/// Récupère tous les événements groupés par description pour éviter les requêtes N+1
pub fn get_all_events_grouped(
    conn: &Connection,
    calendar_id: Option<i32>,
) -> Result<HashMap<String, Vec<NaiveDateTime>>, String> {
    use super::utils::parse_sqlite_datetime;
    
    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT description, datetime(event_time) 
             FROM calendar_events 
             WHERE calendar_import_id = {} 
             ORDER BY event_time",
            cal_id
        )
    } else {
        "SELECT description, datetime(event_time) 
         FROM calendar_events 
         ORDER BY event_time"
            .to_string()
    };

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare all events query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let desc: String = row.get(0)?;
            let time_str: String = row.get(1)?;
            Ok((desc, time_str))
        })
        .map_err(|e| format!("Failed to query events: {}", e))?;

    let mut map: HashMap<String, Vec<NaiveDateTime>> = HashMap::new();

    for row in rows {
        if let Ok((desc, time_str)) = row {
            if let Ok(dt) = parse_sqlite_datetime(&time_str) {
                map.entry(desc).or_default().push(dt);
            }
        }
    }

    Ok(map)
}
