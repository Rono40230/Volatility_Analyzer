// commands/calendar_commands.rs - Commandes Tauri pour le calendrier
// Conforme .clinerules : < 300 lignes, aucun unwrap()

use serde::{Serialize, Deserialize};
use tauri::State;
use std::sync::Mutex;
use crate::db::DbPool;
use crate::models::CalendarEvent;

pub struct CalendarState {
    pub pool: Mutex<Option<DbPool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarCommandError {
    pub message: String,
}

impl From<String> for CalendarCommandError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

#[tauri::command]
pub async fn get_upcoming_events(
    state: State<'_, CalendarState>,
) -> Result<Vec<CalendarEvent>, CalendarCommandError> {
    let pool_guard = state.pool.lock()
        .map_err(|e| format!("Failed to lock pool: {}", e))?;
    
    let _pool = pool_guard.as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    
    Ok(vec![])
}
