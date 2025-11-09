// services/contextual_atr_analyzer.rs - Analyse ATR contextuel avant/après événement
// Conforme .clinerules : < 300L, pas d'unwrap()

use crate::models::{Candle, Result, VolatilityError};
use crate::services::metrics::MetricsCalculator;
use chrono::{DateTime, Duration, Utc};
use tracing::{debug, info};

/// Analyseur d'ATR contextuel pour événements économiques
pub struct ContextualAtrAnalyzer<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
}

/// Résultat de l'analyse ATR contextuelle
#[derive(Debug, Clone)]
pub struct ContextualAtrMetrics {
    /// ATR moyen 30min avant événement (baseline)
    pub atr_before_event: f64,
    /// ATR moyen 30min après événement (pic)
    pub atr_after_event: f64,
    /// Ratio ATR après/avant (multiplicateur d'impact)
    pub atr_ratio: f64,
    /// ATR maximum atteint dans les 2h post-événement
    pub max_atr_spike: f64,
    /// Temps en minutes pour atteindre le pic d'ATR
    /// NOTE: Ce field est public pour introspection mais non utilisé actuellement
    #[allow(dead_code)]
    pub minutes_to_peak: i64,
    /// Multiplicateur SL recommandé basé sur l'historique
    pub recommended_sl_multiplier: f64,
    /// Multiplicateur TP recommandé basé sur l'historique
    pub recommended_tp_multiplier: f64,
    /// Volatilité baseline (faible/moyenne/haute)
    pub baseline_volatility_level: VolatilityLevel,
}

/// Niveau de volatilité
#[derive(Debug, Clone, PartialEq)]
pub enum VolatilityLevel {
    Low,
    Medium,
    High,
}

impl<'a> ContextualAtrAnalyzer<'a> {
    /// Crée un nouvel analyseur
    pub fn new(candles: &'a [Candle], event_time: DateTime<Utc>) -> Self {
        Self {
            candles,
            event_time,
        }
    }

    /// Analyse complète de l'ATR contextuel
    pub fn analyze(&self, atr_period: usize) -> Result<ContextualAtrMetrics> {
        info!("Analyzing contextual ATR for event at {}", self.event_time);

        // Calculer ATR avant l'événement (baseline)
        let atr_before = self.calculate_baseline_atr(atr_period)?;
        debug!("Baseline ATR (30min before): {:.5}", atr_before);

        // Calculer ATR après l'événement (pic)
        let atr_after = self.calculate_post_event_atr(atr_period)?;
        debug!("Post-event ATR (30min after): {:.5}", atr_after);

        // Ratio d'impact
        let atr_ratio = if atr_before > 0.0 {
            atr_after / atr_before
        } else {
            1.0
        };

        // Trouver le pic maximal dans les 2h
        let (max_spike, minutes_to_peak) = self.find_max_atr_spike(atr_period)?;
        debug!(
            "Max ATR spike: {:.5} at +{} minutes",
            max_spike, minutes_to_peak
        );

        // Déterminer le niveau de volatilité baseline
        let volatility_level = self.classify_volatility(atr_before);

        // Recommandations de multiplicateurs
        let (sl_mult, tp_mult) = self.recommend_multipliers(atr_ratio, volatility_level.clone());

        info!(
            "ATR ratio: {:.2}x, Recommended SL: {:.1}x, TP: {:.1}x",
            atr_ratio, sl_mult, tp_mult
        );

        Ok(ContextualAtrMetrics {
            atr_before_event: atr_before,
            atr_after_event: atr_after,
            atr_ratio,
            max_atr_spike: max_spike,
            minutes_to_peak,
            recommended_sl_multiplier: sl_mult,
            recommended_tp_multiplier: tp_mult,
            baseline_volatility_level: volatility_level,
        })
    }

    /// Calcule l'ATR baseline (30min avant événement)
    fn calculate_baseline_atr(&self, atr_period: usize) -> Result<f64> {
        let start_time = self.event_time - Duration::minutes(30);
        let end_time = self.event_time;

        let baseline_candles: Vec<Candle> = self
            .candles
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
        let atr_values = calculator.calculate_atr(atr_period)?;
        
        atr_values
            .last()
            .copied()
            .ok_or_else(|| VolatilityError::InsufficientData("No ATR values".to_string()))
    }

