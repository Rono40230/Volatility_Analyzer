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
    /// 95e percentile du TR (Max Spike stabilisé)
    pub percentile_95: f64,
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
        // IMPORTANT: vérifier la contiguïté temporelle pour éviter les faux TR inter-jours
        // Quand les bougies sont groupées par heure (08:xx de jours différents),
        // prev_close serait celui d'un autre jour → TR artificiellement gonflé
        let mut true_ranges = Vec::new();

        for i in 0..candles.len() {
            let prev_close = if i > 0 {
                let diff = candles[i]
                    .datetime
                    .signed_duration_since(candles[i - 1].datetime);
                // Contiguïté: la bougie précédente doit être à 1-2 minutes max
                if diff.num_minutes().abs() <= 2 {
                    Some(candles[i - 1].close)
                } else {
                    None // Gap temporel → pas de prev_close (TR = high - low)
                }
            } else {
                None
            };
            true_ranges.push(candles[i].true_range(prev_close));
        }

        // Calcule les percentiles (FIX: plus d'unwrap())
        let mut sorted_ranges = true_ranges.clone();
        sorted_ranges.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let len = sorted_ranges.len();
        let p80_index = ((len - 1) as f64 * 0.80) as usize;
        let percentile_80 = sorted_ranges[p80_index];

        let p95_index = ((len - 1) as f64 * 0.95) as usize;
        let percentile_95 = sorted_ranges[p95_index];

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
            "80th percentile: {:.5}, 95th percentile: {:.5}, Median: {:.5}, Breakout threshold (2x median): {:.5}",
            percentile_80, percentile_95, median, breakout_threshold
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
            percentile_95,
            is_breakout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Candle;
    use chrono::Utc;

    #[test]
    fn test_percentile_95() {
        let mut candles = Vec::new();
        // 100 candles avec True Range = 1.0
        for _ in 0..100 {
            candles.push(Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: Utc::now(),
                open: 1.0,
                high: 2.0,
                low: 1.0,
                close: 1.5,
                volume: 100.0,
                ..Default::default()
            });
        }
        // Ajouter quelques outliers (5 candles avec TR = 10.0)
        // Le 95e percentile devrait être la valeur à l'index 94 (0-indexed) après tri.
        // Si on a 105 candles, 95% de 105 = 99.75.
        // Restons sur 100 candles pour simplifier.
        for candle in candles.iter_mut().take(5) {
            candle.high = 11.0;
        }

        let dist = TrueRangeDistribution::calculer(&candles).expect("Calcul distribution échoué");

        // Le 95e percentile devrait ignorer les 5 outliers les plus hauts
        // et retourner 1.0 (la valeur du 95e élément après tri)
        assert!(dist.percentile_95 < 2.0);
        assert!(dist.percentile_95 >= 1.0);
    }
}
