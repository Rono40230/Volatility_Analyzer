// services/entry_timing_optimizer.rs - Optimisation timing d'entrée
// Conforme .clinerules : < 300L, pas d'unwrap()

use crate::models::{Candle, Result, VolatilityError};
use crate::services::win_rate_calculator::{WinRateCalculator, TradeOutcome};
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// Optimiseur de timing d'entrée
pub struct EntryTimingOptimizer<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
}

/// Résultat d'analyse pour un timing spécifique
#[derive(Debug, Clone)]
pub struct TimingAnalysis {
    pub minutes_before: i32,
    /// NOTE: Ces fields sont publics pour introspection mais non utilisés actuellement
    #[allow(dead_code)]
    pub win_count: usize,
    #[allow(dead_code)]
    pub loss_count: usize,
    #[allow(dead_code)]
    pub whipsaw_count: usize,
    pub win_rate: f64,
}

/// Résultat complet d'optimisation
#[derive(Debug, Clone)]
pub struct OptimalTimingResult {
    pub best_entry_minutes_before: i32,
    pub best_win_rate: f64,
    pub worst_entry_minutes_before: i32,
    pub worst_win_rate: f64,
    /// NOTE: Ce field est public pour introspection mais non utilisé actuellement
    #[allow(dead_code)]
    pub all_timings: Vec<TimingAnalysis>,
}

impl<'a> EntryTimingOptimizer<'a> {
    /// Crée un nouvel optimiseur
    pub fn new(candles: &'a [Candle], event_time: DateTime<Utc>) -> Self {
        Self {
            candles,
            event_time,
        }
    }

