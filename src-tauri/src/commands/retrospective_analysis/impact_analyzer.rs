use super::impact_data_processor::ImpactDataProcessor;
use crate::services::pair_data::get_point_value;
use tracing::warn;

pub struct ImpactAnalyzer;

impl ImpactAnalyzer {
    pub async fn calculer(
        pair: &str,
        event_type: &str,
        events: &[crate::models::CalendarEvent],
        loader: &crate::services::DatabaseLoader,
    ) -> Result<super::types::EventImpactResult, String> {
        if events.is_empty() {
            return Err("No events found".into());
        }

        let date_min = events.first().ok_or("No first event")?.event_time.and_utc();
        let date_max = events.last().ok_or("No last event")?.event_time.and_utc();
        let candles_start = date_min - chrono::Duration::minutes(30);
        let candles_end = date_max + chrono::Duration::minutes(90);

        let all_candles = match loader.load_candles_by_pair(pair, "M1", candles_start, candles_end) {
            Ok(data) => data,
            Err(e) => {
                warn!("ImpactAnalyzer: échec chargement bougies {}: {}", pair, e);
                Vec::new()
            }
        };

        if all_candles.is_empty() {
            return Err(format!("No candle data for: {}", pair));
        }

        let data = ImpactDataProcessor::process(events, &all_candles);
        if data.event_count == 0 {
            return Err(format!(
                "No events with sufficient candle data: {}",
                event_type
            ));
        }

        let event_datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(data.avg_timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_else(|| "Unknown".into());

        let point_value = get_point_value(pair);

        Ok(super::types::EventImpactResult {
            atr_timeline_before: data.atr_timeline_before,
            atr_timeline_after: data.atr_timeline_after,
            body_timeline_before: data.body_timeline_before,
            body_timeline_after: data.body_timeline_after,
            noise_ratio_before: data.noise_before,
            noise_ratio_during: data.noise_during,
            noise_ratio_after: data.noise_after,
            volatility_increase_percent: data.volatility_increase,
            event_count: data.event_count,
            avg_deviation: data.avg_deviation,
            surprise_event_count: data.surprise_event_count,
            event_type: event_type.into(),
            pair: pair.into(),
            event_datetime,
            timezone_offset: "UTC+0".into(),
            point_value,
        })
    }
}
