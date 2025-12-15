use crate::models::{Candle, Result, VolatilityError};
use crate::services::metrics::MetricsCalculator;
use chrono::Duration;

/// Niveau de volatilité
#[derive(Debug, Clone, PartialEq)]
pub enum VolatilityLevel {
    Low,
    Medium,
    High,
}

/// Résultat de l'analyse ATR contextuelle
#[derive(Debug, Clone)]
pub struct ContextualAtrMetrics {
    pub atr_before_event: f64,
    pub atr_after_event: f64,
    pub atr_ratio: f64,
    pub max_atr_spike: f64,
    #[allow(dead_code)]
    pub minutes_to_peak: i64,
    pub recommended_sl_multiplier: f64,
    pub recommended_tp_multiplier: f64,
    pub baseline_volatility_level: VolatilityLevel,
}

/// Calcule l'ATR baseline (30min avant événement)
pub fn calculer_atr_reference(
    candles: &[Candle],
    event_time: chrono::DateTime<chrono::Utc>,
    atr_period: usize,
) -> Result<f64> {
    let start_time = event_time - Duration::minutes(30);
    let end_time = event_time;

    let baseline_candles: Vec<Candle> = candles
        .iter()
        .filter(|c| c.datetime >= start_time && c.datetime < end_time)
        .cloned()
        .collect();

    if baseline_candles.len() < atr_period {
        return Err(VolatilityError::InsufficientData(format!(
            "Need {} candles for ATR, got {}",
            atr_period,
            baseline_candles.len()
        )));
    }

    let calculator = MetricsCalculator::new(&baseline_candles);
    let atr_values = calculator.calculer_atr(atr_period)?;

    atr_values
        .last()
        .copied()
        .ok_or_else(|| VolatilityError::InsufficientData("No ATR values".to_string()))
}

/// Calcule l'ATR post-événement (30min après)
pub fn calculer_atr_post_evenement(
    candles: &[Candle],
    event_time: chrono::DateTime<chrono::Utc>,
    atr_period: usize,
) -> Result<f64> {
    let start_time = event_time;
    let end_time = event_time + Duration::minutes(30);

    let post_candles: Vec<Candle> = candles
        .iter()
        .filter(|c| c.datetime >= start_time && c.datetime < end_time)
        .cloned()
        .collect();

    if post_candles.len() < atr_period {
        return Err(VolatilityError::InsufficientData(format!(
            "Need {} candles for post-event ATR, got {}",
            atr_period,
            post_candles.len()
        )));
    }

    let calculator = MetricsCalculator::new(&post_candles);
    let atr_values = calculator.calculer_atr(atr_period)?;

    atr_values
        .last()
        .copied()
        .ok_or_else(|| VolatilityError::InsufficientData("No ATR values".to_string()))
}

/// Trouve le pic maximal d'ATR dans les 2h post-événement
pub fn trouver_pic_atr_max(
    candles: &[Candle],
    event_time: chrono::DateTime<chrono::Utc>,
    atr_period: usize,
) -> Result<(f64, i64)> {
    let end_time = event_time + Duration::minutes(120);

    let post_candles: Vec<Candle> = candles
        .iter()
        .filter(|c| c.datetime >= event_time && c.datetime < end_time)
        .cloned()
        .collect();

    if post_candles.len() < atr_period + 10 {
        return Err(VolatilityError::InsufficientData(
            "Not enough candles to find ATR spike".to_string(),
        ));
    }

    let mut max_atr = 0.0;
    let mut max_index = 0;

    for i in atr_period..post_candles.len() {
        let window = &post_candles[i - atr_period..i];
        let calculator = MetricsCalculator::new(window);

        if let Ok(atr_values) = calculator.calculer_atr(atr_period) {
            if let Some(&atr) = atr_values.last() {
                if atr > max_atr {
                    max_atr = atr;
                    max_index = i;
                }
            }
        }
    }

    let minutes_to_peak = if max_index > 0 {
        let peak_time = post_candles[max_index].datetime;
        (peak_time - event_time).num_minutes()
    } else {
        0
    };

    Ok((max_atr, minutes_to_peak))
}

/// Classifie le niveau de volatilité baseline
pub fn classifier_volatilite(candles: &[Candle], atr: f64) -> VolatilityLevel {
    let sample_candle = candles.first().expect("Candles should not be empty");
    let price = sample_candle.close;
    let atr_pct = (atr / price) * 100.0;

    if atr_pct < 0.01 {
        VolatilityLevel::Low
    } else if atr_pct < 0.03 {
        VolatilityLevel::Medium
    } else {
        VolatilityLevel::High
    }
}

/// Recommande les multiplicateurs SL/TP basés sur le contexte
pub fn recommander_multiplicateurs(atr_ratio: f64, volatility_level: VolatilityLevel) -> (f64, f64) {
    let base_sl: f64 = match volatility_level {
        VolatilityLevel::Low => 1.5,
        VolatilityLevel::Medium => 2.0,
        VolatilityLevel::High => 2.5,
    };

    let base_tp: f64 = match volatility_level {
        VolatilityLevel::Low => 2.5,
        VolatilityLevel::Medium => 3.0,
        VolatilityLevel::High => 3.5,
    };

    let sl_multiplier: f64 = if atr_ratio > 2.0 {
        base_sl + 0.5
    } else if atr_ratio > 1.5 {
        base_sl
    } else {
        base_sl - 0.3
    };

    let tp_multiplier: f64 = if atr_ratio > 2.0 {
        base_tp + 1.0
    } else if atr_ratio > 1.5 {
        base_tp
    } else {
        base_tp - 0.5
    };

    (sl_multiplier.max(1.0), tp_multiplier.max(1.5))
}
