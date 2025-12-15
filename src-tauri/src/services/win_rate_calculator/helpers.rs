use crate::models::{Candle, Result, VolatilityError};

/// Résultat d'un trade simulé
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeOutcome {
    Win,
    Loss,
    Whipsaw,
}

/// Métriques de win rate
#[derive(Debug, Clone)]
pub struct WinRateMetrics {
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

/// Suit un trade jusqu'à TP, SL ou timeout
pub fn track_trade(
    candles: &[Candle],
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

    for candle in candles
        .iter()
        .skip(start_index + 1)
        .take(candles.len().min(start_index + max_minutes) - start_index - 1)
    {
        if is_long {
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

    if hit_tp {
        Ok(TradeOutcome::Win)
    } else if hit_sl {
        if was_in_profit {
            Ok(TradeOutcome::Whipsaw)
        } else {
            Ok(TradeOutcome::Loss)
        }
    } else {
        Ok(TradeOutcome::Loss)
    }
}

/// Calcule l'ATR à un index donné
pub fn calculer_atr_a_index(candles: &[Candle], index: usize) -> Result<f64> {
    use crate::services::metrics::MetricsCalculator;

    if index < 14 {
        return Err(VolatilityError::InsufficientData(
            "Not enough candles for ATR".to_string(),
        ));
    }

    let window = &candles[index - 14..=index];
    let calc = MetricsCalculator::new(window);
    let atrs = calc.calculer_atr(14)?;

    atrs.last()
        .copied()
        .ok_or_else(|| VolatilityError::InsufficientData("No ATR calculated".to_string()))
}

/// Trouve l'index de la bougie
pub fn find_candle_index(
    candles: &[Candle],
    target_time: chrono::DateTime<chrono::Utc>,
) -> Result<usize> {
    candles
        .iter()
        .position(|c| c.datetime >= target_time)
        .ok_or_else(|| {
            VolatilityError::InsufficientData("Target time not found in candles".to_string())
        })
}
