use super::helpers::calculer_atr;
use super::helpers::{calculate_p95, calculate_avg_timestamp};
use chrono::Duration;

pub struct ImpactDataProcessor;

pub struct ProcessedImpactData {
    pub atr_timeline_before: Vec<f64>,
    pub atr_timeline_after: Vec<f64>,
    pub body_timeline_before: Vec<f64>,
    pub body_timeline_after: Vec<f64>,
    pub noise_before: f64,
    pub noise_during: f64,
    pub noise_after: f64,
    pub volatility_increase: f64,
    pub event_count: usize,
    pub avg_timestamp: i64,
    pub p95_wick: f64,
    pub p95_range: f64,
    pub avg_deviation: f64,
    pub surprise_event_count: usize,
}

impl ImpactDataProcessor {
    pub fn process(
        events: &[crate::models::CalendarEvent],
        all_candles: &[crate::models::Candle],
    ) -> ProcessedImpactData {
        let (mut atr_before_sum, mut body_before_sum, mut counts_before) = (vec![0.0; 30], vec![0.0; 30], vec![0usize; 30]);
        let (mut atr_after_sum, mut body_after_sum, mut counts_after) = (vec![0.0; 90], vec![0.0; 90], vec![0usize; 90]);
        let (mut noise_before_sum, mut noise_during_sum, mut noise_after_sum) = (0.0, 0.0, 0.0);
        let (mut event_count, mut deviation_count, mut surprise_event_count) = (0, 0, 0);
        let (mut all_wicks, mut all_ranges, mut total_deviation) = (Vec::new(), Vec::new(), 0.0);

        for event in events {
            event_count += 1;
            
            // Calcul de la déviation
            if let (Some(actual), Some(forecast)) = (event.actual, event.forecast) {
                let deviation = (actual - forecast).abs();
                total_deviation += deviation;
                deviation_count += 1;
                if deviation > 0.0 {
                    surprise_event_count += 1;
                }
            }

            let event_time = event.event_time.and_utc();
            let window_start = event_time - Duration::minutes(30);
            let window_end = event_time + Duration::minutes(90);

            let occurrence_candles: Vec<_> = all_candles
                .iter()
                .filter(|c| c.datetime >= window_start && c.datetime <= window_end)
                .collect();

            if occurrence_candles.len() < 120 {
                continue;
            }

            let (mut atrs, mut bodies) = (Vec::new(), Vec::new());
            for candle in &occurrence_candles {
                atrs.push(calculer_atr(candle.high, candle.low, candle.close));
                let range = candle.high - candle.low;
                bodies.push(if range > 0.0 { (candle.close - candle.open).abs() / range * 100.0 } else { 0.0 });
            }

            // Collect wicks for P95 calculation (T-5 to T+15)
            // event_index is 30 (T0)
            let event_index: usize = 30;
            let wick_start_idx = event_index.saturating_sub(5);
            let wick_end_idx = (event_index + 15).min(occurrence_candles.len());
            
            for i in wick_start_idx..wick_end_idx {
                if let Some(candle) = occurrence_candles.get(i) {
                    let upper_wick = candle.high - candle.close.max(candle.open);
                    let lower_wick = candle.open.min(candle.close) - candle.low;
                    let range = candle.high - candle.low;
                    if upper_wick > 0.0 { all_wicks.push(upper_wick); }
                    if lower_wick > 0.0 { all_wicks.push(lower_wick); }
                    if range > 0.0 { all_ranges.push(range); }
                }
            }

            for i in 0..event_index.min(atrs.len()) {
                atr_before_sum[i] += atrs[i];
                body_before_sum[i] += bodies[i];
                counts_before[i] += 1;
            }

            for i in event_index..atrs.len().min(event_index + 90) {
                let idx = i - event_index;
                atr_after_sum[idx] += atrs[i];
                body_after_sum[idx] += bodies[i];
                counts_after[idx] += 1;
            }

            let calc_noise = |body: f64| if body > 0.0 { 100.0 / body } else { 1.0 };

            for i in 0..event_index.min(atrs.len()) {
                noise_before_sum += calc_noise(bodies[i]);
            }
            if event_index < atrs.len() {
                noise_during_sum += calc_noise(bodies[event_index]);
            }
            for i in (event_index + 1)..atrs.len().min(event_index + 90) {
                noise_after_sum += calc_noise(bodies[i]);
            }
        }

        let mut atr_timeline_before = vec![0.0; 30];
        let mut atr_timeline_after = vec![0.0; 90];
        let mut body_timeline_before = vec![0.0; 30];
        let mut body_timeline_after = vec![0.0; 90];

        for i in 0..30 {
            if counts_before[i] > 0 {
                atr_timeline_before[i] = atr_before_sum[i] / counts_before[i] as f64;
                body_timeline_before[i] = body_before_sum[i] / counts_before[i] as f64;
            }
        }

        for i in 0..90 {
            if counts_after[i] > 0 {
                atr_timeline_after[i] = atr_after_sum[i] / counts_after[i] as f64;
                body_timeline_after[i] = body_after_sum[i] / counts_after[i] as f64;
            }
        }

        let noise_before = if event_count > 0 {
            noise_before_sum / (event_count as f64 * 30.0)
        } else {
            0.0
        };
        let noise_during = if event_count > 0 {
            noise_during_sum / event_count as f64
        } else {
            0.0
        };
        let noise_after = if event_count > 0 {
            noise_after_sum / (event_count as f64 * 89.0)
        } else {
            0.0
        };

        let atr_mean_before = atr_timeline_before.iter().sum::<f64>() / 30.0;
        let atr_mean_after = atr_timeline_after.iter().sum::<f64>() / 90.0;
        let volatility_increase = if atr_mean_before > 0.0 {
            ((atr_mean_after - atr_mean_before) / atr_mean_before) * 100.0
        } else {
            0.0
        };

        let avg_timestamp = calculate_avg_timestamp(events, event_count);

        // Calculate P95 Wick & Range
        let p95_wick = calculate_p95(&mut all_wicks);
        let p95_range = calculate_p95(&mut all_ranges);

        let avg_deviation = if deviation_count > 0 {
            total_deviation / deviation_count as f64
        } else {
            0.0
        };

        ProcessedImpactData {
            atr_timeline_before,
            atr_timeline_after,
            body_timeline_before,
            body_timeline_after,
            noise_before,
            noise_during,
            noise_after,
            volatility_increase,
            event_count,
            avg_timestamp,
            p95_wick,
            p95_range,
            avg_deviation,
            surprise_event_count,
        }
    }
}
