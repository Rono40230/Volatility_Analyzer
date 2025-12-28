use crate::commands::calendar_commands::CalendarState;
use crate::services::cleanup_service::CleanupService;
use tauri::State;

#[tauri::command]
pub async fn preview_cleanup_events(
    filter_type: String,
    filter_value: String,
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<Vec<crate::models::calendar_event::CalendarEvent>, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::preview_cleanup_events(&mut conn, filter_type, filter_value, calendar_id)
}