    /// Analyse tous les timings possibles et trouve l'optimal
    pub fn find_optimal_timing(
        &self,
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<OptimalTimingResult> {
        info!("Finding optimal entry timing for event at {}", self.event_time);

        // Timings à tester : -60, -45, -30, -15, -5, -1 minutes avant événement
        let timings_to_test = vec![60, 45, 30, 15, 5, 1];
        let mut results = Vec::new();

        for minutes_before in timings_to_test {
            debug!("Testing entry at -{} minutes", minutes_before);

            let analysis = self.analyze_timing(
                minutes_before,
                atr_multiplier_sl,
                atr_multiplier_tp,
                max_duration_minutes,
            )?;

            results.push(analysis);
        }

        // Trouver le meilleur et le pire
        let best = results
            .iter()
            .max_by(|a, b| a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| VolatilityError::InsufficientData("No timing results".to_string()))?;

        let worst = results
            .iter()
            .min_by(|a, b| a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| VolatilityError::InsufficientData("No timing results".to_string()))?;

        info!(
            "Optimal timing: -{}min ({}% win rate), Worst: -{}min ({}% win rate)",
            best.minutes_before,
            (best.win_rate * 100.0).round(),
            worst.minutes_before,
            (worst.win_rate * 100.0).round()
        );

        Ok(OptimalTimingResult {
            best_entry_minutes_before: best.minutes_before,
            best_win_rate: best.win_rate,
            worst_entry_minutes_before: worst.minutes_before,
            worst_win_rate: worst.win_rate,
            all_timings: results,
        })
    }

    /// Analyse un timing spécifique
    fn analyze_timing(
        &self,
        minutes_before: i32,
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<TimingAnalysis> {
        let calculator = WinRateCalculator::new(self.candles, self.event_time);

        // Pour un seul événement, on simule un trade
        let outcome = calculator.simulate_trade(
            minutes_before,
            atr_multiplier_sl,
            atr_multiplier_tp,
            max_duration_minutes,
        )?;

        let (wins, losses, whipsaws) = match outcome {
            TradeOutcome::Win => (1, 0, 0),
            TradeOutcome::Loss => (0, 1, 0),
            TradeOutcome::Whipsaw => (0, 0, 1),
        };

        let total = wins + losses + whipsaws;
        let win_rate = if total > 0 {
            wins as f64 / total as f64
        } else {
            0.0
        };

        Ok(TimingAnalysis {
            minutes_before,
            win_count: wins,
            loss_count: losses,
            whipsaw_count: whipsaws,
            win_rate,
        })
    }

    /// Analyse multiple événements pour statistiques robustes
    /// (À utiliser quand on a plusieurs occurrences du même type d'événement)
    /// NOTE: Cette fonction est conservée pour usage futur
    #[allow(dead_code)]
    pub fn analyze_multiple_events(
        events: &[(Vec<Candle>, DateTime<Utc>)],
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<OptimalTimingResult> {
        info!("Analyzing {} events for optimal timing", events.len());

        if events.is_empty() {
            return Err(VolatilityError::InsufficientData(
                "No events provided".to_string(),
            ));
        }

        let timings_to_test = vec![60, 45, 30, 15, 5, 1];
        let mut aggregated_results: Vec<TimingAnalysis> = Vec::new();

        for minutes_before in timings_to_test {
            let mut total_wins = 0;
            let mut total_losses = 0;
            let mut total_whipsaws = 0;

            // Tester ce timing sur tous les événements
            for (candles, event_time) in events {
                let calculator = WinRateCalculator::new(candles, *event_time);

                if let Ok(outcome) = calculator.simulate_trade(
                    minutes_before,
                    atr_multiplier_sl,
                    atr_multiplier_tp,
                    max_duration_minutes,
                ) {
                    match outcome {
                        TradeOutcome::Win => total_wins += 1,
                        TradeOutcome::Loss => total_losses += 1,
                        TradeOutcome::Whipsaw => total_whipsaws += 1,
                    }
                }
            }

            let total = total_wins + total_losses + total_whipsaws;
            let win_rate = if total > 0 {
                total_wins as f64 / total as f64
            } else {
                0.0
            };

            aggregated_results.push(TimingAnalysis {
                minutes_before,
                win_count: total_wins,
                loss_count: total_losses,
                whipsaw_count: total_whipsaws,
                win_rate,
            });
        }

        // Trouver meilleur/pire timing
        let best = aggregated_results
            .iter()
            .max_by(|a, b| a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| VolatilityError::InsufficientData("No results".to_string()))?;

        let worst = aggregated_results
            .iter()
            .min_by(|a, b| a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| VolatilityError::InsufficientData("No results".to_string()))?;

        Ok(OptimalTimingResult {
            best_entry_minutes_before: best.minutes_before,
            best_win_rate: best.win_rate,
            worst_entry_minutes_before: worst.minutes_before,
            worst_win_rate: worst.win_rate,
            all_timings: aggregated_results,
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
    fn test_find_optimal_timing() {
        // Créer scénario où entrée à -15min est optimale
        let mut candles = Vec::new();

        // 60min avant : prix stable
        for i in 0..60 {
            candles.push(create_test_candle(-(60 - i), 1.1000, 0.0010));
        }

        // Après événement : prix monte progressivement
        for i in 0..120 {
            let price = 1.1000 + (i as f64 * 0.00005);
            candles.push(create_test_candle(i, price, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let optimizer = EntryTimingOptimizer::new(&candles, event_time);

        let result = optimizer.find_optimal_timing(2.0, 3.0, 120).unwrap();

        // Vérifier qu'on a des résultats pour tous les timings
        assert_eq!(result.all_timings.len(), 6);

        // Vérifier que le meilleur timing est cohérent
        assert!(result.best_win_rate >= result.worst_win_rate);
        assert!(result.best_entry_minutes_before > 0);
    }

    #[test]
    fn test_timing_analysis() {
        let mut candles = Vec::new();

        for i in 0..60 {
            candles.push(create_test_candle(-(60 - i), 1.1000, 0.0010));
        }

        for i in 0..60 {
            let price = 1.1000 + (i as f64 * 0.0001);
            candles.push(create_test_candle(i, price, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let optimizer = EntryTimingOptimizer::new(&candles, event_time);

        let analysis = optimizer.analyze_timing(15, 2.0, 3.0, 60).unwrap();

        assert_eq!(analysis.minutes_before, 15);
        assert!(analysis.win_rate >= 0.0 && analysis.win_rate <= 1.0);
    }
}
