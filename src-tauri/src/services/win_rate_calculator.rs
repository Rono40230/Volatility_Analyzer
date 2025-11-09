// services/win_rate_calculator.rs - Calcul win rate par simulation de trades
// Conforme .clinerules : < 300L, pas d'unwrap()

use crate::models::{Candle, Result, VolatilityError};
use crate::services::metrics::MetricsCalculator;
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// Calculateur de win rate par simulation
pub struct WinRateCalculator<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
}

/// Résultat d'un trade simulé
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeOutcome {
    Win,
    Loss,
    Whipsaw, // Aller-retour (hit SL après avoir été en profit)
}

/// Métriques de win rate
#[derive(Debug, Clone)]
pub struct WinRateMetrics {
    /// NOTE: Ces fields sont publics pour introspection mais non utilisés actuellement
    #[allow(dead_code)]
    pub total_simulations: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate: f64,
    pub whipsaw_rate: f64,
    #[allow(dead_code)]
    pub avg_profit_pips: f64,
    #[allow(dead_code)]
    pub avg_loss_pips: f64,
    pub risk_reward_ratio: f64,
}

impl<'a> WinRateCalculator<'a> {
    /// Crée un nouveau calculateur
    pub fn new(candles: &'a [Candle], event_time: DateTime<Utc>) -> Self {
        Self {
            candles,
            event_time,
        }
    }

