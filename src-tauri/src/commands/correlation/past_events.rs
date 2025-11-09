use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PastEvent {
    pub id: i32,
    pub name: String,
    pub datetime: String,
    pub country: String,
    pub currency: String,
    pub impact: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventType {
    pub name: String,
    pub count: i32,
    pub impact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTypesByImpact {
    pub high: Vec<EventType>,
    pub medium: Vec<EventType>,
}

#[tauri::command]
pub async fn get_past_events(months_back: Option<i32>) -> Result<EventTypesByImpact, String> {
    let months = months_back.unwrap_or(6);
    
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");
    
    let db_path = data_dir.join("volatility.db");
    
    if !db_path.exists() {
        return Err("Database not found".to_string());
    }
    
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // Calculer la date limite (ex: 6 mois en arrière)
    let cutoff_date = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(months as i64 * 30))
        .ok_or("Date calculation error")?
        .format("%Y-%m-%d")
        .to_string();
    
    // Grouper par description (type d'événement) et compter les occurrences
    let mut stmt = conn
        .prepare(
            "SELECT description, COUNT(*) as count, impact 
             FROM calendar_events 
             WHERE date(event_time) >= ?1 
             GROUP BY description, impact
             ORDER BY count DESC, description"
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let events_iter = stmt
        .query_map([&cutoff_date], |row| {
            Ok(EventType {
                name: row.get(0)?,         // description
                count: row.get(1)?,        // count
                impact: row.get(2)?,       // impact
            })
        })
        .map_err(|e| format!("Failed to query events: {}", e))?;
    
    let all_events: Vec<EventType> = events_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect events: {}", e))?;
    
    // Séparer par impact
    let high = all_events.iter().filter(|e| e.impact == "HIGH").cloned().collect();
    let medium = all_events.iter().filter(|e| e.impact == "MEDIUM").cloned().collect();
    
    Ok(EventTypesByImpact { high, medium })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTypeInfo {
    pub event_type: String,
    pub count: i32,
}
