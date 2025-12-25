use crate::commands::calendar_commands::CalendarState;
use crate::services::cleanup_service::CleanupService;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct RareEventSummary {
    pub description: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencySummary {
    pub symbol: String,
    pub country_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanEventSummary {
    pub reason: String,
    pub count: i64,
}

#[tauri::command]
pub async fn list_rare_events(
    min_occurrences: i64,
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<Vec<RareEventSummary>, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::list_rare_events(&mut conn, min_occurrences, calendar_id)
}

#[tauri::command]
pub async fn delete_rare_events(
    min_occurrences: i64,
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<usize, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::delete_rare_events(&mut conn, min_occurrences, calendar_id)
}

#[tauri::command]
pub async fn list_currencies(
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<Vec<CurrencySummary>, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::list_currencies(&mut conn, calendar_id)
}

#[tauri::command]
pub async fn delete_currency_events(
    currency_symbol: String,
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<usize, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::delete_currency_events(&mut conn, currency_symbol, calendar_id)
}

#[tauri::command]
pub async fn list_orphan_events(
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<Vec<OrphanEventSummary>, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::list_orphan_events(&mut conn, calendar_id)
}

#[tauri::command]
pub async fn delete_orphan_events(
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<usize, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::delete_orphan_events(&mut conn, calendar_id)
}

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
