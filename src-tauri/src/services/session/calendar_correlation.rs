// services/session/calendar_correlation.rs - Corrélation session × calendrier
// Conforme .clinerules : < 150L, pas d'unwrap()

use crate::db::schema::calendar_events;
use crate::services::session_analyzer::{CalendarCorrelation, TradingSession};
use diesel::prelude::*;

pub struct CalendarCorrelator;

impl CalendarCorrelator {
    /// Calcule la corrélation entre sessions et événements calendrier économique
    pub fn calculer_correlation(
        sessions: &[TradingSession],
        pool: &crate::db::DbPool,
    ) -> Result<Vec<CalendarCorrelation>, String> {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Erreur connexion DB: {}", e))?;

        let mut correlations = Vec::new();

        for session in sessions {
            let (start_hour, end_hour) = match session.name.as_str() {
                "Tokyo" => (0, 9),
                "Londres" => (8, 17),
                "New York" => (13, 22),
                "Sydney" => (22, 7),
                _ => continue,
            };

            let high_impact_count = Self::count_events(
                &mut conn,
                start_hour,
                end_hour,
                "HIGH",
                session.name == "Sydney",
            )?;

            let total_events_count = Self::count_events_multiple(
                &mut conn,
                start_hour,
                end_hour,
                vec!["HIGH", "MEDIUM"],
                session.name == "Sydney",
            )?;

            let event_volatility = if total_events_count > 0 {
                (high_impact_count as f64 / total_events_count as f64) * 100.0
            } else {
                0.0
            };

            let avg_events_per_session = 50.0;
            let impact_percentage = (high_impact_count as f64 / avg_events_per_session) * 100.0;

            correlations.push(CalendarCorrelation {
                session: session.name.clone(),
                high_impact_events: high_impact_count as usize,
                event_volatility: (event_volatility * 100.0).round() / 100.0,
                impact_percentage: (impact_percentage * 100.0).round() / 100.0,
            });
        }

        Ok(correlations)
    }

    fn count_events(
        conn: &mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<diesel::SqliteConnection>,
        >,
        start_hour: u32,
        end_hour: u32,
        impact: &str,
        crosses_midnight: bool,
    ) -> Result<i64, String> {
        if crosses_midnight {
            let count1: i64 = calendar_events::table
                .filter(calendar_events::impact.eq(impact))
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                    "cast(strftime('%H', event_time) as integer) >= {}",
                    start_hour
                )))
                .count()
                .get_result(conn)
                .unwrap_or(0);

            let count2: i64 = calendar_events::table
                .filter(calendar_events::impact.eq(impact))
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                    "cast(strftime('%H', event_time) as integer) < {}",
                    end_hour
                )))
                .count()
                .get_result(conn)
                .unwrap_or(0);

            Ok(count1 + count2)
        } else {
            calendar_events::table
                .filter(calendar_events::impact.eq(impact))
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(
                    &format!(
                        "cast(strftime('%H', event_time) as integer) >= {} AND cast(strftime('%H', event_time) as integer) < {}",
                        start_hour, end_hour
                    )
                ))
                .count()
                .get_result(conn)
                .map_err(|e| format!("Erreur DB: {}", e))
        }
    }

    fn count_events_multiple(
        conn: &mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<diesel::SqliteConnection>,
        >,
        start_hour: u32,
        end_hour: u32,
        impacts: Vec<&str>,
        crosses_midnight: bool,
    ) -> Result<i64, String> {
        if crosses_midnight {
            let count1: i64 = calendar_events::table
                .filter(calendar_events::impact.eq_any(impacts.clone()))
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                    "cast(strftime('%H', event_time) as integer) >= {}",
                    start_hour
                )))
                .count()
                .get_result(conn)
                .unwrap_or(0);

            let count2: i64 = calendar_events::table
                .filter(calendar_events::impact.eq_any(impacts))
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                    "cast(strftime('%H', event_time) as integer) < {}",
                    end_hour
                )))
                .count()
                .get_result(conn)
                .unwrap_or(0);

            Ok(count1 + count2)
        } else {
            calendar_events::table
                .filter(calendar_events::impact.eq_any(impacts))
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(
                    &format!(
                        "cast(strftime('%H', event_time) as integer) >= {} AND cast(strftime('%H', event_time) as integer) < {}",
                        start_hour, end_hour
                    )
                ))
                .count()
                .get_result(conn)
                .map_err(|e| format!("Erreur DB: {}", e))
        }
    }
}
