use crate::db::DbPool;
use crate::models::planning::ProjectedEvent;
use crate::models::calendar_event::CalendarEvent;
use crate::models::archive::Archive;
use crate::services::archive_service::ArchiveService;
use crate::schema::calendar_events;
use diesel::prelude::*;
use diesel::SelectableHelper;
use chrono::{DateTime, Utc};

pub struct ProjectionEngine {
    calendar_pool: DbPool,
    archive_service: ArchiveService,
}

impl ProjectionEngine {
    pub fn new(calendar_pool: DbPool, archive_service: ArchiveService) -> Self {
        Self {
            calendar_pool,
            archive_service,
        }
    }

    pub async fn project_stats(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ProjectedEvent>, String> {
        // 1. Fetch calendar events
        let events = self.fetch_calendar_events(start_date, end_date)?;

        // 2. Fetch all archives
        let archives = self.archive_service.list_archives()?;

        let mut projected = Vec::new();

        for event in events {
            // Find best matching archive
            if let Some(match_data) = self.find_best_match(&event, &archives) {
                projected.push(match_data);
            } else {
                // Add event without projection
                projected.push(ProjectedEvent {
                    id: event.id.to_string(),
                    time: event.event_time.to_string(),
                    name: event.description.clone(),
                    currency: event.symbol.clone(),
                    impact: event.impact.clone(),
                    pair: "N/A".to_string(),
                    offset: 0.0,
                    tp: 0.0,
                    sl: 0.0,
                    confidence_score: 0.0,
                    source: "None".to_string(),
                });
            }
        }

        Ok(projected)
    }

    fn fetch_calendar_events(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, String> {
        let mut conn = self.calendar_pool.get().map_err(|e| e.to_string())?;

        let start_naive = start_date.naive_utc();
        let end_naive = end_date.naive_utc();

        calendar_events::table
            .filter(calendar_events::event_time.ge(start_naive))
            .filter(calendar_events::event_time.le(end_naive))
            .order(calendar_events::event_time.asc())
            .select(CalendarEvent::as_select())
            .load::<CalendarEvent>(&mut conn)
            .map_err(|e| e.to_string())
    }

    fn find_best_match(&self, event: &CalendarEvent, archives: &[Archive]) -> Option<ProjectedEvent> {
        let mut best_match: Option<ProjectedEvent> = None;
        let mut best_score = -1.0;

        for archive in archives {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&archive.data_json) {
                let archive_event_type = data.get("event_type").and_then(|v| v.as_str());
                let archive_pair = data.get("symbol").and_then(|v| v.as_str());
                
                if let (Some(arch_evt), Some(pair)) = (archive_event_type, archive_pair) {
                    if self.events_match(&event.description, arch_evt) {
                        if self.pair_matches_currency(pair, &event.symbol) {
                            // Try to get confidence score from various possible locations
                            let score = data.get("confidence_score").and_then(|v| v.as_f64())
                                .or_else(|| data.get("recommendation").and_then(|r| r.get("score").and_then(|v| v.as_f64())))
                                .unwrap_or(0.0);
                            
                            if score > best_score {
                                best_score = score;
                                
                                // Extract params from straddle_params or similar
                                let params = data.get("straddle_params");
                                let offset = params.and_then(|p| p.get("offset").and_then(|v| v.as_f64())).unwrap_or(0.0);
                                let tp = params.and_then(|p| p.get("tp").and_then(|v| v.as_f64())).unwrap_or(0.0);
                                let sl = params.and_then(|p| p.get("sl").and_then(|v| v.as_f64())).unwrap_or(0.0);
                                
                                best_match = Some(ProjectedEvent {
                                    id: event.id.to_string(),
                                    time: event.event_time.to_string(),
                                    name: event.description.clone(),
                                    currency: event.symbol.clone(),
                                    impact: event.impact.clone(),
                                    pair: pair.to_string(),
                                    offset,
                                    tp,
                                    sl,
                                    confidence_score: score,
                                    source: "Archive".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        best_match
    }

    fn events_match(&self, calendar_name: &str, archive_name: &str) -> bool {
        let c = calendar_name.to_lowercase();
        let a = archive_name.to_lowercase();
        c.contains(&a) || a.contains(&c)
    }

    fn pair_matches_currency(&self, pair: &str, currency: &str) -> bool {
        pair.contains(currency)
    }
}
