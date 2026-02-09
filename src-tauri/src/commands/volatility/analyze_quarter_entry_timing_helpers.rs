// Helpers pour analyze_quarter_entry_timing_command.rs
use crate::commands::volatility::minute_scoring::{
    calculer_confiance as calculate_confidence_fn, noter_metriques as score_metrics_fn,
};
use chrono::Timelike;
use std::collections::HashMap;

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
pub fn group_candles_by_day(candles: &[crate::models::Candle]) -> Vec<Vec<crate::models::Candle>> {
    let mut daily_map: HashMap<String, Vec<crate::models::Candle>> = HashMap::new();

    for candle in candles {
        let date_key = candle.datetime.format("%Y-%m-%d").to_string();
        daily_map.entry(date_key).or_default().push(candle.clone());
    }

    daily_map.into_values().collect()
}

/// Trouve la meilleure minute (offset 0-14) dans un jour donné
pub fn find_best_minute_in_quarter(daily_candles: &[crate::models::Candle]) -> Result<u8, String> {
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

        let metrics = calculer_metriques_minute(&minute_candles)?;
        let score = score_metrics_fn(&metrics);

        if score > best_score {
            best_score = score;
            best_minute = offset;
        }
    }

    Ok(best_minute)
}

/// Calcule les métriques pour une minute spécifique
pub fn calculer_metriques_minute(
    candles: &[&crate::models::Candle],
) -> Result<MinuteMetrics, String> {
    if candles.is_empty() {
        return Ok(MinuteMetrics::default());
    }

    let symbol = candles[0].symbol.as_str();
    let asset_props = crate::services::pair_data::symbol_properties::get_asset_properties(symbol);

    let mut atr_sum = 0.0;
    let mut range_sum = 0.0;
    let mut body_range_sum = 0.0;
    let mut noise_ratio_sum = 0.0;
    let mut volume_imbalance_sum = 0.0;
    let mut breakout_count = 0;

    for candle in candles {
        let atr = asset_props.normalize(candle.high - candle.low);
        atr_sum += atr;

        let range = asset_props.normalize(candle.high - candle.low);
        range_sum += range;

        let body = (candle.close - candle.open).abs();
        let wick = candle.high - candle.low;
        let body_range = if wick > 0.0 {
            (body / wick) * 100.0
        } else {
            0.0
        };
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

    let win_rate = (0.5 + (avg_volatility * 50.0)).clamp(0.40, 0.95);

    Ok(win_rate)
}

/// Calcule le score de confiance (basé sur la consistance)
pub fn calculer_confiance(offsets: &[u8], optimal: u8) -> f64 {
    calculate_confidence_fn(offsets, optimal)
}
