// services/entry_window_optimizer.rs
// Optimise le moment d'entrée dans le créneau (Entry Window)

use crate::models::Candle;
use crate::services::straddle_simulator::normalize_to_pips;
use chrono::Timelike;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EntryWindowResult {
    pub optimal_offset_minutes: i64, // Minutes après le début du créneau (ex: 2 min)
    pub optimal_win_rate: f64,       // Win rate si on entre à ce moment
    #[allow(dead_code)]
    pub avg_drawdown_pips: f64, // Drawdown moyen avant profit
    #[allow(dead_code)]
    pub confidence_score: f64,
}

/// Analyse le meilleur moment pour entrer dans le trade
///
/// Méthode :
/// 1. Simule des entrées à chaque minute du créneau (0 à 14)
/// 2. Pour chaque minute, calcule le résultat potentiel (Win/Loss) sur l'historique
/// 3. Identifie la minute qui offre le meilleur Win Rate avec le moins de Drawdown
pub fn optimize_entry_window(candles: &[Candle], symbol: &str, quarter: u32) -> EntryWindowResult {
    if candles.is_empty() {
        return EntryWindowResult {
            optimal_offset_minutes: 0,
            optimal_win_rate: 0.0,
            avg_drawdown_pips: 0.0,
            confidence_score: 0.0,
        };
    }

    // Grouper les bougies par jour pour reconstituer la session
    let mut candles_by_day: HashMap<String, Vec<&Candle>> = HashMap::new();
    for candle in candles {
        let day_key = candle.datetime.date_naive().to_string();
        candles_by_day
            .entry(day_key)
            .or_default()
            .push(candle);
    }

    let start_minute_of_quarter = (quarter * 15) as i64;

    // Résultats par minute d'offset (0 à 14)
    // (wins, total, drawdown_sum)
    let mut results_by_offset: HashMap<i64, (usize, usize, f64)> = HashMap::new();

    for i in 0..15 {
        results_by_offset.insert(i, (0, 0, 0.0));
    }

    for (_day, day_candles) in candles_by_day.iter() {
        // Trier par heure
        let mut sorted_candles = day_candles.clone();
        sorted_candles.sort_by_key(|c| c.datetime);

        // Pour chaque offset possible (minute 0 à 14 du créneau)
        for offset in 0..15 {
            let target_minute = (start_minute_of_quarter + offset) % 60;

            // Trouver la bougie correspondant à cette minute
            if let Some(entry_candle) = sorted_candles
                .iter()
                .find(|c| c.datetime.minute() as i64 == target_minute)
            {
                let entry_price = entry_candle.close; // On suppose entrée à la clôture de la minute

                // Simuler le trade sur le reste du créneau (et un peu après si on avait les données, mais ici on se limite au créneau)
                // Pour simplifier, on regarde si le prix monte ou descend significativement après

                let mut max_profit: f64 = 0.0;
                let mut max_drawdown: f64 = 0.0;

                // Regarder les bougies suivantes dans le créneau
                for future_candle in sorted_candles.iter() {
                    if future_candle.datetime > entry_candle.datetime {
                        // On suppose un trade Long pour l'analyse de volatilité (ou Short, on prend le max mouvement)
                        // Ici on cherche juste si ça bouge

                        // Simplification: On considère un trade GAGNANT si le prix bouge de > 10 pips sans drawdown > 10 pips
                        // C'est une heuristique pour déterminer le "bon moment" où ça part

                        // On mesure la volatilité directionnelle
                        let move_up = future_candle.close - entry_price;
                        let move_down = entry_price - future_candle.close;

                        if move_up > 0.0 {
                            max_profit = max_profit.max(move_up);
                        }
                        if move_down > 0.0 {
                            // Drawdown pour un long
                            max_drawdown = max_drawdown.max(move_down);
                        }
                    }
                }

                // Critère de succès heuristique : Profit > 2 * Drawdown
                let is_success = max_profit > (max_drawdown * 1.5) && max_profit > 0.0;

                if let Some(entry) = results_by_offset.get_mut(&offset) {
                    entry.1 += 1; // Total samples
                    if is_success {
                        entry.0 += 1; // Wins
                    }
                    entry.2 += max_drawdown; // Drawdown sum
                }
            }
        }
    }

    // Trouver l'offset optimal
    let mut best_offset = 0;
    let mut best_score = -1.0;
    let mut best_win_rate = 0.0;
    let mut best_avg_drawdown = 0.0;

    for (offset, (wins, total, drawdown_sum)) in results_by_offset {
        if total < 10 {
            continue;
        }

        let win_rate = wins as f64 / total as f64;
        let avg_drawdown = drawdown_sum / total as f64;

        // Score: WinRate pondéré par Drawdown inverse
        // On veut maximiser WinRate et minimiser Drawdown
        let score = win_rate * (1.0 / (1.0 + avg_drawdown));

        if score > best_score {
            best_score = score;
            best_offset = offset;
            best_win_rate = win_rate;
            best_avg_drawdown = avg_drawdown;
        }
    }

    EntryWindowResult {
        optimal_offset_minutes: best_offset,
        optimal_win_rate: best_win_rate * 100.0,
        avg_drawdown_pips: normalize_to_pips(best_avg_drawdown, symbol).ceil(),
        confidence_score: if best_score > 0.0 { 80.0 } else { 0.0 },
    }
}
