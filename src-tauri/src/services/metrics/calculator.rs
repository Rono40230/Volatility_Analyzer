// services/metrics/calculator.rs - Calculateur principal de métriques
// Conforme .clinerules : < 200L, pas d'unwrap()

use super::distribution::TrueRangeDistribution;
use crate::models::{Candle, Result, VolatilityError};
use tracing::{debug, info};

/// Calculateur de métriques de volatilité
pub struct MetricsCalculator<'a> {
    candles: &'a [Candle],
}

impl<'a> MetricsCalculator<'a> {
    /// Crée un nouveau calculateur avec les bougies fournies
    pub fn new(candles: &'a [Candle]) -> Self {
        Self { candles }
    }

    /// Calcule l'ATR (Average True Range) sur une période donnée
    pub fn calculer_atr(&self, period: usize) -> Result<Vec<f64>> {
        if self.candles.len() < period {
            return Err(VolatilityError::InsufficientData(format!(
                "Need at least {} candles for ATR calculation, got {}",
                period,
                self.candles.len()
            )));
        }

        info!("Calculating ATR with period {}", period);

        let mut true_ranges = Vec::new();

        // Calcule le True Range pour chaque bougie
        for i in 0..self.candles.len() {
            let is_contiguous = if i > 0 {
                let diff = self
                    .candles[i]
                    .datetime
                    .signed_duration_since(self.candles[i - 1].datetime);
                diff.num_minutes() == 1
            } else {
                false
            };

            let prev_close = if is_contiguous {
                Some(self.candles[i - 1].close)
            } else {
                None
            };
            true_ranges.push(self.candles[i].true_range(prev_close));
        }

        // Calcule l'ATR avec Wilder's smoothing (EMA)
        let mut atr_values = Vec::new();

        // Première valeur: moyenne simple des premiers TR
        let first_atr = true_ranges[0..period].iter().sum::<f64>() / period as f64;
        atr_values.push(first_atr);

        // Smoothing Wilder: ATR[i] = (ATR[i-1] * (period - 1) + TR[i]) / period
        let multiplier = (period - 1) as f64;
        for tr in true_ranges.iter().skip(period) {
            let prev_atr = atr_values[atr_values.len() - 1]; // Dernière valeur ATR calculée
            let atr = (prev_atr * multiplier + tr) / period as f64;
            atr_values.push(atr);
        }

        debug!(
            "ATR calculated with Wilder's smoothing: {} values",
            atr_values.len()
        );
        Ok(atr_values)
    }

    /// Calcule la volatilité en pourcentage (écart-type des rendements)
    pub fn calculer_volatilite(&self, period: usize) -> Result<Vec<f64>> {
        if self.candles.len() < period + 1 {
            return Err(VolatilityError::InsufficientData(format!(
                "Need at least {} candles for volatility calculation",
                period + 1
            )));
        }

        info!("Calculating volatility with period {}", period);

        // Calcule les rendements (returns)
        let mut returns = Vec::new();
        for i in 1..self.candles.len() {
            let is_contiguous = self
                .candles[i]
                .datetime
                .signed_duration_since(self.candles[i - 1].datetime)
                .num_minutes()
                == 1;

            if is_contiguous {
                let ret = (self.candles[i].close - self.candles[i - 1].close)
                    / self.candles[i - 1].close;
                returns.push(ret);
            }
        }

        if returns.len() < period {
            return Ok(Vec::new());
        }

        // Calcule l'écart-type glissant
        let mut volatilities = Vec::new();

        for i in (period - 1)..returns.len() {
            let start_idx = i + 1 - period;
            let slice = &returns[start_idx..=i];
            let mean = slice.iter().sum::<f64>() / period as f64;
            let variance = slice.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / period as f64;
            let std_dev = variance.sqrt();
            volatilities.push(std_dev * 100.0); // En pourcentage
        }

        debug!("Volatility calculated: {} values", volatilities.len());
        Ok(volatilities)
    }

    /// Calcule le Body Range pour toutes les bougies
    pub fn calculer_ranges_corps(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.body_range()).collect()
    }

    /// Calcule le Shadow Ratio pour toutes les bougies
    pub fn calculer_ratios_ombres(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.shadow_ratio()).collect()
    }

    /// Calcule le Tick Quality (variation prix / volume)
    pub fn calculer_qualite_tick(&self) -> Vec<f64> {
        self.candles
            .iter()
            .map(|c| {
                let price_change = (c.close - c.open).abs();
                let volume = c.volume.max(0.1); // Évite division par zéro
                price_change / volume
            })
            .collect()
    }

    /// Calcule le Noise Ratio (True Range / mouvement net)
    pub fn calculer_ratio_bruit(&self) -> Vec<f64> {
        let mut noise_ratios = Vec::new();

        for i in 0..self.candles.len() {
            let prev_close = if i > 0 {
                Some(self.candles[i - 1].close)
            } else {
                None
            };

            let true_range = self.candles[i].true_range(prev_close);
            let close_change = if i > 0 {
                (self.candles[i].close - self.candles[i - 1].close).abs()
            } else {
                (self.candles[i].close - self.candles[i].open).abs()
            };

            let noise = if close_change > 0.0001 {
                true_range / close_change
            } else {
                1.0 // Neutre si pas de mouvement
            };

            noise_ratios.push(noise);
        }

        noise_ratios
    }

    /// Calcule la distribution du True Range (détecte les breakouts)
    pub fn calculer_distribution_true_range(&self) -> Result<TrueRangeDistribution> {
        TrueRangeDistribution::calculer(self.candles)
    }

    /// Retourne le nombre de bougies
    #[allow(dead_code)]
    pub fn candle_count(&self) -> usize {
        self.candles.len()
    }
}
