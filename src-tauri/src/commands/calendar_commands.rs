// commands/calendar_commands.rs - Commandes Tauri pour le calendrier
// Conforme .clinerules : < 300 lignes, aucun unwrap()

use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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
