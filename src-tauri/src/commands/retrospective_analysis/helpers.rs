// commands/retrospective_analysis/helpers.rs
// Fonctions utilitaires pour la analyse rétrospective (extracted)

use crate::schema::calendar_events::dsl::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use rusqlite;

pub async fn setup_databases(
    state: &tauri::State<'_, crate::commands::calendar_commands::CalendarState>,
) -> Result<
    (
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>>,
        crate::services::DatabaseLoader,
    ),
    String,
> {
    let pool_opt = state.pool.lock().map_err(|_| "DB lock failed")?;
    let pool = pool_opt.as_ref().ok_or("DB not initialized")?;
    let conn = pool
        .get()
        .map_err(|e| format!("Connection failed: {}", e))?;
    let data_dir = dirs::data_local_dir().ok_or("No data dir")?;
    let pairs_db_url = format!(
        "sqlite://{}",
        data_dir.join("volatility-analyzer/pairs.db").display()
    );
    let pairs_pool =
        crate::db::create_pool(&pairs_db_url).map_err(|e| format!("Pool failed: {}", e))?;
    Ok((conn, crate::services::DatabaseLoader::new(pairs_pool)))
}

pub async fn load_events_by_type(
    mut conn: diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>>,
    event_type_param: &str,
) -> Result<Vec<crate::models::CalendarEvent>, String> {
    use diesel::SelectableHelper;
    calendar_events
        .filter(description.eq(event_type_param))
        .select(crate::models::CalendarEvent::as_select())
        .order(event_time.asc())
        .load(&mut conn)
        .map_err(|e| format!("Load failed: {}", e))
}

pub fn calculer_atr(high: f64, low: f64, close: f64) -> f64 {
    (high - low).max((high - close.abs()).max(close - low.abs()))
}

pub fn get_event_types_from_db(
    db_path: &std::path::PathBuf,
    calendar_id: Option<i32>,
) -> Result<Vec<(String, usize)>, String> {
    let conn = rusqlite::Connection::open(db_path).map_err(|e| format!("Open: {}", e))?;
    let query = if let Some(cal_id) = calendar_id {
        format!(
            "SELECT description, COUNT(*) FROM calendar_events WHERE calendar_import_id = {} GROUP BY description ORDER BY COUNT(*) DESC",
            cal_id
        )
    } else {
        format!(
            "SELECT description, COUNT(*) FROM calendar_events GROUP BY description ORDER BY COUNT(*) DESC"
        )
    };

    let mut stmt = conn.prepare(&query).map_err(|e| format!("Prep: {}", e))?;
    let types: Vec<(String, usize)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("Query: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Collect: {}", e))?;

    Ok(types)
}
