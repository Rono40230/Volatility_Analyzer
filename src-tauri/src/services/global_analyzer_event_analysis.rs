// services/global_analyzer_event_analysis.rs
// Fonctions d'analyse des événements (extracted from global_analyzer_metrics)

use super::global_analyzer_types::*;
use crate::models::{OptimalTimeWindow, TradableEventType};
use std::collections::HashMap;
use tracing::warn;

pub fn compute_tradable_events(archives: &[crate::models::Archive]) -> Vec<TradableEventType> {
    let mut event_stats: HashMap<String, (f64, f64, Vec<String>)> = HashMap::new();

    for archive in archives {
        if !archive.archive_type.contains("Corrélation événement/paire") {
            continue;
        }

        match serde_json::from_str::<EventImpactArchive>(&archive.data_json) {
            Ok(event_data) => {
                let event_name = event_data.event_impact.event_name.clone();
                let pair_impacts = &event_data.event_impact.pair_impacts;

                if pair_impacts.is_empty() {
                    continue;
                }

                let avg_ratio: f64 = pair_impacts
                    .iter()
                    .map(|p| {
                        if p.baseline_volatility > 0.0 {
                            p.event_volatility / p.baseline_volatility
                        } else {
                            1.0
                        }
                    })
                    .sum::<f64>()
                    / pair_impacts.len() as f64;

                let affected_pairs: Vec<String> =
                    pair_impacts.iter().map(|p| p.symbol.clone()).collect();

                let entry = event_stats
                    .entry(event_name)
                    .or_insert((0.0, 0.0, Vec::new()));
                entry.0 += avg_ratio;
                entry.1 += 1.0;
                entry.2.extend(affected_pairs);
            }
            Err(e) => {
                warn!("Erreur lecture archive corrélation {}: {}", archive.id, e);
            }
        }
    }

    let mut tradable_events: Vec<TradableEventType> = event_stats
        .into_iter()
        .map(|(event_name, (sum_ratio, count, mut pairs))| {
            let avg_volatility_increase = sum_ratio / count;
            let tradability_score = ((avg_volatility_increase - 1.0) * 100.0).clamp(0.0, 100.0);
            pairs.sort();
            pairs.dedup();

            TradableEventType {
                event_name,
                occurrence_count: count as usize,
                avg_volatility_increase,
                tradability_score,
                affected_pairs: pairs,
            }
        })
        .collect();

    tradable_events.sort_by(|a, b| {
        b.tradability_score
            .partial_cmp(&a.tradability_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    tradable_events
}

pub fn compute_optimal_time_windows(archives: &[crate::models::Archive]) -> Vec<OptimalTimeWindow> {
    let mut event_timing_stats: HashMap<String, (usize, f64, f64, Vec<String>)> = HashMap::new();

    for archive in archives {
        if !archive.archive_type.contains("Corrélation paire/événement") {
            continue;
        }

        match serde_json::from_str::<PairCorrelationArchive>(&archive.data_json) {
            Ok(pair_data) => {
                let pair = pair_data.pair_correlation.pair.clone();

                for event in &pair_data.pair_correlation.events {
                    let entry = event_timing_stats.entry(event.name.clone()).or_insert((
                        0,
                        0.0,
                        0.0,
                        Vec::new(),
                    ));

                    entry.0 += event.count;
                    entry.1 += event.volatility_before;
                    entry.2 += event.volatility_after;

                    if !entry.3.contains(&pair) {
                        entry.3.push(pair.clone());
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Erreur lecture archive corrélation paire pour timing {}: {}",
                    archive.id, e
                );
            }
        }
    }

    let mut time_windows: Vec<OptimalTimeWindow> = event_timing_stats
        .into_iter()
        .map(|(event_type, (count, sum_before, sum_after, pairs))| {
            let avg_before_vol = if count > 0 {
                sum_before / count as f64
            } else {
                0.0
            };
            let avg_after_vol = if count > 0 {
                sum_after / count as f64
            } else {
                0.0
            };

            let avg_peak_time_minutes = if avg_after_vol > avg_before_vol {
                (10.0 + (avg_after_vol / (avg_before_vol + 0.01)) * 5.0).min(120.0)
            } else {
                2.0
            };

            let avg_entry_window_minutes = if avg_before_vol > 0.1 { 15.0 } else { 5.0 };

            let total_vol = avg_before_vol + avg_after_vol;
            let avg_return_to_normal_minutes = if total_vol > 0.5 {
                60.0
            } else if total_vol > 0.2 {
                30.0
            } else {
                15.0
            };

            let consistency_score = if count >= 20 {
                90.0 + (count as f64 / 10.0).min(10.0)
            } else if count >= 10 {
                70.0 + (count as f64 * 2.0)
            } else if count >= 5 {
                50.0 + (count as f64 * 4.0)
            } else {
                (count as f64 * 10.0).min(40.0)
            };

            OptimalTimeWindow {
                event_type,
                occurrence_count: count,
                avg_peak_time_minutes,
                avg_entry_window_minutes,
                avg_return_to_normal_minutes,
                consistency_score,
                affected_pairs: pairs,
            }
        })
        .collect();

    time_windows.sort_by(|a, b| {
        b.consistency_score
            .partial_cmp(&a.consistency_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    time_windows
}
