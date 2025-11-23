// services/volatility/offset_calculator.rs - Calcul de l'offset optimal pour Straddle
use crate::models::Candle;

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

    for candle in candles {
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
    }

    if wicks.is_empty() {
        return 10.0; // Fallback
    }

    // 2. Trier les wicks pour calculer le percentile 95
    wicks.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p95_index = ((wicks.len() as f64) * 0.95).ceil() as usize;
    let p95_index = p95_index.min(wicks.len() - 1);

    let wick_95 = wicks[p95_index];

    // 3. Ajouter 10% de marge de sécurité
    let optimal_offset = wick_95 * 1.1;

    // 4. Convertir en pips (valeur est déjà normalisée, on la multiplie par 10000)
    optimal_offset * 10000.0
}

/// Calcule l'offset optimal avec des statistiques détaillées pour le debug
pub fn calculate_optimal_offset_with_stats(candles: &[Candle]) -> (f64, OffsetStats) {
    let stats = OffsetStats::from_candles(candles);

    let offset = if !stats.wicks.is_empty() {
        let wick_95 = stats.percentile_95;
        wick_95 * 1.1 * 10000.0
    } else {
        10.0
    };

    (offset, stats)
}

/// Statistiques détaillées pour l'offset
#[derive(Debug, Clone)]
pub struct OffsetStats {
    pub candle_count: usize,
    pub wicks: Vec<f64>,
    pub min_wick: f64,
    pub max_wick: f64,
    pub mean_wick: f64,
    pub percentile_95: f64,
    pub recommended_offset_pips: f64,
}

impl OffsetStats {
    pub fn from_candles(candles: &[Candle]) -> Self {
        let mut wicks = Vec::new();

        for candle in candles {
            let upper_wick = candle.high - candle.close.max(candle.open);
            if upper_wick > 0.0 {
                wicks.push(upper_wick);
            }

            let lower_wick = candle.open.min(candle.close) - candle.low;
            if lower_wick > 0.0 {
                wicks.push(lower_wick);
            }
        }

        if wicks.is_empty() {
            return Self {
                candle_count: candles.len(),
                wicks: Vec::new(),
                min_wick: 0.0,
                max_wick: 0.0,
                mean_wick: 0.0,
                percentile_95: 0.0,
                recommended_offset_pips: 10.0,
            };
        }

        let min_wick = wicks.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_wick = wicks.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean_wick = wicks.iter().sum::<f64>() / wicks.len() as f64;

        // Percentile 95
        let mut sorted_wicks = wicks.clone();
        sorted_wicks.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_index = ((sorted_wicks.len() as f64) * 0.95).ceil() as usize;
        let p95_index = p95_index.min(sorted_wicks.len() - 1);
        let percentile_95 = sorted_wicks[p95_index];

        let recommended_offset_pips = percentile_95 * 1.1 * 10000.0;

        Self {
            candle_count: candles.len(),
            wicks,
            min_wick,
            max_wick,
            mean_wick,
            percentile_95,
            recommended_offset_pips,
        }
    }
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

    #[test]
    fn test_offset_stats() {
        let candles = vec![
            Candle {
                open: 1.0850,
                high: 1.0860,
                low: 1.0840,
                close: 1.0855,
                volume: 1000.0,
                time: 0,
                hour_utc: 14,
            },
        ];

        let (offset, stats) = calculate_optimal_offset_with_stats(&candles);
        assert!(offset > 0.0);
        assert_eq!(stats.candle_count, 1);
        assert!(!stats.wicks.is_empty());
    }
}
