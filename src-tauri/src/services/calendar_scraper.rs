// services/calendar_scraper.rs - Service de gestion des événements calendrier
// NOTE: Ce module est conservé pour usage futur (Phase 2 - scraping événements)

use chrono::NaiveDateTime;
use diesel::prelude::*;
use crate::db::DbPool;
use crate::db::schema::calendar_events;
use crate::models::{CalendarEvent, VolatilityError};

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
        let mut conn = self.db_pool.get()
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))?;

        calendar_events::table
            .filter(calendar_events::symbol.eq(symbol))
            .filter(calendar_events::event_time.between(start_time, end_time))
            .order(calendar_events::event_time.asc())
            .select(CalendarEvent::as_select())
            .load(&mut conn)
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))
    }

    /// Récupère les événements à venir pour un symbole
    #[allow(dead_code)]
    pub fn get_upcoming_events(
        &self,
        symbol: &str,
        hours_ahead: i64,
    ) -> Result<Vec<CalendarEvent>, VolatilityError> {
        let mut conn = self.db_pool.get()
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now().naive_utc();
        let future = now + chrono::Duration::hours(hours_ahead);

        calendar_events::table
            .filter(calendar_events::symbol.eq(symbol))
            .filter(calendar_events::event_time.between(now, future))
            .order(calendar_events::event_time.asc())
            .select(CalendarEvent::as_select())
            .load(&mut conn)
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))
    }
}
