// services/metrics/distribution.rs - Distribution du True Range
// Conforme .clinerules : < 100L, pas d'unwrap()

use crate::models::{Candle, Result, VolatilityError};
use std::cmp::Ordering;
use tracing::debug;

/// Distribution du True Range avec détection de breakout
#[derive(Debug, Clone)]
pub struct TrueRangeDistribution {
    /// Tous les True Ranges calculés
    /// NOTE: Ce field est public pour introspection (utilisé par clients externes)
    #[allow(dead_code)]
    pub true_ranges: Vec<f64>,
    /// 80e percentile du TR (seuil de breakout)
    /// NOTE: Ce field est public pour introspection (utilisé par clients externes)
    #[allow(dead_code)]
    pub percentile_80: f64,
    /// Indicateur de breakout pour chaque bougie
    pub is_breakout: Vec<bool>,
}

impl TrueRangeDistribution {
    /// Calcule la distribution du True Range
    pub fn calculer(candles: &[Candle]) -> Result<Self> {
        if candles.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "No candles for True Range distribution".to_string(),
            ));
        }

        debug!(
            "Calculating True Range distribution for {} candles",
            candles.len()
        );

        // Calcule tous les True Ranges
        let mut true_ranges = Vec::new();

        for i in 0..candles.len() {
            let prev_close = if i > 0 {
                Some(candles[i - 1].close)
            } else {
                None
            };
            true_ranges.push(candles[i].true_range(prev_close));
        }

        // Calcule le 80e percentile (FIX: plus d'unwrap())
        let mut sorted_ranges = true_ranges.clone();
        sorted_ranges.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let percentile_index = (sorted_ranges.len() as f64 * 0.80) as usize;
        let percentile_80 = sorted_ranges
            .get(percentile_index.min(sorted_ranges.len() - 1))
            .copied()
            .unwrap_or(0.0);

        // Calcule la médiane pour un seuil de breakout plus dynamique
        let median_index = sorted_ranges.len() / 2;
        let median = if sorted_ranges.len() % 2 == 0 {
            (sorted_ranges[median_index - 1] + sorted_ranges[median_index]) / 2.0
        } else {
            sorted_ranges[median_index]
        };

        // Seuil de breakout : 2x la médiane (plus variable que percentile fixe)
        let breakout_threshold = median * 2.0;

        debug!(
            "80th percentile: {:.5}, Median: {:.5}, Breakout threshold (2x median): {:.5}",
            percentile_80, median, breakout_threshold
        );

        // Détecte les breakouts (TR > 2x médiane)
        let is_breakout: Vec<bool> = true_ranges
            .iter()
            .map(|&tr| tr > breakout_threshold)
            .collect();

        let breakout_count = is_breakout.iter().filter(|&&b| b).count();
        debug!(
            "Detected {} breakout candles ({:.1}%)",
            breakout_count,
            (breakout_count as f64 / true_ranges.len() as f64) * 100.0
        );

        Ok(Self {
            true_ranges,
            percentile_80,
            is_breakout,
        })
    }
}
