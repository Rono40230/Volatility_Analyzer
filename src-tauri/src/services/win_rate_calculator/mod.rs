mod helpers;

pub use helpers::{
    calculer_atr_a_index, find_candle_index, track_trade, TradeOutcome, WinRateMetrics,
};

use crate::models::{Candle, Result};
use chrono::DateTime;
use chrono::Utc;
use tracing::info;

/// Calculateur de win rate par simulation
pub struct WinRateCalculator<'a> {
    candles: &'a [Candle],
    event_time: DateTime<Utc>,
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
        let entry_time = self.event_time - chrono::Duration::minutes(entry_minutes_before as i64);
        let entry_index = find_candle_index(self.candles, entry_time)?;

        if entry_index < 14 {
            return Err(crate::models::VolatilityError::InsufficientData(
                "Not enough candles before entry for ATR calculation".to_string(),
            ));
        }

        let atr = calculer_atr_a_index(self.candles, entry_index)?;
        let entry_candle = &self.candles[entry_index];
        let entry_price = entry_candle.close;
        let sl_distance = atr * atr_multiplier_sl;
        let tp_distance = atr * atr_multiplier_tp;

        let sl_long = entry_price - sl_distance;
        let tp_long = entry_price + tp_distance;

        let sl_short = entry_price + sl_distance;
        let tp_short = entry_price - tp_distance;

        let outcome_long = track_trade(
            self.candles,
            entry_index,
            entry_price,
            tp_long,
            sl_long,
            max_duration_minutes,
            true,
        )?;

        let outcome_short = track_trade(
            self.candles,
            entry_index,
            entry_price,
            tp_short,
            sl_short,
            max_duration_minutes,
            false,
        )?;

        match (outcome_long, outcome_short) {
            (TradeOutcome::Win, _) | (_, TradeOutcome::Win) => Ok(TradeOutcome::Win),
            (TradeOutcome::Whipsaw, TradeOutcome::Whipsaw) => Ok(TradeOutcome::Whipsaw),
            _ => Ok(TradeOutcome::Loss),
        }
    }

    /// Simule N trades avec mêmes paramètres pour calculer statistiques
    pub fn calculer_taux_reussite(
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
            avg_profit_pips: 0.0,
            avg_loss_pips: 0.0,
            risk_reward_ratio: atr_multiplier_tp / atr_multiplier_sl,
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
        let mut candles = Vec::new();

        for i in 0..30 {
            candles.push(create_test_candle(-(30 - i), 1.1000, 0.0010));
        }

        for i in 0..60 {
            let price = 1.1000 + (i as f64 * 0.0001);
            candles.push(create_test_candle(i, price, 0.0010));
        }

        let event_time = DateTime::from_timestamp(1609459200, 0)
            .expect("Invalid timestamp")
            .into();
        let calc = WinRateCalculator::new(&candles, event_time);

        let outcome = calc
            .simulate_trade(15, 2.0, 3.0, 60)
            .expect("Failed to simulate");
        assert_eq!(outcome, TradeOutcome::Win);
    }

    #[test]
    fn test_losing_trade() {
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

        let outcome = calc
            .simulate_trade(15, 2.0, 3.0, 60)
            .expect("Failed to simulate");
        assert_eq!(outcome, TradeOutcome::Loss);
    }
}
