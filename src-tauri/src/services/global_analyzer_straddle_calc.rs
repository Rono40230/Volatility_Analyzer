// services/global_analyzer_straddle_calc.rs
// Calcul du succès Straddle par paire

use super::global_analyzer_types::PairCorrelationArchive;
use crate::models::StraddleSuccessRate;
use std::collections::HashMap;
use tracing::warn;

pub fn compute_pair_straddle_rates(
    archives: &[crate::models::Archive],
) -> Vec<StraddleSuccessRate> {
    // (Volatilities, EventNames, BodyPcts)
    type PairStats = HashMap<String, (Vec<f64>, Vec<String>, Vec<f64>)>;
    let mut pair_stats: PairStats = HashMap::new();

    for archive in archives {
        if !archive.archive_type.contains("Corrélation paire/événement") {
            continue;
        }

        match serde_json::from_str::<PairCorrelationArchive>(&archive.data_json) {
            Ok(pair_data) => {
                let pair = pair_data.pair_correlation.pair.clone();
                let events = &pair_data.pair_correlation.events;

                if events.is_empty() {
                    continue;
                }

                let volatilities: Vec<f64> = events.iter().map(|e| e.volatility_total).collect();
                let body_pcts: Vec<f64> = events.iter().map(|e| e.body_pct).collect();
                let event_names: Vec<String> = events.iter().map(|e| e.name.clone()).collect();

                let entry = pair_stats.entry(pair).or_insert((Vec::new(), Vec::new(), Vec::new()));
                entry.0.extend(volatilities);
                entry.1.extend(event_names);
                entry.2.extend(body_pcts);
            }
            Err(e) => {
                warn!(
                    "Erreur lecture archive corrélation paire {}: {}",
                    archive.id, e
                );
            }
        }
    }

    let mut straddle_rates: Vec<StraddleSuccessRate> = pair_stats
        .into_iter()
        .map(|(pair, (volatilities, event_names, body_pcts))| {
            let total_events = volatilities.len();

            let avg_volatility = if total_events > 0 {
                volatilities.iter().sum::<f64>() / total_events as f64
            } else {
                0.0
            };

            // Directional Move: Volatility > 50% of avg AND Body > 40% of Range (Not a Doji)
            let directional_moves = volatilities
                .iter()
                .zip(body_pcts.iter())
                .filter(|(&v, &body)| v > avg_volatility * 0.5 && body > 40.0)
                .count();
            let directional_move_rate = if total_events > 0 {
                (directional_moves as f64 / total_events as f64) * 100.0
            } else {
                0.0
            };

            let whipsaw_moves = volatilities
                .iter()
                .filter(|&&v| v < avg_volatility * 0.3)
                .count();
            let whipsaw_rate = if total_events > 0 {
                (whipsaw_moves as f64 / total_events as f64) * 100.0
            } else {
                0.0
            };

            let straddle_score = (directional_move_rate - whipsaw_rate).max(0.0);

            let mut event_vol_pairs: Vec<(String, f64)> = event_names
                .iter()
                .zip(volatilities.iter())
                .map(|(name, &vol)| (name.clone(), vol))
                .collect();
            event_vol_pairs
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            event_vol_pairs.truncate(3);

            let top_events: Vec<String> =
                event_vol_pairs.into_iter().map(|(name, _)| name).collect();

            StraddleSuccessRate {
                pair,
                total_events,
                directional_move_rate,
                whipsaw_rate,
                avg_volatility,
                straddle_score,
                top_events,
            }
        })
        .collect();

    straddle_rates.sort_by(|a, b| {
        b.straddle_score
            .partial_cmp(&a.straddle_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    straddle_rates
}
