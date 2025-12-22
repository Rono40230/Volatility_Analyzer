use crate::models::planning::ProjectedEvent;
use crate::services::planning::projection_engine::ProjectionEngine;
use crate::services::archive_service::ArchiveService;
use crate::commands::calendar_commands::CalendarState;
use tauri::State;
use chrono::{DateTime, Utc};

#[tauri::command]
pub async fn project_stats_on_calendar(
    start_date: String,
    end_date: String,
    calendar_state: State<'_, CalendarState>,
    archive_service: State<'_, ArchiveService>,
) -> Result<Vec<ProjectedEvent>, String> {
    let start = DateTime::parse_from_rfc3339(&start_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    let end = DateTime::parse_from_rfc3339(&end_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);

    let pool = calendar_state
        .pool
        .lock()
        .map_err(|_| "Failed to lock calendar state".to_string())?
        .clone()
        .ok_or("Calendar DB not initialized")?;
    
    let engine = ProjectionEngine::new(pool, archive_service.inner().clone());
    
    engine.project_stats(start, end).await
}
