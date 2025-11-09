// services/event_duration_analyzer.rs - Analyse durée mouvement post-événement
// Conforme .clinerules : < 300L, pas d'unwrap()

use crate::models::{Candle, Result, VolatilityError};
use crate::services::metrics::MetricsCalculator;
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// Analyseur de durée de mouvement après événement
pub struct EventDurationAnalyzer<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
}

impl<'a> EventDurationAnalyzer<'a> {
    /// Crée un nouvel analyseur
    pub fn new(candles: &'a [Candle], event_time: DateTime<Utc>) -> Self {
        Self {
            candles,
            event_time,
        }
    }

    /// Analyse la durée du mouvement post-événement
    pub fn analyze(&self) -> Result<EventDurationMetrics> {
        info!("Analyzing event duration for event at {}", self.event_time);

        // 1. Calculer ATR de référence (30min avant événement)
        let baseline_atr = self.calculate_baseline_atr()?;
        debug!("Baseline ATR (30min before): {:.6}", baseline_atr);

        // 2. Trouver index de l'événement
        let event_index = self.find_event_index()?;

        // 3. Mesurer durée du pic de volatilité
        let peak_duration = self.measure_peak_duration(event_index, baseline_atr)?;

        // 4. Mesurer temps de retour à la normale
        let return_duration = self.measure_return_to_normal(event_index, baseline_atr)?;

        // 5. Trouver moment du pic maximum
        let peak_time = self.find_peak_time(event_index, 120)?; // 2h max

        info!(
            "Event duration analysis complete: peak={}min, return={}min",
            peak_duration, return_duration
        );

        Ok(EventDurationMetrics {
            peak_duration_minutes: peak_duration,
            return_to_normal_minutes: return_duration,
            peak_time_minutes: peak_time,
            baseline_atr,
        })
    }

    /// Calcule l'ATR de référence 30min avant l'événement
    fn calculate_baseline_atr(&self) -> Result<f64> {
        let thirty_min_before = self.event_time - chrono::Duration::minutes(30);

        let candles_before: Vec<&Candle> = self
            .candles
            .iter()
            .filter(|c| c.datetime < self.event_time && c.datetime >= thirty_min_before)
            .collect();

        if candles_before.len() < 10 {
            return Err(VolatilityError::InsufficientData(
                "Not enough candles before event for baseline ATR".to_string(),
            ));
        }

        let candles_vec: Vec<Candle> = candles_before
            .iter()
            .map(|c| (*c).clone())
            .collect();
        
        let calc = MetricsCalculator::new(&candles_vec);
        let atrs = calc.calculate_atr(14)?;

        let mean_atr = atrs.iter().sum::<f64>() / atrs.len() as f64;
        Ok(mean_atr)
    }

    /// Trouve l'index de la bougie correspondant à l'événement
    fn find_event_index(&self) -> Result<usize> {
        self.candles
            .iter()
            .position(|c| c.datetime >= self.event_time)
            .ok_or_else(|| {
                VolatilityError::InsufficientData("Event time not found in candles".to_string())
            })
    }

    /// Mesure la durée du pic de volatilité (ATR > baseline * 1.5)
    fn measure_peak_duration(&self, start_index: usize, baseline_atr: f64) -> Result<i32> {
        let threshold = baseline_atr * 1.5; // 150% de l'ATR normal
        let max_window = 180; // 3h max

        let mut duration_minutes = 0;
        let mut consecutive_normal = 0;

        for i in start_index..self.candles.len().min(start_index + max_window) {
            if i + 14 >= self.candles.len() {
                break;
            }

            // Calculer ATR instantané (fenêtre glissante de 14 bougies)
            let window = &self.candles[i..i + 14];
            let calc = MetricsCalculator::new(window);
            let atrs = calc.calculate_atr(14).unwrap_or_default();

            if atrs.is_empty() {
                continue;
            }

            let current_atr = atrs.last().copied().unwrap_or(0.0);

            if current_atr > threshold {
                duration_minutes += 1;
                consecutive_normal = 0;
            } else {
                consecutive_normal += 1;
                // Si 5 minutes consécutives sous le seuil, pic terminé
                if consecutive_normal >= 5 {
                    break;
                }
            }
        }

        Ok(duration_minutes)
    }

