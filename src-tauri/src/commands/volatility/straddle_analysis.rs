// commands/volatility/straddle_analysis.rs - Commands pour calculs Straddle
use crate::models::{Candle, AssetProperties};
use crate::services::straddle_simulator_helpers::calculer_atr_moyen;
use crate::services::volatility::calculer_frequence_whipsaw as service_calculer_frequence_whipsaw;
use crate::services::straddle_simulator::simulate_straddle;
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
pub fn calculer_offset_optimal(
    candles: Vec<Candle>,
    _window: tauri::Window,
) -> Result<OptimalOffsetResponse, String> {
    tracing::info!(
        "Command: calculer_offset_optimal for {} candles",
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
    let asset_props = AssetProperties::from_symbol(symbol);
    let pip_value = asset_props.pip_value;

    // 2. Calculer les métriques nécessaires
    let raw_atr_mean = calculer_atr_moyen(&candles);
    let atr_mean = asset_props.normalize(raw_atr_mean);

    // Calcul du Noise Ratio moyen
    let noise_ratio_mean: f64 = if !candles.is_empty() {
        let sum: f64 = candles
            .iter()
            .map(|c| {
                let range = c.high - c.low;
                let body = (c.open - c.close).abs();
                if body < pip_value * 0.1 {
                    if range < pip_value * 0.1 {
                        1.0
                    } else {
                        5.0
                    }
                } else {
                    range / body
                }
            })
            .sum();
        sum / candles.len() as f64
    } else {
        1.0
    };

    // 3. Utiliser le service unifié pour calculer l'offset
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
    let raw_percentile_95 = sorted_wicks.get(p95_index).copied().unwrap_or(0.0);

    // Conversion correcte en pips via AssetProperties
    let p95_pips = asset_props.normalize(raw_percentile_95);

    let params = StraddleParameterService::calculate_parameters(
        atr_mean,
        noise_ratio_mean,
        pip_value,
        symbol,
        None,
        Some(p95_pips),
    );

    let offset_pips = params.offset_pips;

    Ok(OptimalOffsetResponse {
        offset_pips,
        percentile_95_wicks: p95_pips,
        with_margin: p95_pips * 1.1,
    })
}

/// Simule le win rate pour un ensemble de candles
#[command]
pub fn calculer_taux_reussite(
    candles: Vec<Candle>,
    offset_pips: f64,
    _window: tauri::Window,
) -> Result<WinRateResponse, String> {
    tracing::info!(
        "Command: calculer_taux_reussite for {} candles with offset {}",
        candles.len(),
        offset_pips
    );

    let symbol = candles.first().map(|c| c.symbol.as_str()).unwrap_or("EURUSD");
    let result = simulate_straddle(&candles, symbol, Some(offset_pips));

    Ok(WinRateResponse {
        total_trades: result.total_trades,
        wins: result.wins,
        losses: result.losses,
        whipsaws: result.whipsaws,
        win_rate_percentage: result.win_rate_percentage,
        offset_pips: result.offset_optimal_pips,
    })
}

/// Calcule la fréquence des whipsaws
#[command]
pub fn calculer_frequence_whipsaw(
    candles: Vec<Candle>,
    offset_pips: f64,
    _window: tauri::Window,
) -> Result<WhipsawResponse, String> {
    tracing::info!(
        "Command: calculer_frequence_whipsaw for {} candles with offset {}",
        candles.len(),
        offset_pips
    );

    let symbol = candles.first().map(|c| c.symbol.as_str()).unwrap_or("EURUSD");
    let analysis = service_calculer_frequence_whipsaw(&candles, offset_pips, symbol);

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