    /// Calcule l'ATR post-événement (30min après)
    fn calculate_post_event_atr(&self, atr_period: usize) -> Result<f64> {
        let start_time = self.event_time;
        let end_time = self.event_time + Duration::minutes(30);

        let post_candles: Vec<Candle> = self
            .candles
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
        let atr_values = calculator.calculate_atr(atr_period)?;
        
        atr_values
            .last()
            .copied()
            .ok_or_else(|| VolatilityError::InsufficientData("No ATR values".to_string()))
    }

    /// Trouve le pic maximal d'ATR dans les 2h post-événement
    fn find_max_atr_spike(&self, atr_period: usize) -> Result<(f64, i64)> {
        let end_time = self.event_time + Duration::minutes(120);

        let post_candles: Vec<Candle> = self
            .candles
            .iter()
            .filter(|c| c.datetime >= self.event_time && c.datetime < end_time)
            .cloned()
            .collect();

        if post_candles.len() < atr_period + 10 {
            return Err(VolatilityError::InsufficientData(
                "Not enough candles to find ATR spike".to_string(),
            ));
        }

        let mut max_atr = 0.0;
        let mut max_index = 0;

        // Fenêtre glissante pour calculer ATR à chaque point
        for i in atr_period..post_candles.len() {
            let window = &post_candles[i - atr_period..i];
            let calculator = MetricsCalculator::new(window);

            if let Ok(atr_values) = calculator.calculate_atr(atr_period) {
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
            (peak_time - self.event_time).num_minutes()
        } else {
            0
        };

        Ok((max_atr, minutes_to_peak))
    }

    /// Classifie le niveau de volatilité baseline
    fn classify_volatility(&self, atr: f64) -> VolatilityLevel {
        // Calcul relatif basé sur le prix (pour Forex, ATR en pips)
        let sample_candle = self
            .candles
            .first()
            .expect("Candles should not be empty");
        let price = sample_candle.close;

        // ATR relatif en %
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
    fn recommend_multipliers(
        &self,
        atr_ratio: f64,
        volatility_level: VolatilityLevel,
    ) -> (f64, f64) {
        // Logique adaptative : plus l'événement est volatile, plus on espace les stops

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

        // Ajuster selon l'impact de l'événement
        let sl_multiplier: f64 = if atr_ratio > 2.0 {
            base_sl + 0.5 // Événement très volatile : éloigner SL
        } else if atr_ratio > 1.5 {
            base_sl
        } else {
            base_sl - 0.3 // Événement peu volatil : rapprocher SL
        };

        let tp_multiplier: f64 = if atr_ratio > 2.0 {
            base_tp + 1.0 // Plus de potentiel de mouvement
        } else if atr_ratio > 1.5 {
            base_tp
        } else {
            base_tp - 0.5
        };

        (sl_multiplier.max(1.0), tp_multiplier.max(1.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candle(minutes_offset: i64, price: f64, range: f64) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: DateTime::from_timestamp(1609459200 + (minutes_offset * 60), 0)
                .expect("Invalid timestamp")
                .into(),
            open: price,
            high: price + range,
            low: price - range,
            close: price + range / 2.0,
            volume: 100.0,
        }
    }

    #[test]
    fn test_contextual_atr_analysis() {
        let mut candles = Vec::new();

        // 60min avant : volatilité normale (ATR ~0.0010)
        for i in 0..60 {
            candles.push(create_test_candle(-(60 - i), 1.1000, 0.0010));
        }

        // 120min après : volatilité élevée (ATR ~0.0030)
        for i in 0..120 {
            let increased_range = 0.0030;
            candles.push(create_test_candle(i, 1.1000, increased_range));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let analyzer = ContextualAtrAnalyzer::new(&candles, event_time);

        let result = analyzer.analyze(14).unwrap();

        // Vérifier que l'ATR après > ATR avant
        assert!(result.atr_after_event > result.atr_before_event);

        // Vérifier ratio cohérent
        assert!(result.atr_ratio > 1.0);

        // Vérifier recommandations valides
        assert!(result.recommended_sl_multiplier >= 1.0);
        assert!(result.recommended_tp_multiplier >= 1.5);
    }

    #[test]
    fn test_volatility_classification() {
        let mut candles = Vec::new();

        for i in 0..100 {
            candles.push(create_test_candle(i, 1.1000, 0.0005)); // Faible volatilité
        }

        let event_time = DateTime::from_timestamp(1609459200 + 3000, 0)
            .expect("Invalid timestamp")
            .into();
        let analyzer = ContextualAtrAnalyzer::new(&candles, event_time);

        let result = analyzer.analyze(14).unwrap();

        assert_eq!(result.baseline_volatility_level, VolatilityLevel::Low);
    }
}
