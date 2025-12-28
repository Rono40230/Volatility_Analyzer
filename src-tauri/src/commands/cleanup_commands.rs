use crate::commands::calendar_commands::CalendarState;
use crate::services::cleanup_service::CleanupService;
use crate::services::cleanup_helpers::{RareEventSummary, CurrencySummary, OrphanEventSummary, ImpactGroupSummary};
use tauri::State;

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
pub async fn update_symbol_for_description(
    description: String,
    old_symbol: String,
    new_symbol: String,
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

    CleanupService::update_symbol_for_description(&mut conn, description, old_symbol, new_symbol, calendar_id)
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
pub async fn list_impact_groups(
    calendar_id: Option<i32>,
    state: State<'_, CalendarState>,
) -> Result<Vec<ImpactGroupSummary>, String> {
    let pool_guard = state
        .pool
        .lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    let pool = pool_guard.as_ref().ok_or("Database pool not initialized")?;
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    CleanupService::list_impact_groups(&mut conn, calendar_id)
}

#[tauri::command]
pub async fn update_impact_for_description(
    description: String,
    new_impact: String,
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

    CleanupService::update_impact_for_description(&mut conn, description, new_impact, calendar_id)
}

#[tauri::command]
pub async fn delete_events_by_impact(
    impact: String,
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

    CleanupService::delete_events_by_impact(&mut conn, impact, calendar_id)
}