    /// Mesure le temps de retour à ATR normal (< baseline * 1.1)
    fn measure_return_to_normal(&self, start_index: usize, baseline_atr: f64) -> Result<i32> {
        let threshold = baseline_atr * 1.1; // 110% de l'ATR normal
        let max_window = 240; // 4h max

        for i in start_index..self.candles.len().min(start_index + max_window) {
            if i + 14 >= self.candles.len() {
                break;
            }

            let window = &self.candles[i..i + 14];
            let calc = MetricsCalculator::new(window);
            let atrs = calc.calculate_atr(14).unwrap_or_default();

            if let Some(&current_atr) = atrs.last() {
                if current_atr < threshold {
                    return Ok((i - start_index) as i32);
                }
            }
        }

        Ok(max_window as i32) // Retourne max si pas de retour observé
    }

    /// Trouve le moment du pic maximum ATR après l'événement
    fn find_peak_time(&self, start_index: usize, max_minutes: usize) -> Result<i64> {
        let mut max_atr = 0.0;
        let mut peak_minute = 0;

        for i in start_index..self.candles.len().min(start_index + max_minutes) {
            if i + 14 >= self.candles.len() {
                break;
            }

            let window = &self.candles[i..i + 14];
            let calc = MetricsCalculator::new(window);
            let atrs = calc.calculate_atr(14).unwrap_or_default();

            if let Some(&current_atr) = atrs.last() {
                if current_atr > max_atr {
                    max_atr = current_atr;
                    peak_minute = i - start_index;
                }
            }
        }

        Ok(peak_minute as i64)
    }
}

/// Métriques de durée d'un événement
#[derive(Debug, Clone)]
pub struct EventDurationMetrics {
    pub peak_duration_minutes: i32,
    pub return_to_normal_minutes: i32,
    pub peak_time_minutes: i64,
    pub baseline_atr: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candle(minutes_offset: i64, atr_level: f64) -> Candle {
        let base_price = 1.1000;
        let range = atr_level * 10000.0; // Convertir ATR en pips

        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: DateTime::from_timestamp(1609459200 + (minutes_offset * 60), 0)
                .expect("Invalid timestamp")
                .into(),
            open: base_price,
            high: base_price + range / 2.0,
            low: base_price - range / 2.0,
            close: base_price + range / 4.0,
            volume: 100.0,
        }
    }

    #[test]
    fn test_baseline_atr_calculation() {
        // Créer 30 bougies avec ATR stable avant événement
        let mut candles: Vec<Candle> = (0..30)
            .map(|i| create_test_candle(-(30 - i), 0.0001))
            .collect();

        // Ajouter bougies après événement
        candles.extend((0..30).map(|i| create_test_candle(i, 0.0003)));

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let analyzer = EventDurationAnalyzer::new(&candles, event_time);

        let baseline = analyzer.calculate_baseline_atr().unwrap();
        assert!(baseline > 0.0);
        assert!(baseline < 0.0002); // ATR normal
    }

    #[test]
    fn test_event_duration_analysis() {
        // Scénario : ATR normal → pic 2h → retour normal
        let mut candles = Vec::new();

        // 30min avant : ATR normal (0.0001)
        candles.extend((0..30).map(|i| create_test_candle(-(30 - i), 0.0001)));

        // 60min pic : ATR élevé (0.0003)
        candles.extend((0..60).map(|i| create_test_candle(i, 0.0003)));

        // 30min retour : ATR normal
        candles.extend((60..90).map(|i| create_test_candle(i, 0.0001)));

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let analyzer = EventDurationAnalyzer::new(&candles, event_time);

        let metrics = analyzer.analyze().unwrap();

        // Pic devrait durer ~60 minutes
        assert!(metrics.peak_duration_minutes > 50);
        assert!(metrics.peak_duration_minutes < 70);

        // Retour normal après ~60 minutes
        assert!(metrics.return_to_normal_minutes > 50);
    }
}
