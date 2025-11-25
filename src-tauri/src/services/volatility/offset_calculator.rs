// services/volatility/offset_calculator.rs - Calcul de l'offset optimal pour Straddle
use crate::models::Candle;
use tracing::{debug, info};

/// Calcule l'offset optimal pour un Straddle en évitant 95% des fausses mèches
/// 
/// Algorithme :
/// 1. Extraire toutes les wicks (upper et lower) des candles
/// 2. Calculer le percentile 95 de toutes les wicks
/// 3. Ajouter 10% de marge de sécurité
/// 4. Retourner en points (pips × 10000)
pub fn calculate_optimal_offset(candles: &[Candle]) -> f64 {
    if candles.is_empty() {
        return 10.0; // Fallback : 10 pips minimum
    }

    // 1. Extraire toutes les wicks
    let mut wicks: Vec<f64> = Vec::new();

    for (idx, candle) in candles.iter().enumerate() {
        // Upper wick : différence entre High et max(Open, Close)
        let upper_wick = candle.high - candle.close.max(candle.open);
        if upper_wick > 0.0 {
            wicks.push(upper_wick);
        }

        // Lower wick : différence entre min(Open, Close) et Low
        let lower_wick = candle.open.min(candle.close) - candle.low;
        if lower_wick > 0.0 {
            wicks.push(lower_wick);
        }
        
        // Log des 5 premières candles pour debug
        if idx < 3 {
            debug!("  Candle {}: O={} H={} L={} C={}", idx, candle.open, candle.high, candle.low, candle.close);
            debug!("    Upper wick: {} Lower wick: {}", upper_wick, lower_wick);
        }
    }

    if wicks.is_empty() {
        return 10.0; // Fallback
    }

    info!("📊 Offset calc: {} candles → {} wicks total", candles.len(), wicks.len());

    // 2. Trier les wicks pour calculer le percentile 95
    wicks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p95_index = ((wicks.len() as f64) * 0.95).ceil() as usize;
    let p95_index = p95_index.min(wicks.len() - 1);

    let wick_95 = wicks[p95_index];
    let min_wick = wicks[0];
    let max_wick = wicks[wicks.len() - 1];
    let mean_wick = wicks.iter().sum::<f64>() / wicks.len() as f64;

    info!("  Min wick: {:.8} | Mean: {:.8} | P95: {:.8} | Max: {:.8}", min_wick, mean_wick, wick_95, max_wick);

    // 3. Ajouter 10% de marge de sécurité
    let optimal_offset = wick_95 * 1.1;

    info!("  Offset (avant ×10000): {:.8}", optimal_offset);

    // 4. Convertir en pips (valeur est déjà normalisée, on la multiplie par 10000)
    let result = optimal_offset * 10000.0;
    
    info!("  ✅ Offset final: {:.0} pips", result);
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_calculation() {
        // Créer 10 candles avec wicks connus
        let candles = vec![
            Candle {
                open: 1.0850,
                high: 1.0860,
                low: 1.0840,
                close: 1.0855,
                volume: 1000.0,
                time: 0,
                hour_utc: 14,
            }, // Upper wick: 0.0005, Lower wick: 0.0010
            Candle {
                open: 1.0855,
                high: 1.0870,
                low: 1.0845,
                close: 1.0860,
                volume: 1000.0,
                time: 60000,
                hour_utc: 14,
            }, // Upper wick: 0.0010, Lower wick: 0.0015
        ];

        let offset = calculate_optimal_offset(&candles);
        assert!(offset > 0.0);
        // P95 des wicks [0.0005, 0.0010, 0.0010, 0.0015] = 0.0015
        // Avec 10% marge = 0.00165
        // En pips = 0.00165 * 10000 = 16.5
        assert!(offset > 15.0 && offset < 20.0);
    }

    #[test]
    fn test_empty_candles() {
        let candles = vec![];
        let offset = calculate_optimal_offset(&candles);
        assert_eq!(offset, 10.0); // Fallback
    }
}
