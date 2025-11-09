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
    pub fn calculate(candles: &[Candle]) -> Result<Self> {
        if candles.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "No candles for True Range distribution".to_string(),
            ));
        }

        debug!("Calculating True Range distribution for {} candles", candles.len());

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

        debug!("80th percentile True Range: {:.5}", percentile_80);

        // Détecte les breakouts (TR > 80e percentile)
        let is_breakout: Vec<bool> = true_ranges
            .iter()
            .map(|&tr| tr > percentile_80)
            .collect();

        let breakout_count = is_breakout.iter().filter(|&&b| b).count();
        debug!("Detected {} breakout candles", breakout_count);

        Ok(Self {
            true_ranges,
            percentile_80,
            is_breakout,
        })
    }
}
