// Requêtes BD pour calendrier (utilitaires pour refactorisation)
use super::CalendarImportInfo;
use rusqlite::Connection;

pub fn get_db_path() -> Result<std::path::PathBuf, String> {
    Ok(dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer")
        .join("volatility.db"))
}

pub fn open_volatility_db() -> Result<Connection, String> {
    let path = get_db_path()?;
    Connection::open(&path).map_err(|e| format!("Failed to open volatility.db: {}", e))
}

pub fn query_calendar_imports(conn: &Connection) -> Result<Vec<CalendarImportInfo>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, filename, event_count, oldest_event_date, newest_event_date, imported_at, is_active FROM calendar_imports ORDER BY imported_at DESC")
        .map_err(|e| format!("Query failed: {}", e))?;

    let calendars: Vec<CalendarImportInfo> = stmt
        .query_map([], |row| {
            Ok(CalendarImportInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                filename: row.get(2)?,
                event_count: row.get(3)?,
                oldest_event_date: row.get(4)?,
                newest_event_date: row.get(5)?,
                imported_at: row.get(6)?,
                is_active: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query execution failed: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Result collection failed: {}", e))?;

    Ok(calendars)
}
