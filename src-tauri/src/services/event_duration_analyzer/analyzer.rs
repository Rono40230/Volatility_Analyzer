// services/event_duration_analyzer/analyzer.rs - Logique d'analyse de durée
// Conforme RÈGLE 5: < 300L par fichier

use crate::models::{Candle, Result, VolatilityError};
use crate::services::metrics::MetricsCalculator;
use chrono::DateTime;
use chrono::Utc;
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
        let baseline_atr = self.calculer_atr_reference()?;
        debug!("Baseline ATR (30min before): {:.6}", baseline_atr);

        // 2. Trouver index de l'événement
        let event_index = self.trouver_index_evenement()?;

        // 3. Mesurer durée du pic de volatilité
        let peak_duration = self.mesurer_duree_pic(event_index, baseline_atr)?;

        // 4. Mesurer temps de retour à la normale
        let return_duration = self.mesurer_retour_normale(event_index, baseline_atr)?;

        // 5. Trouver moment du pic maximum
        let peak_time = self.trouver_temps_pic(event_index, 120)?; // 2h max

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
    pub fn calculer_atr_reference(&self) -> Result<f64> {
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

        let candles_vec: Vec<Candle> = candles_before.iter().map(|c| (*c).clone()).collect();

        let calc = MetricsCalculator::new(&candles_vec);
        let atrs = calc.calculer_atr(14)?;

        let mean_atr = atrs.iter().sum::<f64>() / atrs.len() as f64;
        Ok(mean_atr)
    }

    /// Trouve l'index de la bougie correspondant à l'événement
    pub fn trouver_index_evenement(&self) -> Result<usize> {
        self.candles
            .iter()
            .position(|c| c.datetime >= self.event_time)
            .ok_or_else(|| {
                VolatilityError::InsufficientData("Event time not found in candles".to_string())
            })
    }

    /// Mesure la durée du pic de volatilité (ATR > baseline * 1.5)
    pub fn mesurer_duree_pic(&self, start_index: usize, baseline_atr: f64) -> Result<i32> {
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
            let atrs = calc.calculer_atr(14).unwrap_or_default();

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
    pub fn mesurer_retour_normale(&self, start_index: usize, baseline_atr: f64) -> Result<i32> {
        let threshold = baseline_atr * 1.1; // 110% de l'ATR normal
        let max_window = 240; // 4h max

        for i in start_index..self.candles.len().min(start_index + max_window) {
            if i + 14 >= self.candles.len() {
                break;
            }

            let window = &self.candles[i..i + 14];
            let calc = MetricsCalculator::new(window);
            let atrs = calc.calculer_atr(14).unwrap_or_default();

            if let Some(&current_atr) = atrs.last() {
                if current_atr < threshold {
                    return Ok((i - start_index) as i32);
                }
            }
        }

        Ok(max_window as i32) // Retourne max si pas de retour observé
    }

    /// Trouve le moment du pic maximum ATR après l'événement
    pub fn trouver_temps_pic(&self, start_index: usize, max_minutes: usize) -> Result<i64> {
        let mut max_atr = 0.0;
        let mut peak_minute = 0;

        for i in start_index..self.candles.len().min(start_index + max_minutes) {
            if i + 14 >= self.candles.len() {
                break;
            }

            let window = &self.candles[i..i + 14];
            let calc = MetricsCalculator::new(window);
            let atrs = calc.calculer_atr(14).unwrap_or_default();

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
