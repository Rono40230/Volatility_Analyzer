use super::calendar_db_queries::{open_volatility_db, query_calendar_imports};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarImportInfo {
    pub id: i32,
    pub name: String,
    pub filename: String,
    pub event_count: i32,
    pub oldest_event_date: Option<String>,
    pub newest_event_date: Option<String>,
    pub imported_at: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarMetadataUI {
    pub id: i32,
    pub name: String,
    pub event_count: i32,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarPeriod {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub async fn get_calendar_imports_from_db() -> Result<Vec<CalendarImportInfo>, String> {
    tracing::info!("📊 Getting calendar imports from volatility.db...");
    let conn = open_volatility_db()?;
    let calendars = query_calendar_imports(&conn)?;
    tracing::info!("✅ Found {} calendar imports in database", calendars.len());
    Ok(calendars)
}

#[tauri::command]
pub async fn get_calendars_metadata() -> Result<Vec<CalendarMetadataUI>, String> {
    let calendars = get_calendar_imports_from_db().await?;
    Ok(calendars
        .into_iter()
        .map(|c| CalendarMetadataUI {
            id: c.id,
            name: c.name,
            event_count: c.event_count,
            start_date: c.oldest_event_date,
            end_date: c.newest_event_date,
        })
        .collect())
}

#[tauri::command]
#[allow(dead_code)]
pub async fn get_active_calendar_id() -> Result<Option<i32>, String> {
    tracing::info!("🔍 Getting active calendar ID...");
    let conn = open_volatility_db()?;
    let mut stmt = conn
        .prepare("SELECT id FROM calendar_imports WHERE is_active = 1 LIMIT 1")
        .map_err(|e| format!("Query failed: {}", e))?;
    let active_id = stmt.query_row([], |row| row.get(0)).ok();
    tracing::info!("🔍 Active calendar ID: {:?}", active_id);
    Ok(active_id)
}

#[tauri::command]
#[allow(dead_code)]
pub async fn set_active_calendar_id(calendar_id: i32) -> Result<(), String> {
    tracing::info!("📝 Setting active calendar ID to {}", calendar_id);
    let mut conn = open_volatility_db()?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;
    tx.execute("UPDATE calendar_imports SET is_active = 0", [])
        .map_err(|e| format!("Failed to deactivate calendars: {}", e))?;
    tx.execute(
        "UPDATE calendar_imports SET is_active = 1 WHERE id = ?",
        [calendar_id],
    )
    .map_err(|e| format!("Failed to activate calendar: {}", e))?;
    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;
    tracing::info!("✅ Active calendar set to {}", calendar_id);
    Ok(())
}

#[tauri::command]
pub async fn get_calendar_id_by_filename(filename: String) -> Result<Option<i32>, String> {
    tracing::info!("🔍 Getting calendar ID for filename: {}", filename);
    let conn = open_volatility_db()?;

    // Exact match
    let mut stmt = conn
        .prepare("SELECT id FROM calendar_imports WHERE filename = ?")
        .map_err(|e| format!("Query failed: {}", e))?;
    if let Ok(id) = stmt.query_row([&filename], |row| row.get(0)) {
        tracing::info!("🔍 Calendar ID (exact match) for {}: {:?}", filename, id);
        return Ok(Some(id));
    }

    // Pattern match
    let pattern = format!("%{}%", filename.trim_end_matches(".csv"));
    let mut stmt = conn
        .prepare("SELECT id FROM calendar_imports WHERE filename LIKE ?")
        .map_err(|e| format!("Query failed: {}", e))?;
    if let Ok(id) = stmt.query_row([&pattern], |row| row.get(0)) {
        tracing::info!("🔍 Calendar ID (pattern match) for {}: {:?}", filename, id);
        return Ok(Some(id));
    }

    tracing::warn!("⚠️ Calendar ID not found for {}", filename);
    Ok(None)
}

#[tauri::command]
pub async fn get_calendar_period_by_id(calendar_id: i32) -> Result<CalendarPeriod, String> {
    tracing::info!("🔍 Getting calendar period for ID: {}", calendar_id);
    let conn = open_volatility_db()?;
    let mut stmt = conn
        .prepare("SELECT oldest_event_date, newest_event_date FROM calendar_imports WHERE id = ?")
        .map_err(|e| format!("Query failed: {}", e))?;

    let period = stmt
        .query_row([calendar_id], |row| {
            let start: Option<String> = row.get(0)?;
            let end: Option<String> = row.get(1)?;
            tracing::debug!(
                "📅 Raw database values - start: {:?}, end: {:?}",
                start,
                end
            );
            Ok(CalendarPeriod {
                start_date: start,
                end_date: end,
            })
        })
        .map_err(|e| {
            tracing::error!("❌ Query row error: {}", e);
            format!("Calendar with ID {} not found", calendar_id)
        })?;

    tracing::info!(
        "✅ Calendar {} period: {:?} to {:?}",
        calendar_id,
        period.start_date,
        period.end_date
    );
    Ok(period)
}
