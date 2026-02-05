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

        // 3. Fetch known event counts from history
        let known_events = self.fetch_known_event_counts()?;

        let mut projected = Vec::new();

        for event in events {
            let occurrence_count = known_events.get(&event.description).cloned().unwrap_or(0);
            let has_history = occurrence_count > 0;

            // Find best matching archive
            if let Some(mut match_data) = self.find_best_match(&event, &archives) {
                match_data.has_history = has_history;
                match_data.occurrence_count = occurrence_count;
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
                    offset_simultaneous: 0.0,
                    tp_simultaneous: 0.0,
                    sl_simultaneous: 0.0,
                    confidence_score: 0.0,
                    source: "None".to_string(),
                    has_history,
                    occurrence_count,
                });
            }
        }

        Ok(projected)
    }

    fn fetch_known_event_counts(&self) -> Result<std::collections::HashMap<String, i64>, String> {
        use diesel::dsl::count;
        
        let mut conn = self.calendar_pool.get().map_err(|e| e.to_string())?;
        
        let now = Utc::now().naive_utc();
        
        let results = calendar_events::table
            .filter(calendar_events::event_time.lt(now))
            .group_by(calendar_events::description)
            .select((calendar_events::description, count(calendar_events::id)))
            .load::<(String, i64)>(&mut conn)
            .map_err(|e| e.to_string())?;
            
        Ok(results.into_iter().collect())
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
            .filter(calendar_events::impact.eq_any(vec!["High", "Medium"]))
            .order(calendar_events::event_time.asc())
            .select(CalendarEvent::as_select())
            .load(&mut conn)
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
                    if self.events_match(&event.description, arch_evt)
                        && self.pair_matches_currency(pair, &event.symbol)
                    {
                        // Try to get confidence score from various possible locations
                        let score = data.get("confidence_score").and_then(|v| v.as_f64())
                            .or_else(|| data.get("recommendation").and_then(|r| r.get("score").and_then(|v| v.as_f64())))
                            .unwrap_or(0.0);
                        
                        if score > best_score {
                            best_score = score;
                            
                            // Extract params (support both legacy straddle_params and new root-level format)
                            let (offset, tp, sl) = if let Some(params) = data.get("straddle_params") {
                                (
                                    params.get("offset").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                    params.get("tp").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                    params.get("sl").and_then(|v| v.as_f64()).unwrap_or(0.0)
                                )
                            } else {
                                (
                                    data.get("offset").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                    data.get("trailingStop").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                    data.get("stopLoss").and_then(|v| v.as_f64()).unwrap_or(0.0)
                                )
                            };

                            // Extract simultaneous params
                            let offset_simultaneous = data.get("offsetSimultaneous").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let tp_simultaneous = data.get("trailingStopSimultaneous").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let sl_simultaneous = data.get("stopLossSimultaneous").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            
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
                                offset_simultaneous,
                                tp_simultaneous,
                                sl_simultaneous,
                                confidence_score: score,
                                source: "Archive".to_string(),
                                has_history: false, // Will be updated in caller
                                occurrence_count: 0, // Will be updated in caller
                            });
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
