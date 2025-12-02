// services/volatility/win_rate_calculator.rs - Simulation du win rate pour Straddle
use crate::models::Candle;

/// Simule le win rate d'un Straddle sur un ensemble de candles
/// 
/// Algorithme :
/// 1. Pour chaque "événement" (1ère bougie), on place une commande Straddle
/// 2. Buy stop = prix de fermeture + offset_pips
/// 3. Sell stop = prix de fermeture - offset_pips
/// 4. On analyse les 15 minutes suivantes (15 candles)
/// 5. Si les 2 ordres se déclenchent = WHIPSAW (perte) → pas compté comme win
/// 6. Si 1 seul se déclenche et fait profit = WIN
/// 7. Si 1 seul se déclenche mais pas de profit = LOSS
/// 8. Si aucun ne se déclenche = LOSS
pub fn simulate_straddle_win_rate(candles: &[Candle], offset_pips: f64) -> WinRateResult {
    if candles.len() < 16 {
        // Besoin au moins 16 candles (1 d'entrée + 15 de follow)
        return WinRateResult::default();
    }

    let mut trades = Vec::new();
    let mut wins = 0;
    let mut losses = 0;
    let mut whipsaws = 0;

    // Parcourir chaque candle comme point d'entrée potentiel
    // On s'arrête 15 candles avant la fin pour avoir assez de données
    for i in 0..candles.len() - 15 {
        let entry_candle = &candles[i];
        let entry_price = entry_candle.close;

        // Ordres Straddle
        let buy_stop = entry_price + (offset_pips / 10000.0);
        let sell_stop = entry_price - (offset_pips / 10000.0);

        // Analyser les 15 candles suivantes
        let follow_up_candles = &candles[i + 1..=i + 15];

        // Vérifier déclenchements
        let buy_triggered = follow_up_candles.iter().any(|c| c.high >= buy_stop);
        let sell_triggered = follow_up_candles.iter().any(|c| c.low <= sell_stop);

        let trade_result = if buy_triggered && sell_triggered {
            // WHIPSAW : les 2 se déclenchent = perte garantie
            whipsaws += 1;
            TradeResult::Whipsaw {
                entry_price,
                buy_stop,
                sell_stop,
            }
        } else if buy_triggered {
            // Seul le buy se déclenche
            let max_price = follow_up_candles
                .iter()
                .map(|c| c.high)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(buy_stop);
            let profit = max_price - buy_stop;

            // Besoin de profit > offset pour considérer comme WIN
            if profit > (offset_pips / 10000.0) {
                wins += 1;
                TradeResult::Win {
                    entry_price,
                    entry_side: "BUY",
                    entry_level: buy_stop,
                    max_level: max_price,
                    profit,
                }
            } else {
                losses += 1;
                TradeResult::Loss {
                    entry_price,
                    entry_side: "BUY",
                    entry_level: buy_stop,
                    exit_level: max_price,
                }
            }
        } else if sell_triggered {
            // Seul le sell se déclenche
            let min_price = follow_up_candles
                .iter()
                .map(|c| c.low)
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(sell_stop);
            let profit = sell_stop - min_price;

            if profit > (offset_pips / 10000.0) {
                wins += 1;
                TradeResult::Win {
                    entry_price,
                    entry_side: "SELL",
                    entry_level: sell_stop,
                    max_level: min_price,
                    profit,
                }
            } else {
                losses += 1;
                TradeResult::Loss {
                    entry_price,
                    entry_side: "SELL",
                    entry_level: sell_stop,
                    exit_level: min_price,
                }
            }
        } else {
            // Aucun déclenchement = pas tradé
            losses += 1;
            TradeResult::NoTrigger {
                entry_price,
                buy_stop,
                sell_stop,
            }
        };

        trades.push(trade_result);
    }

    let total_trades = wins + losses + whipsaws;
    let win_rate = if total_trades > 0 {
        wins as f64 / total_trades as f64
    } else {
        0.0
    };

    WinRateResult {
        total_trades,
        wins,
        losses,
        whipsaws,
        win_rate,
        candles_analyzed: candles.len(),
        offset_pips,
        trades,
    }
}

/// Résultat de la simulation du win rate
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WinRateResult {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate: f64,
    pub candles_analyzed: usize,
    pub offset_pips: f64,
    pub trades: Vec<TradeResult>,
}

impl Default for WinRateResult {
    fn default() -> Self {
        Self {
            total_trades: 0,
            wins: 0,
            losses: 0,
            whipsaws: 0,
            win_rate: 0.0,
            candles_analyzed: 0,
            offset_pips: 0.0,
            trades: Vec::new(),
        }
    }
}

/// Résultat détaillé d'un trade simulé
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TradeResult {
    Win {
        entry_price: f64,
        entry_side: &'static str,
        entry_level: f64,
        max_level: f64,
        profit: f64,
    },
    Loss {
        entry_price: f64,
        entry_side: &'static str,
        entry_level: f64,
        exit_level: f64,
    },
    Whipsaw {
        entry_price: f64,
        buy_stop: f64,
        sell_stop: f64,
    },
    NoTrigger {
        entry_price: f64,
        buy_stop: f64,
        sell_stop: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_rate_simulation() {
        // Créer 20 candles avec un pattern clair
        let mut candles = Vec::new();

        // 5 candles initiales
        use chrono::{DateTime, Utc};
        let base_time = DateTime::parse_from_rfc3339("2024-12-01T14:00:00Z")
            .expect("valid RFC3339 datetime")
            .with_timezone(&Utc);
        for i in 0..5 {
            candles.push(Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: base_time + chrono::Duration::minutes(i as i64),
                open: 1.0850,
                high: 1.0860,
                low: 1.0840,
                close: 1.0855,
                volume: 1000.0,
            });
        }

        // 15 candles avec un mouvement haut (win du buy)
        for i in 5..20 {
            candles.push(Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: base_time + chrono::Duration::minutes(i as i64),
                open: 1.0855 + (i as f64 - 5.0) * 0.0001,
                high: 1.0865 + (i as f64 - 5.0) * 0.0001,
                low: 1.0850 + (i as f64 - 5.0) * 0.0001,
                close: 1.0860 + (i as f64 - 5.0) * 0.0001,
                volume: 1000.0,
            });
        }

        let result = simulate_straddle_win_rate(&candles, 10.0);
        assert!(result.total_trades > 0);
        assert!(result.wins > 0);
    }

    #[test]
    fn test_empty_candles() {
        let candles = vec![];
        let result = simulate_straddle_win_rate(&candles, 10.0);
        assert_eq!(result.total_trades, 0);
        assert_eq!(result.win_rate, 0.0);
    }
}
