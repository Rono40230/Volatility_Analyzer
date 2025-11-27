// Helpers pour analyze_quarter_entry_timing_command.rs
use std::collections::HashMap;
use chrono::Timelike;

/// Métriques calculées pour une minute spécifique
#[derive(Debug, Default)]
pub struct MinuteMetrics {
    pub atr_percent: f64,
    pub range_percent: f64,
    pub body_range: f64,
    pub noise_ratio: f64,
    pub volume_imbalance: f64,
    pub breakout_percent: f64,
}

/// Groupe les candles par jour (pour isoler chaque occurrence du quarter)
pub fn group_candles_by_day(
    candles: &[crate::models::Candle],
) -> Vec<Vec<crate::models::Candle>> {
    let mut daily_map: HashMap<String, Vec<crate::models::Candle>> = HashMap::new();

    for candle in candles {
        let date_key = candle.datetime.format("%Y-%m-%d").to_string();
        daily_map
            .entry(date_key)
            .or_insert_with(Vec::new)
            .push(candle.clone());
    }

    daily_map.into_values().collect()
}

/// Trouve la meilleure minute (offset 0-14) dans un jour donné
pub fn find_best_minute_in_quarter(
    daily_candles: &[crate::models::Candle],
) -> Result<u8, String> {
    if daily_candles.is_empty() {
        return Ok(0);
    }

    let mut best_score = -f64::INFINITY;
    let mut best_minute: u8 = 0;

    for offset in 0..15u8 {
        let minute_candles: Vec<_> = daily_candles
            .iter()
            .filter(|c| (c.datetime.minute() as u8 % 15) == offset)
            .collect();

        if minute_candles.is_empty() {
            continue;
        }

        let metrics = calculate_minute_metrics(&minute_candles)?;
        let score = score_metrics(&metrics);

        if score > best_score {
            best_score = score;
            best_minute = offset;
        }
    }

    Ok(best_minute)
}

/// Calcule les métriques pour une minute spécifique
pub fn calculate_minute_metrics(
    candles: &[&crate::models::Candle],
) -> Result<MinuteMetrics, String> {
    if candles.is_empty() {
        return Ok(MinuteMetrics::default());
    }

    let avg_close = candles.iter().map(|c| c.close).sum::<f64>() / candles.len() as f64;
    let price = if avg_close > 100.0 {
        100000.0
    } else if avg_close > 10.0 {
        10000.0
    } else {
        1.0
    };

    let mut atr_sum = 0.0;
    let mut range_sum = 0.0;
    let mut body_range_sum = 0.0;
    let mut noise_ratio_sum = 0.0;
    let mut volume_imbalance_sum = 0.0;
    let mut breakout_count = 0;

    for candle in candles {
        let atr = candle.high - candle.low;
        atr_sum += (atr / price) * 100.0;

        let range = candle.high - candle.low;
        range_sum += (range / price) * 100.0;

        let body = (candle.close - candle.open).abs();
        let wick = candle.high - candle.low;
        let body_range = if wick > 0.0 { (body / wick) * 100.0 } else { 0.0 };
        body_range_sum += body_range;

        let upper_wick = candle.high - candle.close.max(candle.open);
        let lower_wick = candle.open.min(candle.close) - candle.low;
        let total_wick = upper_wick + lower_wick;
        let noise_ratio = if body > 0.0 { total_wick / body } else { 0.0 };
        noise_ratio_sum += noise_ratio;

        volume_imbalance_sum += 0.15;

        if atr > range * 0.8 {
            breakout_count += 1;
        }
    }

    let count = candles.len() as f64;

    Ok(MinuteMetrics {
        atr_percent: atr_sum / count,
        range_percent: range_sum / count,
        body_range: body_range_sum / count,
        noise_ratio: noise_ratio_sum / count,
        volume_imbalance: volume_imbalance_sum / count,
        breakout_percent: (breakout_count as f64 / count) * 100.0,
    })
}

/// Score les métriques
pub fn score_metrics(m: &MinuteMetrics) -> f64 {
    let mut score = 0.0;
    // Range scoring
    score += if m.range_percent > 2.5 { 60.0 } else if m.range_percent > 2.0 { 50.0 } 
            else if m.range_percent > 1.5 { 40.0 } else if m.range_percent > 1.0 { 20.0 } else { 0.0 };
    // ATR scoring
    score += if m.atr_percent > 2.0 { 25.0 } else if m.atr_percent > 1.5 { 20.0 }
            else if m.atr_percent > 1.0 { 15.0 } else if m.atr_percent > 0.5 { 8.0 } else { 0.0 };
    // Body Range
    score += if m.body_range > 45.0 { 15.0 } else if m.body_range > 35.0 { 12.0 }
            else if m.body_range > 25.0 { 8.0 } else if m.body_range > 15.0 { 3.0 } else { 0.0 };
    // Noise Ratio
    score += if m.noise_ratio < 1.0 { 8.0 } else if m.noise_ratio < 1.5 { 5.0 }
            else if m.noise_ratio < 2.5 { 2.0 } else { 0.0 };
    // Volume + Breakout
    score += if m.volume_imbalance > 0.25 { 8.0 } else if m.volume_imbalance > 0.15 { 5.0 }
            else if m.volume_imbalance > 0.05 { 2.0 } else { 0.0 };
    score += if m.breakout_percent > 50.0 { 12.0 } else if m.breakout_percent > 25.0 { 8.0 }
            else if m.breakout_percent > 10.0 { 4.0 } else { 0.0 };
    score
}

/// Estime le win-rate pour une minute spécifique
pub fn estimate_win_rate_for_minute(
    daily_candles: &[crate::models::Candle],
) -> Result<f64, String> {
    if daily_candles.is_empty() {
        return Ok(0.5);
    }

    let volatility_sum: f64 = daily_candles
        .iter()
        .map(|c| (c.high - c.low) / c.open)
        .sum();
    let avg_volatility = volatility_sum / daily_candles.len() as f64;

    let win_rate = (0.5 + (avg_volatility * 50.0)).min(0.95).max(0.40);

    Ok(win_rate)
}

/// Calcule le score de confiance (basé sur la consistance des offsets)
pub fn calculate_confidence(offsets: &[u8], optimal: u8) -> f64 {
    if offsets.is_empty() {
        return 0.0;
    }

    let optimal_i32 = optimal as i32;
    let variance: f64 = offsets
        .iter()
        .map(|&o| {
            let diff = (o as i32 - optimal_i32).abs() as f64;
            diff * diff
        })
        .sum::<f64>()
        / offsets.len() as f64;

    let std_dev = variance.sqrt();
    (100.0 - (std_dev * 10.0)).max(20.0).min(100.0)
}
