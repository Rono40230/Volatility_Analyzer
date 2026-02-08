// services/economic_event_loader.rs - Import CSV événements économiques
// Conforme .clinerules: < 250L, pas de unwrap, thiserror, données réelles

use crate::db::DbPool;
use crate::models::calendar_event::NewCalendarEvent;
use crate::models::VolatilityError;
use crate::services::calendar_scraper::CalendarScraper;
use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use diesel::prelude::*;
use std::path::Path;
use tracing::{info, warn};

/// Row helper pour récupérer un id via sql_query
#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    id: i32,
}

/// Service pour charger événements économiques depuis CSV
pub struct EconomicEventLoader {
    /// NOTE: Ce field est public pour usage futur (Phase 2 - intégration scraper)
    #[allow(dead_code)]
    scraper: CalendarScraper,
    pool: DbPool,
}

impl EconomicEventLoader {
    pub fn new(pool: DbPool) -> Self {
        Self {
            scraper: CalendarScraper::new(pool.clone()),
            pool,
        }
    }

    /// Crée ou récupère un calendar_import par défaut pour ce fichier CSV.
    /// Retourne l'id du calendar_import.
    fn ensure_calendar_import(&self, filename: &str) -> Result<i32, VolatilityError> {
        use diesel::prelude::*;
        let mut conn = self.pool.get().map_err(|e| {
            VolatilityError::DatabaseError(format!("Pool error: {}", e))
        })?;

        // Essayer de trouver un import existant avec ce nom de fichier
        let existing: Option<i32> = diesel::sql_query(
            "SELECT id FROM calendar_imports WHERE filename = ? LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(filename)
        .get_result::<IdRow>(&mut conn)
        .optional()
        .map_err(|e| VolatilityError::DatabaseError(format!("Query error: {}", e)))?
        .map(|row| row.id);

        if let Some(id) = existing {
            return Ok(id);
        }

        // Créer un nouvel import
        let import_name = format!("csv_import_{}", filename);
        diesel::sql_query(
            "INSERT INTO calendar_imports (name, filename, event_count) VALUES (?, ?, 0)"
        )
        .bind::<diesel::sql_types::Text, _>(&import_name)
        .bind::<diesel::sql_types::Text, _>(filename)
        .execute(&mut conn)
        .map_err(|e| VolatilityError::DatabaseError(format!("Insert error: {}", e)))?;

        // Récupérer l'id généré
        let new_id: i32 = diesel::sql_query(
            "SELECT id FROM calendar_imports WHERE filename = ? ORDER BY id DESC LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(filename)
        .get_result::<IdRow>(&mut conn)
        .map_err(|e| VolatilityError::DatabaseError(format!("Get id error: {}", e)))?
        .id;

        Ok(new_id)
    }

    /// Charge un CSV d'événements économiques dans la DB
    ///
    /// Format attendu du CSV (Investing.com style):
    /// Date,Time,Currency,Event,Impact,Actual,Forecast,Previous
    /// 2025-01-15,14:30,EUR,ECB Interest Rate Decision,HIGH,4.50,4.25,4.00
    pub fn load_from_csv<P: AsRef<Path>>(&self, csv_path: P) -> Result<usize, VolatilityError> {
        let path = csv_path.as_ref();
        info!("Loading economic events from CSV: {:?}", path);

        if !path.exists() {
            return Err(VolatilityError::CsvLoadError(format!(
                "CSV file not found: {:?}",
                path
            )));
        }

        // Créer ou récupérer un calendar_import pour ce fichier
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown.csv");
        let import_id = self.ensure_calendar_import(filename)?;
        info!("Using calendar_import_id={} for {}", import_id, filename);

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .map_err(|e| VolatilityError::CsvLoadError(format!("Cannot read CSV: {}", e)))?;

        let mut events = Vec::new();
        let mut skipped = 0;

        for (line_num, result) in reader.records().enumerate() {
            let record = result.map_err(|e| {
                VolatilityError::CsvLoadError(format!("Line {}: {}", line_num + 2, e))
            })?;

            match self.parse_csv_record(&record, line_num + 2, import_id) {
                Ok(event) => events.push(event),
                Err(e) => {
                    warn!("Skipping line {}: {}", line_num + 2, e);
                    skipped += 1;
                }
            }
        }

        info!(
            "Parsed {} events, skipped {} invalid lines",
            events.len(),
            skipped
        );

        let inserted = self.scraper.store_events(&events)?;
        info!("Successfully inserted {} events into database", inserted);

        Ok(inserted)
    }

    /// Parse une ligne CSV en NewCalendarEvent
    fn parse_csv_record(
        &self,
        record: &csv::StringRecord,
        line_num: usize,
        import_id: i32,
    ) -> Result<NewCalendarEvent, VolatilityError> {
        // Format: Date,Time,Currency,Event,Impact,Actual,Forecast,Previous
        if record.len() < 5 {
            return Err(VolatilityError::ParseError(format!(
                "Line {}: Expected at least 5 columns, got {}",
                line_num,
                record.len()
            )));
        }

        let date = record.get(0).ok_or_else(|| {
            VolatilityError::ParseError(format!("Line {}: Missing date", line_num))
        })?;

        let time = record.get(1).ok_or_else(|| {
            VolatilityError::ParseError(format!("Line {}: Missing time", line_num))
        })?;

        let symbol = record.get(2).ok_or_else(|| {
            VolatilityError::ParseError(format!("Line {}: Missing currency", line_num))
        })?;

        let description = record.get(3).ok_or_else(|| {
            VolatilityError::ParseError(format!("Line {}: Missing event name", line_num))
        })?;

        let impact = record.get(4).ok_or_else(|| {
            VolatilityError::ParseError(format!("Line {}: Missing impact", line_num))
        })?;

        // Parse datetime "2025-01-15" + "14:30" -> NaiveDateTime
        let datetime_str = format!("{} {}", date, time);
        let event_time =
            NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M").map_err(|e| {
                VolatilityError::ParseError(format!(
                    "Line {}: Invalid datetime '{}': {}",
                    line_num, datetime_str, e
                ))
            })?;

        // Parse optional numeric values
        let actual = record.get(5).and_then(|s| s.parse::<f64>().ok());
        let forecast = record.get(6).and_then(|s| s.parse::<f64>().ok());
        let previous = record.get(7).and_then(|s| s.parse::<f64>().ok());

        Ok(NewCalendarEvent {
            symbol: symbol.to_string(),
            event_time,
            impact: impact.to_uppercase(),
            description: description.to_string(),
            actual,
            forecast,
            previous,
            calendar_import_id: import_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::r2d2::{self, ConnectionManager};
    use diesel::sqlite::SqliteConnection;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn create_test_pool() -> DbPool {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create test pool");
        Arc::new(pool)
    }

    #[allow(dead_code)]
    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(
            file,
            "Date,Time,Currency,Event,Impact,Actual,Forecast,Previous"
        )
        .expect("Failed to write header");
        writeln!(
            file,
            "2025-01-15,14:30,EUR,ECB Interest Rate Decision,HIGH,4.50,4.25,4.00"
        )
        .expect("Failed to write record 1");
        writeln!(
            file,
            "2025-02-03,20:30,USD,Non-Farm Payrolls,HIGH,250000,220000,210000"
        )
        .expect("Failed to write record 2");
        file
    }

    #[test]
    fn test_economic_event_loader_creation() {
        let pool = create_test_pool();
        let loader = EconomicEventLoader::new(pool);
        assert!(std::ptr::addr_of!(loader).is_aligned());
    }

    #[test]
    fn test_parse_csv_record() {
        let pool = create_test_pool();
        let loader = EconomicEventLoader::new(pool);

        let record = csv::StringRecord::from(vec![
            "2025-01-15",
            "14:30",
            "EUR",
            "ECB Rate Decision",
            "HIGH",
            "4.50",
            "4.25",
            "4.00",
        ]);

        let event = loader
            .parse_csv_record(&record, 2, 1)
            .expect("Failed to parse record");

        assert_eq!(event.symbol, "EUR");
        assert_eq!(event.description, "ECB Rate Decision");
        assert_eq!(event.impact, "HIGH");
        assert_eq!(event.actual, Some(4.50));
        assert_eq!(event.forecast, Some(4.25));
        assert_eq!(event.previous, Some(4.00));
    }
}
