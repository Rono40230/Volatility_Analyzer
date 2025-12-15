use super::helpers::calculer_atr;
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
}

impl ImpactDataProcessor {
    pub fn process(
        events: &[crate::models::CalendarEvent],
        all_candles: &[crate::models::Candle],
    ) -> ProcessedImpactData {
        let mut atr_before_sum = vec![0.0; 30];
        let mut atr_after_sum = vec![0.0; 90];
        let mut body_before_sum = vec![0.0; 30];
        let mut body_after_sum = vec![0.0; 90];
        let mut counts_before = vec![0usize; 30];
        let mut counts_after = vec![0usize; 90];
        let mut noise_before_sum = 0.0;
        let mut noise_during_sum = 0.0;
        let mut noise_after_sum = 0.0;
        let mut event_count = 0;

        for event in events {
            event_count += 1;
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

            let mut atrs = Vec::new();
            let mut bodies = Vec::new();
            for candle in &occurrence_candles {
                let atr = calculer_atr(candle.high, candle.low, candle.close);
                let range = candle.high - candle.low;
                let body = (candle.close - candle.open).abs();
                let body_pct = if range > 0.0 {
                    (body / range) * 100.0
                } else {
                    0.0
                };
                atrs.push(atr);
                bodies.push(body_pct);
            }

            let event_index = 30;

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

            for i in 0..event_index.min(atrs.len()) {
                noise_before_sum += if bodies[i] > 0.0 {
                    100.0 / bodies[i]
                } else {
                    1.0
                };
            }
            if event_index < atrs.len() {
                noise_during_sum += if bodies[event_index] > 0.0 {
                    100.0 / bodies[event_index]
                } else {
                    1.0
                };
            }
            for i in (event_index + 1)..atrs.len().min(event_index + 90) {
                noise_after_sum += if bodies[i] > 0.0 {
                    100.0 / bodies[i]
                } else {
                    1.0
                };
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

        let avg_timestamp = if event_count > 0 {
            events
                .iter()
                .take(event_count)
                .map(|e| e.event_time.and_utc().timestamp())
                .sum::<i64>()
                / event_count as i64
        } else {
            0
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
        }
    }
}