    /// Simule un trade avec paramètres donnés
    pub fn simulate_trade(
        &self,
        entry_minutes_before: i32,
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<TradeOutcome> {
        // 1. Trouver point d'entrée
        let entry_time = self.event_time - chrono::Duration::minutes(entry_minutes_before as i64);
        let entry_index = self.find_candle_index(entry_time)?;

        if entry_index < 14 {
            return Err(VolatilityError::InsufficientData(
                "Not enough candles before entry for ATR calculation".to_string(),
            ));
        }

        // 2. Calculer ATR au moment de l'entrée
        let atr = self.calculate_atr_at_index(entry_index)?;
        debug!("Entry ATR: {:.6}", atr);

        // 3. Définir prix d'entrée, SL et TP
        let entry_candle = &self.candles[entry_index];
        let entry_price = entry_candle.close;
        let sl_distance = atr * atr_multiplier_sl;
        let tp_distance = atr * atr_multiplier_tp;

        // Position longue (pour simplifier, on peut simuler les deux)
        let sl_long = entry_price - sl_distance;
        let tp_long = entry_price + tp_distance;

        // Position courte
        let sl_short = entry_price + sl_distance;
        let tp_short = entry_price - tp_distance;

        // 4. Suivre le trade sur max_duration_minutes
        let outcome_long = self.track_trade(
            entry_index,
            entry_price,
            tp_long,
            sl_long,
            max_duration_minutes,
            true,
        )?;

        let outcome_short = self.track_trade(
            entry_index,
            entry_price,
            tp_short,
            sl_short,
            max_duration_minutes,
            false,
        )?;

        // Retourner le meilleur résultat (Straddle = gagne des deux côtés en théorie)
        // Pour l'analyse, on compte WIN si au moins un côté gagne
        match (outcome_long, outcome_short) {
            (TradeOutcome::Win, _) | (_, TradeOutcome::Win) => Ok(TradeOutcome::Win),
            (TradeOutcome::Whipsaw, TradeOutcome::Whipsaw) => Ok(TradeOutcome::Whipsaw),
            _ => Ok(TradeOutcome::Loss),
        }
    }

    /// Simule N trades avec mêmes paramètres pour calculer statistiques
    pub fn calculate_win_rate(
        &self,
        entry_minutes_before: i32,
        atr_multiplier_sl: f64,
        atr_multiplier_tp: f64,
        max_duration_minutes: usize,
    ) -> Result<WinRateMetrics> {
        info!(
            "Calculating win rate: entry={}min, sl={}xATR, tp={}xATR",
            entry_minutes_before, atr_multiplier_sl, atr_multiplier_tp
        );

        // Pour un seul événement, on simule le trade une fois
        let outcome = self.simulate_trade(
            entry_minutes_before,
            atr_multiplier_sl,
            atr_multiplier_tp,
            max_duration_minutes,
        )?;

        let (wins, losses, whipsaws) = match outcome {
            TradeOutcome::Win => (1, 0, 0),
            TradeOutcome::Loss => (0, 1, 0),
            TradeOutcome::Whipsaw => (0, 0, 1),
        };

        let total = 1;
        let win_rate = wins as f64 / total as f64;
        let whipsaw_rate = whipsaws as f64 / total as f64;

        Ok(WinRateMetrics {
            total_simulations: total,
            wins,
            losses,
            whipsaws,
            win_rate,
            whipsaw_rate,
            avg_profit_pips: 0.0, // TODO: calculer profit/loss réel
            avg_loss_pips: 0.0,
            risk_reward_ratio: atr_multiplier_tp / atr_multiplier_sl,
        })
    }

    /// Suit un trade jusqu'à TP, SL ou timeout
    fn track_trade(
        &self,
        start_index: usize,
        entry_price: f64,
        tp: f64,
        sl: f64,
        max_minutes: usize,
        is_long: bool,
    ) -> Result<TradeOutcome> {
        let mut hit_tp = false;
        let mut hit_sl = false;
        let mut was_in_profit = false;

        for i in start_index + 1..self.candles.len().min(start_index + max_minutes) {
            let candle = &self.candles[i];

            if is_long {
                // Position longue
                if candle.high >= tp {
                    hit_tp = true;
                    break;
                }
                if candle.low <= sl {
                    hit_sl = true;
                    break;
                }
                if candle.close > entry_price {
                    was_in_profit = true;
                }
            } else {
                // Position courte
                if candle.low <= tp {
                    hit_tp = true;
                    break;
                }
                if candle.high >= sl {
                    hit_sl = true;
                    break;
                }
                if candle.close < entry_price {
                    was_in_profit = true;
                }
            }
        }

        // Déterminer l'outcome
        if hit_tp {
            Ok(TradeOutcome::Win)
        } else if hit_sl {
            if was_in_profit {
                Ok(TradeOutcome::Whipsaw)
            } else {
                Ok(TradeOutcome::Loss)
            }
        } else {
            // Timeout sans TP ni SL
            Ok(TradeOutcome::Loss)
        }
    }

    /// Calcule l'ATR à un index donné
    fn calculate_atr_at_index(&self, index: usize) -> Result<f64> {
        if index < 14 {
            return Err(VolatilityError::InsufficientData(
                "Not enough candles for ATR".to_string(),
            ));
        }

        let window = &self.candles[index - 14..=index];
        let calc = MetricsCalculator::new(window);
        let atrs = calc.calculate_atr(14)?;

        atrs.last()
            .copied()
            .ok_or_else(|| VolatilityError::InsufficientData("No ATR calculated".to_string()))
    }

    /// Trouve l'index de la bougie la plus proche d'un timestamp
    fn find_candle_index(&self, target_time: DateTime<Utc>) -> Result<usize> {
        self.candles
            .iter()
            .position(|c| c.datetime >= target_time)
            .ok_or_else(|| {
                VolatilityError::InsufficientData("Target time not found in candles".to_string())
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
    fn test_winning_trade() {
        // Créer scénario où le prix monte après événement
        let mut candles = Vec::new();

        // 30min avant avec ATR stable
        for i in 0..30 {
            candles.push(create_test_candle(-(30 - i), 1.1000, 0.0010));
        }

        // Après événement : prix monte progressivement
        for i in 0..60 {
            let price = 1.1000 + (i as f64 * 0.0001);
            candles.push(create_test_candle(i, price, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let calc = WinRateCalculator::new(&candles, event_time);

        let outcome = calc.simulate_trade(15, 2.0, 3.0, 60).unwrap();
        assert_eq!(outcome, TradeOutcome::Win);
    }

    #[test]
    fn test_losing_trade() {
        // Créer scénario où le prix chute
        let mut candles = Vec::new();

        for i in 0..30 {
            candles.push(create_test_candle(-(30 - i), 1.1000, 0.0010));
        }

        for i in 0..60 {
            let price = 1.1000 - (i as f64 * 0.0001);
            candles.push(create_test_candle(i, price, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let calc = WinRateCalculator::new(&candles, event_time);

        let outcome = calc.simulate_trade(15, 2.0, 3.0, 60).unwrap();
        assert_eq!(outcome, TradeOutcome::Loss);
    }
}
