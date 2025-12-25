use diesel::prelude::*;
use crate::commands::cleanup_commands::{RareEventSummary, CurrencySummary, OrphanEventSummary};
use crate::models::calendar_event::CalendarEvent;

pub struct CleanupService;

impl CleanupService {
    pub fn get_allowed_import_ids(conn: &mut SqliteConnection, target_calendar_id: Option<i32>) -> Result<Vec<i32>, String> {
        if let Some(target_id) = target_calendar_id {
            return Ok(vec![target_id]);
        }
    
        use crate::schema::calendar_imports::dsl::*;
        
        // Load all imports to filter by name in Rust
        let all_imports = calendar_imports
            .select((id, name))
            .load::<(i32, String)>(conn)
            .map_err(|e| format!("Database error loading imports: {}", e))?;
    
        // Filter out any import that contains "Planning Hebdo"
        let allowed: Vec<i32> = all_imports
            .into_iter()
            .filter(|(_, import_name)| !import_name.contains("Planning Hebdo"))
            .map(|(import_id, _)| import_id)
            .collect();
    
        Ok(allowed)
    }

    pub fn list_rare_events(conn: &mut SqliteConnection, min_occurrences: i64, calendar_id: Option<i32>) -> Result<Vec<RareEventSummary>, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = Self::get_allowed_import_ids(conn, calendar_id)?;
    
