// services/calendar_scraper.rs - Service de gestion des événements calendrier
// NOTE: Ce module est conservé pour usage futur (Phase 2 - scraping événements)

use crate::schema::calendar_events;
use crate::db::DbPool;
use crate::models::{CalendarEvent, VolatilityError};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::SelectableHelper;

#[allow(dead_code)]
pub struct CalendarScraper {
    db_pool: DbPool,
}

impl CalendarScraper {
    #[allow(dead_code)]
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Récupère les événements historiques pour un symbole dans une plage de dates
    #[allow(dead_code)]
    pub fn get_historical_events(
        &self,
        symbol: &str,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> Result<Vec<CalendarEvent>, VolatilityError> {
        let mut conn = self
            .db_pool
            .get()
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))?;

        calendar_events::table
            .filter(calendar_events::symbol.eq(symbol))
            .filter(calendar_events::event_time.between(start_time, end_time))
            .order(calendar_events::event_time.asc())
            .select(CalendarEvent::as_select())
            .load(&mut conn)
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))
    }

    /// Sauvegarde une liste d'événements dans la base de données
    #[allow(dead_code)]
    pub fn store_events(&self, events: &[crate::models::calendar_event::NewCalendarEvent]) -> Result<usize, VolatilityError> {
        use crate::schema::calendar_events::dsl::*;
        
        let mut conn = self.db_pool.get()
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))?;

        // SQLite doesn't support batch insert with returning or some features in older diesel versions/sqlite versions
        // But standard batch insert should work.
        // The error "trait diesel::Expression is not implemented for f64" was due to schema mismatch (Float vs Double).
        // Now that schema is Double, it should work.
        diesel::insert_into(calendar_events)
            .values(events)
            .execute(&mut conn)
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))
    }
}
