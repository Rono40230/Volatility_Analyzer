// commands/volatility/straddle_analysis.rs - Commands pour calculs Straddle
use crate::models::Candle;
use crate::services::pair_data::get_point_value;
use crate::services::straddle_simulator_helpers::calculate_atr_mean;
use crate::services::volatility::{
    calculate_whipsaw_frequency, simulate_straddle_win_rate,
};
use crate::services::StraddleParameterService;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimalOffsetResponse {
    pub offset_pips: f64,
    pub percentile_95_wicks: f64,
    pub with_margin: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WinRateResponse {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate_percentage: f64,
    pub offset_pips: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhipsawResponse {
    pub total_trades: usize,
    pub whipsaw_count: usize,
    pub whipsaw_frequency_percentage: f64,
    pub risk_level: String,
    pub risk_color: String,
    pub offset_pips: f64,
}

/// Calcule l'offset optimal pour éviter 95% des fausses mèches
#[command]
pub fn calculate_offset_optimal(
    candles: Vec<Candle>,
    _window: tauri::Window,
) -> Result<OptimalOffsetResponse, String> {
    tracing::info!(
        "Command: calculate_offset_optimal for {} candles",
        candles.len()
    );

    if candles.is_empty() {
        return Ok(OptimalOffsetResponse {
            offset_pips: 0.0,
            percentile_95_wicks: 0.0,
            with_margin: 0.0,
        });
    }

    // 1. Récupérer les infos du symbole
    let symbol = &candles[0].symbol;
    let point_value = get_point_value(symbol);

    // 2. Calculer les métriques nécessaires
    let atr_mean = calculate_atr_mean(&candles);
    
    // Calcul du Noise Ratio moyen
    let noise_ratio_mean: f64 = if !candles.is_empty() {
        let sum: f64 = candles.iter().map(|c| {
            let range = c.high - c.low;
            let body = (c.open - c.close).abs();
            if body < point_value * 0.1 {
                if range < point_value * 0.1 { 1.0 } else { 5.0 }
            } else {
                range / body
            }
        }).sum();
        sum / candles.len() as f64
    } else {
        1.0
    };

    // 3. Utiliser le service unifié pour calculer l'offset
    let params = StraddleParameterService::calculate_parameters(
        atr_mean,
        noise_ratio_mean,
        point_value
    );
    
    let offset_pips = params.offset_pips;

    // Calculer aussi les stats détaillées (Percentile 95 des mèches)
    let wicks: Vec<f64> = candles
        .iter()
        .flat_map(|c| {
            let upper = c.high - c.close.max(c.open);
            let lower = c.open.min(c.close) - c.low;
            vec![
                if upper > 0.0 { upper } else { 0.0 },
                if lower > 0.0 { lower } else { 0.0 },
            ]
        })
        .filter(|w| *w > 0.0)
        .collect();

    let mut sorted_wicks = wicks.clone();
    sorted_wicks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95_index = ((sorted_wicks.len() as f64) * 0.95).ceil() as usize;
    let p95_index = p95_index.min(sorted_wicks.len().saturating_sub(1));
    let percentile_95 = sorted_wicks.get(p95_index).copied().unwrap_or(0.0);

    // Conversion correcte en pips via point_value
    let p95_pips = percentile_95 / point_value;

    Ok(OptimalOffsetResponse {
        offset_pips,
        percentile_95_wicks: p95_pips,
        with_margin: p95_pips * 1.1,
    })
}

/// Simule le win rate pour un ensemble de candles
#[command]
pub fn calculate_win_rate(
    candles: Vec<Candle>,
    offset_pips: f64,
    _window: tauri::Window,
) -> Result<WinRateResponse, String> {
    tracing::info!(
        "Command: calculate_win_rate for {} candles with offset {}",
        candles.len(),
        offset_pips
    );

    let result = simulate_straddle_win_rate(&candles, offset_pips);

    Ok(WinRateResponse {
        total_trades: result.total_trades,
        wins: result.wins,
        losses: result.losses,
        whipsaws: result.whipsaws,
        win_rate_percentage: result.win_rate * 100.0,
        offset_pips: result.offset_pips,
    })
}

/// Calcule la fréquence des whipsaws
#[command]
pub fn calculate_whipsaw_freq(
    candles: Vec<Candle>,
    offset_pips: f64,
    _window: tauri::Window,
) -> Result<WhipsawResponse, String> {
    tracing::info!(
        "Command: calculate_whipsaw_freq for {} candles with offset {}",
        candles.len(),
        offset_pips
    );

    let analysis = calculate_whipsaw_frequency(&candles, offset_pips);

    Ok(WhipsawResponse {
        total_trades: analysis.total_trades,
        whipsaw_count: analysis.whipsaw_count,
        whipsaw_frequency_percentage: analysis.whipsaw_frequency * 100.0,
        risk_level: analysis.risk_level.as_str().to_string(),
        risk_color: analysis.risk_level.color().to_string(),
        offset_pips: analysis.offset_pips,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_response_serialization() {
        let resp = OptimalOffsetResponse {
            offset_pips: 12.5,
            percentile_95_wicks: 10.0,
            with_margin: 11.0,
        };
        let json = serde_json::to_string(&resp).expect("serialization failed");
        assert!(json.contains("offset_pips"));
    }
}
