mod helpers;

pub use helpers::{
    calculer_atr_reference, calculer_atr_post_evenement, classifier_volatilite, trouver_pic_atr_max,
    recommander_multiplicateurs, ContextualAtrMetrics, VolatilityLevel,
};

use crate::models::{Candle, Result};
use chrono::DateTime;
use chrono::Utc;
use tracing::info;

/// Analyseur d'ATR contextuel pour événements économiques
pub struct ContextualAtrAnalyzer<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
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

        let atr_before = calculer_atr_reference(self.candles, self.event_time, atr_period)?;
        let atr_after = calculer_atr_post_evenement(self.candles, self.event_time, atr_period)?;

        let atr_ratio = if atr_before > 0.0 {
            atr_after / atr_before
        } else {
            1.0
        };

        let (max_spike, minutes_to_peak) =
            trouver_pic_atr_max(self.candles, self.event_time, atr_period)?;
        let volatility_level = classifier_volatilite(self.candles, atr_before);
        let (sl_mult, tp_mult) = recommander_multiplicateurs(atr_ratio, volatility_level.clone());

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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candle(minutes_offset: i64, price: f64, range: f64) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: DateTime::from_timestamp(1609459200 + (minutes_offset * 60), 0)
                .expect("Invalid timestamp"),
            open: price,
            high: price + range,
            low: price - range,
            close: price + range / 2.0,
            volume: 100.0,
            ..Default::default()
        }
    }

    #[test]
    fn test_contextual_atr_analysis() {
        let mut candles = Vec::new();

        for i in 0..60 {
            candles.push(create_test_candle(-(60 - i), 1.1000, 0.0010));
        }

        for i in 0..120 {
            let increased_range = 0.0030;
            candles.push(create_test_candle(i, 1.1000, increased_range));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp");
        let analyzer = ContextualAtrAnalyzer::new(&candles, event_time);

        let result = analyzer.analyze(14).expect("Failed to analyze");
        assert!(result.atr_after_event > result.atr_before_event);
        assert!(result.atr_ratio > 1.0);
        assert!(result.recommended_sl_multiplier >= 1.0);
        assert!(result.recommended_tp_multiplier >= 1.5);
    }

    #[test]
    fn test_volatility_classification() {
        let mut candles = Vec::new();

        for i in 0..100 {
            // Range très faible pour être classifié comme "Low"
            candles.push(create_test_candle(i, 1.1000, 0.00005));
        }

        let event_time = DateTime::from_timestamp(1609459200 + 3000, 0)
            .expect("Invalid timestamp");
        let analyzer = ContextualAtrAnalyzer::new(&candles, event_time);

        let result = analyzer.analyze(14).expect("Failed to analyze");
        assert_eq!(result.baseline_volatility_level, VolatilityLevel::Low);
    }
}