        let results = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids))
            .group_by(description)
            .select((description, diesel::dsl::count(id)))
            .load::<(String, i64)>(conn)
            .map_err(|e| format!("Database error: {}", e))?;
    
        let rare_events: Vec<RareEventSummary> = results
            .into_iter()
            .filter(|(_, count)| *count < min_occurrences)
            .map(|(desc, count)| RareEventSummary {
                description: desc,
                count,
            })
            .collect();
    
        Ok(rare_events)
    }

    pub fn delete_rare_events(conn: &mut SqliteConnection, min_occurrences: i64, calendar_id: Option<i32>) -> Result<usize, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = Self::get_allowed_import_ids(conn, calendar_id)?;
    
        let results = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids))
            .group_by(description)
            .select((description, diesel::dsl::count(id)))
            .load::<(String, i64)>(conn)
            .map_err(|e| format!("Database error: {}", e))?;
    
        let descriptions_to_delete: Vec<String> = results
            .into_iter()
            .filter(|(_, count)| *count < min_occurrences)
            .map(|(desc, _)| desc)
            .collect();
    
        if descriptions_to_delete.is_empty() {
            return Ok(0);
        }
    
        let deleted_count = diesel::delete(
            calendar_events
                .filter(description.eq_any(descriptions_to_delete))
                .filter(calendar_import_id.eq_any(&allowed_ids)),
        )
        .execute(conn)
        .map_err(|e| format!("Failed to delete events: {}", e))?;
    
        Ok(deleted_count)
    }

    pub fn list_currencies(conn: &mut SqliteConnection, calendar_id: Option<i32>) -> Result<Vec<CurrencySummary>, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = Self::get_allowed_import_ids(conn, calendar_id)?;
    
        let results = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids))
            .group_by(symbol)
            .select((symbol, diesel::dsl::count(id)))
            .load::<(String, i64)>(conn)
            .map_err(|e| format!("Database error: {}", e))?;
    
        let mut currencies: Vec<CurrencySummary> = results
            .into_iter()
            .map(|(sym, count)| {
                let country = Self::get_country_name(&sym);
                CurrencySummary {
                    symbol: sym.to_string(),
                    country_name: country,
                    count,
                }
            })
            .collect();
        
        currencies.sort_by(|a, b| a.country_name.cmp(&b.country_name));
    
        Ok(currencies)
    }

    fn get_country_name(symbol: &str) -> String {
        match symbol {
            "USD" => "États-Unis",
            "EUR" => "Zone Euro",
            "GBP" => "Royaume-Uni",
            "JPY" => "Japon",
            "AUD" => "Australie",
            "CAD" => "Canada",
            "CHF" => "Suisse",
            "NZD" => "Nouvelle-Zélande",
            "CNY" => "Chine",
            "HKD" => "Hong Kong",
            "SGD" => "Singapour",
            "TRY" => "Turquie",
            "ZAR" => "Afrique du Sud",
            "BRL" => "Brésil",
            "INR" => "Inde",
            "RUB" => "Russie",
            "KRW" => "Corée du Sud",
            "MXN" => "Mexique",
            "SEK" => "Suède",
            "NOK" => "Norvège",
            "DKK" => "Danemark",
            "PLN" => "Pologne",
            "HUF" => "Hongrie",
            "CZK" => "République Tchèque",
            "IDR" => "Indonésie",
            "THB" => "Thaïlande",
            "MYR" => "Malaisie",
            _ => "Autre",
        }.to_string()
    }

    pub fn delete_currency_events(conn: &mut SqliteConnection, currency_symbol: String, calendar_id: Option<i32>) -> Result<usize, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = Self::get_allowed_import_ids(conn, calendar_id)?;
    
        let deleted_count = diesel::delete(
            calendar_events
                .filter(symbol.eq(currency_symbol))
                .filter(calendar_import_id.eq_any(&allowed_ids)),
        )
        .execute(conn)
        .map_err(|e| format!("Failed to delete events: {}", e))?;
    
        Ok(deleted_count)
    }

    pub fn list_orphan_events(conn: &mut SqliteConnection, calendar_id: Option<i32>) -> Result<Vec<OrphanEventSummary>, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = Self::get_allowed_import_ids(conn, calendar_id)?;
    
        let empty_symbol_count: i64 = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids))
            .filter(symbol.eq(""))
            .count()
            .get_result(conn)
            .unwrap_or(0);
    
        let empty_desc_count: i64 = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids))
            .filter(description.eq(""))
            .count()
            .get_result(conn)
            .unwrap_or(0);
    
        let empty_impact_count: i64 = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids))
            .filter(impact.eq(""))
            .count()
            .get_result(conn)
            .unwrap_or(0);
    
        let mut summaries = Vec::new();
        if empty_symbol_count > 0 {
            summaries.push(OrphanEventSummary {
                reason: "Symbole (Pays) manquant".to_string(),
                count: empty_symbol_count,
            });
        }
        if empty_desc_count > 0 {
            summaries.push(OrphanEventSummary {
                reason: "Description manquante".to_string(),
                count: empty_desc_count,
            });
        }
        if empty_impact_count > 0 {
            summaries.push(OrphanEventSummary {
                reason: "Impact manquant".to_string(),
                count: empty_impact_count,
            });
        }
    
        Ok(summaries)
    }

    pub fn delete_orphan_events(conn: &mut SqliteConnection, calendar_id: Option<i32>) -> Result<usize, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = Self::get_allowed_import_ids(conn, calendar_id)?;
    
        let count1 = diesel::delete(
            calendar_events
                .filter(symbol.eq(""))
                .filter(calendar_import_id.eq_any(&allowed_ids)),
        )
        .execute(conn)
        .unwrap_or(0);
    
        let count2 = diesel::delete(
            calendar_events
                .filter(description.eq(""))
                .filter(calendar_import_id.eq_any(&allowed_ids)),
        )
        .execute(conn)
        .unwrap_or(0);
    
        let count3 = diesel::delete(
            calendar_events
                .filter(impact.eq(""))
                .filter(calendar_import_id.eq_any(&allowed_ids)),
        )
        .execute(conn)
        .unwrap_or(0);
    
        Ok(count1 + count2 + count3)
    }

    pub fn preview_cleanup_events(conn: &mut SqliteConnection, filter_type: String, filter_value: String, calendar_id: Option<i32>) -> Result<Vec<CalendarEvent>, String> {
        use crate::schema::calendar_events::dsl::*;
    
        let allowed_ids = Self::get_allowed_import_ids(conn, calendar_id)?;
        let limit = 100;
    
        let query = calendar_events
            .filter(calendar_import_id.eq_any(&allowed_ids))
            .limit(limit);
    
        let events = match filter_type.as_str() {
            "description" => query
                .filter(description.eq(filter_value))
                .load::<CalendarEvent>(conn),
            "symbol" => query
                .filter(symbol.eq(filter_value))
                .load::<CalendarEvent>(conn),
            "orphan_symbol" => query
                .filter(symbol.eq(""))
                .load::<CalendarEvent>(conn),
            "orphan_desc" => query
                .filter(description.eq(""))
                .load::<CalendarEvent>(conn),
            "orphan_impact" => query
                .filter(impact.eq(""))
                .load::<CalendarEvent>(conn),
            _ => return Err("Invalid filter type".to_string()),
        }
        .map_err(|e| format!("Database error: {}", e))?;
    
        Ok(events)
    }
}
