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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_data: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventType {
    pub name: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTypesByImpact {
    pub high: Vec<EventType>,
    pub medium: Vec<EventType>,
}

#[tauri::command]
pub async fn get_past_events(
    _months_back: Option<i32>,
    calendar_id: Option<i32>,
) -> Result<Vec<EventType>, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");

    let db_path = data_dir.join("volatility.db");

    if !db_path.exists() {
        return Err("Database not found".to_string());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    // Grouper par description (type d'événement) et compter les occurrences
    // IMPORTANT: COUNT(DISTINCT event_time) déduplique les événements de même jour/heure
    // mais différentes devises (ex: "Bank Holiday" du 1er janvier pour JPY, USD, EUR = 1 occurrence)
    // Filtrer par calendar_id si fourni
    // Afficher tous les événements HIGH et MEDIUM (toute la période)
    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT description, COUNT(DISTINCT event_time) as count
             FROM calendar_events 
             WHERE (UPPER(impact) IN ('H', 'HIGH', 'M', 'MEDIUM', 'N')) AND calendar_import_id = {}
             GROUP BY description
             ORDER BY count DESC, description",
            cal_id
        )
    } else {
        "SELECT description, COUNT(DISTINCT event_time) as count
         FROM calendar_events 
         WHERE UPPER(impact) IN ('H', 'HIGH', 'M', 'MEDIUM', 'N')
         GROUP BY description
         ORDER BY count DESC, description"
            .to_string()
    };

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let events_iter = stmt
        .query_map([], |row| {
            Ok(EventType {
                name: row.get(0)?,  // description
                count: row.get(1)?, // count
            })
        })
        .map_err(|e| format!("Failed to query events: {}", e))?;

    let all_events: Vec<EventType> = events_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect events: {}", e))?;

    Ok(all_events)
}
