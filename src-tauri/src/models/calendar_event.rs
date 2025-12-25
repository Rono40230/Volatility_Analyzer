// models/calendar_event.rs - Événement économique
// Respect .clinerules: structs séparés Queryable vs Insertable

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::calendar_events;

/// Événement du calendrier économique (pour SELECT queries)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = calendar_events)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CalendarEvent {
    pub id: i32,
    pub symbol: String,
    pub event_time: NaiveDateTime,
    pub impact: String,
    pub description: String,
    pub actual: Option<f32>,
    pub forecast: Option<f32>,
    pub previous: Option<f32>,
    pub created_at: NaiveDateTime,
    pub calendar_import_id: i32,
}

/// Structure pour insérer un nouvel événement (INSERT)
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = calendar_events)]
pub struct NewCalendarEvent {
    pub symbol: String,
    pub event_time: NaiveDateTime,
    pub impact: String,
    pub description: String,
    pub actual: Option<f32>,
    pub forecast: Option<f32>,
    pub previous: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_new_calendar_event_creation() {
        let event = NewCalendarEvent {
            symbol: "EUR/USD".to_string(),
            event_time: Utc::now().naive_utc(),
            event_type: "Interest Rate".to_string(),
            impact_level: "HIGH".to_string(),
            description: Some("Fed Rate Decision".to_string()),
            actual_value: Some(5.50),
            forecast_value: Some(5.25),
            previous_value: Some(5.00),
        };

        assert_eq!(event.symbol, "EUR/USD");
        assert_eq!(event.impact_level, "HIGH");
        assert_eq!(event.actual_value, Some(5.50));
    }
}
