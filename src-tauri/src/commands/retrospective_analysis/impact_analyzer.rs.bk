use super::bidi_calculator::BidiCalculator;
use super::impact_data_processor::ImpactDataProcessor;
use crate::services::pair_data::get_point_value;

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

        let all_candles = loader
            .load_candles_by_pair(pair, "M1", date_min, date_max)
            .unwrap_or_default();

        if all_candles.is_empty() {
            return Err(format!("No candle data for: {}", pair));
        }

        let data = ImpactDataProcessor::process(events, &all_candles);

        let event_datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(data.avg_timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_else(|| "Unknown".into());

        // === CALCUL DES PARAMÈTRES BIDI POUR STRADDLE ===
        let point_value = get_point_value(pair);

        let bidi_params = BidiCalculator::calculer_depuis_impact(
            &data.atr_timeline_before,
            &data.atr_timeline_after,
            data.noise_during,
            data.volatility_increase,
            data.event_count,
            point_value,
            data.p95_wick,
        );

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
            event_type: event_type.into(),
            pair: pair.into(),
            event_datetime,
            timezone_offset: "UTC+0".into(),
            meilleur_moment: bidi_params.0,
            stop_loss: bidi_params.1,
            trailing_stop: bidi_params.2,
            timeout: bidi_params.3,
            offset: bidi_params.4,
            stop_loss_recovery: bidi_params.5,
            // Mapping des nouveaux paramètres Simultané
            stop_loss_simultaneous: bidi_params.6,
            trailing_stop_simultaneous: bidi_params.7,
            offset_simultaneous: bidi_params.8,
            stop_loss_recovery_simultaneous: bidi_params.9,
            point_value,
        })
    }
}
