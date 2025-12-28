use diesel::prelude::*;
use crate::services::cleanup_helpers::{RareEventSummary, CurrencySummary, OrphanEventSummary, ImpactGroupSummary};
use crate::models::calendar_event::CalendarEvent;
use crate::services::cleanup_helpers;

pub struct CleanupService;

impl CleanupService {
    pub fn list_rare_events(conn: &mut SqliteConnection, min_occurrences: i64, calendar_id: Option<i32>) -> Result<Vec<RareEventSummary>, String> {
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
        cleanup_helpers::list_rare_events_helper(conn, min_occurrences, &allowed_ids)
    }

    pub fn delete_rare_events(conn: &mut SqliteConnection, min_occurrences: i64, calendar_id: Option<i32>) -> Result<usize, String> {
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
        cleanup_helpers::delete_rare_events_helper(conn, min_occurrences, &allowed_ids)
    }

    pub fn list_currencies(conn: &mut SqliteConnection, calendar_id: Option<i32>) -> Result<Vec<CurrencySummary>, String> {
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
        cleanup_helpers::list_currencies_helper(conn, &allowed_ids)
    }

    pub fn delete_currency_events(conn: &mut SqliteConnection, currency_symbol: String, calendar_id: Option<i32>) -> Result<usize, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
    
        let deleted_count = diesel::delete(
            calendar_events
                .filter(symbol.eq(currency_symbol))
                .filter(calendar_import_id.eq_any(&allowed_ids)),
        )
        .execute(conn)
        .map_err(|e| format!("Failed to delete events: {}", e))?;
    
        Ok(deleted_count)
    }

    pub fn update_symbol_for_description(
        conn: &mut SqliteConnection,
        target_description: String,
        old_symbol: String,
        new_symbol: String,
        calendar_id: Option<i32>,
    ) -> Result<usize, String> {
        use crate::schema::calendar_events::dsl::*;

        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;

        let updated_count = diesel::update(
            calendar_events
                .filter(description.eq(target_description))
                .filter(symbol.eq(old_symbol))
                .filter(calendar_import_id.eq_any(&allowed_ids)),
        )
        .set(symbol.eq(new_symbol))
        .execute(conn)
        .map_err(|e| format!("Failed to update symbol: {}", e))?;

        Ok(updated_count)
    }

    pub fn list_orphan_events(conn: &mut SqliteConnection, calendar_id: Option<i32>) -> Result<Vec<OrphanEventSummary>, String> {
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
        cleanup_helpers::list_orphan_events_helper(conn, &allowed_ids)
    }

    pub fn delete_orphan_events(conn: &mut SqliteConnection, calendar_id: Option<i32>) -> Result<usize, String> {
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
        cleanup_helpers::delete_orphan_events_helper(conn, &allowed_ids)
    }

    pub fn preview_cleanup_events(conn: &mut SqliteConnection, filter_type: String, filter_value: String, calendar_id: Option<i32>) -> Result<Vec<CalendarEvent>, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
        
        use diesel::SelectableHelper;
    
        let query = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids));
    
        let events = match filter_type.as_str() {
            "description" => query
                .filter(description.eq(filter_value))
                .select(CalendarEvent::as_select())
                .load(conn),
            "symbol" => query
                .filter(symbol.eq(filter_value))
                .select(CalendarEvent::as_select())
                .load(conn),
            "orphan_symbol" => query
                .filter(symbol.eq(""))
                .select(CalendarEvent::as_select())
                .load(conn),
            "orphan_desc" => query
                .filter(description.eq(""))
                .select(CalendarEvent::as_select())
                .load(conn),
            "orphan_impact" => query
                .filter(impact.eq(""))
                .select(CalendarEvent::as_select())
                .load(conn),
            _ => return Err("Invalid filter type".to_string()),
        }
        .map_err(|e| format!("Database error: {}", e))?;
    
        Ok(events)
    }

    pub fn list_impact_groups(conn: &mut SqliteConnection, calendar_id: Option<i32>) -> Result<Vec<ImpactGroupSummary>, String> {
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
        cleanup_helpers::list_impact_groups_helper(conn, &allowed_ids)
    }

    pub fn update_impact_for_description(conn: &mut SqliteConnection, target_description: String, new_impact: String, calendar_id: Option<i32>) -> Result<usize, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
    
        let updated_count = diesel::update(
            calendar_events
                .filter(description.eq(target_description))
                .filter(calendar_import_id.eq_any(&allowed_ids))
        )
        .set(impact.eq(new_impact))
        .execute(conn)
        .map_err(|e| format!("Failed to update impact: {}", e))?;
    
        Ok(updated_count)
    }

    pub fn delete_events_by_impact(conn: &mut SqliteConnection, target_impact: String, calendar_id: Option<i32>) -> Result<usize, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = cleanup_helpers::get_allowed_import_ids(conn, calendar_id)?;
    
        let deleted_count = diesel::delete(
            calendar_events
                .filter(impact.eq(target_impact))
                .filter(calendar_import_id.eq_any(&allowed_ids))
        )
        .execute(conn)
        .map_err(|e| format!("Failed to delete events: {}", e))?;
    
        Ok(deleted_count)
    }
}
