// services/slice_metrics_analyzer.rs
// Analyseur de métriques pour un créneau de 15 minutes spécifique
// Calcule les vraies statistiques historiques sur la période élue

use crate::models::Candle;
use crate::services::candle_index::CandleIndex;

#[derive(Debug, Clone)]
pub struct SliceMetrics {
    pub candle_count: usize,
    pub atr_mean: f64,
    pub atr_max: f64,
    pub volatility_mean: f64,
    pub range_mean: f64,
    pub body_range_mean: f64,
    pub noise_ratio_mean: f64,
    pub breakout_percentage: f64,
    pub volume_imbalance_mean: f64,
    pub shadow_ratio_mean: f64,
}

impl Default for SliceMetrics {
    fn default() -> Self {
        Self {
            candle_count: 0,
            atr_mean: 0.0,
            atr_max: 0.0,
            volatility_mean: 0.0,
            range_mean: 0.0,
            body_range_mean: 0.0,
            noise_ratio_mean: 0.0,
            breakout_percentage: 0.0,
            volume_imbalance_mean: 0.0,
            shadow_ratio_mean: 0.0,
        }
    }
}

/// Analyse les métriques pour un créneau de 15 minutes spécifique
/// en utilisant TOUTES les occurrences historiques de ce créneau
pub fn analyze_slice_metrics(
    candle_index: &CandleIndex,
    symbol: &str,
    hour: u32,
    quarter: u32,
) -> Result<(SliceMetrics, Vec<Candle>), String> {
    // Vérifier que la paire est chargée
    if !candle_index.is_pair_loaded(symbol) {
        return Err(format!("Symbol {} not loaded in candle index", symbol));
    }

    // Calculer les minutes de début et fin du créneau
    let start_minute = quarter * 15;
    let end_minute = start_minute + 15;

    // Collecter toutes les bougies du créneau à travers l'historique (OPTIMISÉ)
    let slice_candles =
        candle_index.get_candles_for_slice_all_history(symbol, hour, start_minute, end_minute);

    if slice_candles.is_empty() {
        return Ok((SliceMetrics::default(), Vec::new()));
    }

    // Calculer les métriques réelles
    let metrics = calculate_metrics_from_candles(&slice_candles, symbol)?;

    Ok((metrics, slice_candles))
}

/// Calcule les métriques à partir d'un ensemble de bougies
fn calculate_metrics_from_candles(
    candles: &[Candle],
    symbol: &str,
) -> Result<SliceMetrics, String> {
    use crate::services::straddle_simulator::normalize_to_pips;

    if candles.is_empty() {
        return Ok(SliceMetrics::default());
    }

    let count = candles.len();
    let mut atr_sum = 0.0;
    let mut atr_max = 0.0;
    let mut range_sum = 0.0;
    let mut body_range_sum = 0.0;
    let mut noise_ratio_sum = 0.0;
    let mut volume_imbalance_sum = 0.0;
    let mut shadow_ratio_sum = 0.0;

    for candle in candles {
        // ATR (True Range)
        let tr = candle.high - candle.low;
        atr_sum += tr;
        if tr > atr_max {
            atr_max = tr;
        }

        // Range
        let range = candle.high - candle.low;
        range_sum += range;

        // Body Range %
        let body = (candle.close - candle.open).abs();
        let body_pct = if range > 0.0 {
            (body / range) * 100.0
        } else {
            0.0
        };
        body_range_sum += body_pct;

        // Noise Ratio (wicks / body)
        let upper_wick = candle.high - candle.close.max(candle.open);
        let lower_wick = candle.open.min(candle.close) - candle.low;
        let total_wicks = upper_wick + lower_wick;
        let noise = if body > 0.0 { total_wicks / body } else { 0.0 };
        noise_ratio_sum += noise;

        // Volume Imbalance (Direction Strength)
        // Simplifié: ratio du body par rapport au range
        volume_imbalance_sum += body_pct / 100.0;

        // Shadow Ratio
        let shadow_ratio = if range > 0.0 {
            total_wicks / range
        } else {
            0.0
        };
        shadow_ratio_sum += shadow_ratio;
    }

    let atr_mean = atr_sum / count as f64;
    let range_mean = range_sum / count as f64;
    let body_range_mean = body_range_sum / count as f64;
    let noise_ratio_mean = noise_ratio_sum / count as f64;
    let volume_imbalance_mean = volume_imbalance_sum / count as f64;
    let shadow_ratio_mean = shadow_ratio_sum / count as f64;

    // Volatilité % (ATR / Close moyen)
    let avg_close: f64 = candles.iter().map(|c| c.close).sum::<f64>() / count as f64;
    let volatility_mean = if avg_close > 0.0 {
        (atr_mean / avg_close) * 100.0
    } else {
        0.0
    };

    // Breakout % (using real detection)
    use crate::services::breakout_detector::calculate_breakout_percentage;
    let breakout_percentage = calculate_breakout_percentage(candles);

    // Normaliser ATR et Range en pips
    let atr_mean_pips = normalize_to_pips(atr_mean, symbol).ceil();
    let atr_max_pips = normalize_to_pips(atr_max, symbol).ceil();
    let range_mean_pips = normalize_to_pips(range_mean, symbol).ceil();

    Ok(SliceMetrics {
        candle_count: count,
        atr_mean: atr_mean_pips,
        atr_max: atr_max_pips,
        volatility_mean,
        range_mean: range_mean_pips,
        body_range_mean,
        noise_ratio_mean,
        breakout_percentage,
        volume_imbalance_mean,
        shadow_ratio_mean,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_metrics_default() {
        let metrics = SliceMetrics::default();
        assert_eq!(metrics.candle_count, 0);
        assert_eq!(metrics.atr_mean, 0.0);
    }
}
