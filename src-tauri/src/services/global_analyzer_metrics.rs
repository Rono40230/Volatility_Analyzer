// services/global_analyzer_metrics.rs
// Calcul des métriques globales (extracted compute functions)

use crate::models::{
    BestPair, GlobalStats, GoldenHour, TradableEventType,
    StraddleSuccessRate, OptimalTimeWindow,
};
use super::global_analyzer_types::*;
use super::global_analyzer_helpers::*;
use super::global_analyzer_straddle_calc;
use std::collections::HashMap;
use tracing::warn;

pub fn compute_global_stats(results: &[WeightedArchiveData]) -> GlobalStats {
    if results.is_empty() {
        return GlobalStats {
            average_volatility: 0.0,
            average_confidence: 0.0,
            most_analyzed_pair: "-".to_string(),
            most_frequent_recommendation: "-".to_string(),
        };
    }

    let all_volatilities: Vec<f64> = results.iter()
        .map(|r| r.data.global_metrics.as_ref().map(|m| m.mean_volatility).unwrap_or(0.0))
        .collect();
    
    let all_confidences: Vec<f64> = results.iter()
        .map(|r| r.data.confidence_score)
        .collect();

    let mut weighted_volatility_sum = 0.0;
    let mut weighted_confidence_sum = 0.0;
    let mut total_weight = 0.0;

    for r in results {
        let volatility = r.data.global_metrics.as_ref().map(|m| m.mean_volatility).unwrap_or(0.0);
        let confidence = r.data.confidence_score;
        
        if !is_outlier(volatility, &all_volatilities) && !is_outlier(confidence, &all_confidences) {
            weighted_volatility_sum += volatility * r.weight;
            weighted_confidence_sum += confidence * r.weight;
            total_weight += r.weight;
        }
    }

    let avg_volatility = if total_weight > 0.0 { weighted_volatility_sum / total_weight } else { 0.0 };
    let avg_confidence = if total_weight > 0.0 { weighted_confidence_sum / total_weight } else { 0.0 };

    let mut pair_counts = HashMap::new();
    for r in results {
        *pair_counts.entry(&r.data.symbol).or_insert(0) += 1;
    }
    
    let most_analyzed_pair = pair_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(pair, _)| pair.clone())
        .unwrap_or_else(|| "-".to_string());

    GlobalStats {
        average_volatility: avg_volatility,
        average_confidence: avg_confidence,
        most_analyzed_pair,
        most_frequent_recommendation: "Scalp Prudent".to_string(),
    }
}

pub fn compute_best_pairs(results: &[WeightedArchiveData]) -> Vec<BestPair> {
    let mut pair_stats: HashMap<String, (f64, f64, f64)> = HashMap::new();

    for r in results {
        let vol = r.data.global_metrics.as_ref().map(|m| m.mean_volatility).unwrap_or(0.0);
        let entry = pair_stats.entry(r.data.symbol.clone()).or_insert((0.0, 0.0, 0.0));
        entry.0 += vol * r.weight;
        entry.1 += r.data.confidence_score * r.weight;
        entry.2 += r.weight;
    }

    let mut best_pairs: Vec<BestPair> = pair_stats
        .into_iter()
        .map(|(symbol, (weighted_vol, weighted_conf, total_weight))| {
            let avg_vol = if total_weight > 0.0 { weighted_vol / total_weight } else { 0.0 };
            let avg_conf = if total_weight > 0.0 { weighted_conf / total_weight } else { 0.0 };
            BestPair {
                symbol,
                score: avg_vol * avg_conf * 100.0,
                avg_volatility: avg_vol,
                win_rate: avg_conf,
                analysis_count: total_weight as usize,
            }
        })
        .collect();

    best_pairs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    best_pairs
}

pub fn compute_golden_hours(results: &[WeightedArchiveData]) -> Vec<GoldenHour> {
    let mut hour_weights: HashMap<u8, f64> = HashMap::new();
    let mut total_weight = 0.0;

    for r in results {
        for &hour in &r.data.best_hours {
            *hour_weights.entry(hour).or_insert(0.0) += r.weight;
        }
        total_weight += r.weight * r.data.best_hours.len() as f64;
    }

    let mut golden_hours: Vec<GoldenHour> = hour_weights
        .into_iter()
        .map(|(hour, weight)| {
            GoldenHour {
                hour,
                score: weight,
                avg_volatility: 0.0,
                reliability: if total_weight > 0.0 { (weight / total_weight) * 100.0 } else { 0.0 },
            }
        })
        .collect();

    golden_hours.sort_by(|a, b| b.reliability.partial_cmp(&a.reliability).unwrap_or(std::cmp::Ordering::Equal));
    golden_hours.truncate(24);
    golden_hours
}

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

                let avg_ratio: f64 = pair_impacts.iter()
                    .map(|p| {
                        if p.baseline_volatility > 0.0 {
                            p.event_volatility / p.baseline_volatility
                        } else {
                            1.0
                        }
                    })
                    .sum::<f64>() / pair_impacts.len() as f64;

                let affected_pairs: Vec<String> = pair_impacts.iter()
                    .map(|p| p.symbol.clone())
                    .collect();

                let entry = event_stats.entry(event_name).or_insert((0.0, 0.0, Vec::new()));
                entry.0 += avg_ratio;
                entry.1 += 1.0;
                entry.2.extend(affected_pairs);
            },
            Err(e) => {
                warn!("Erreur lecture archive corrélation {}: {}", archive.id, e);
            }
        }
    }

    let mut tradable_events: Vec<TradableEventType> = event_stats
        .into_iter()
        .map(|(event_name, (sum_ratio, count, mut pairs))| {
            let avg_volatility_increase = sum_ratio / count;
            let tradability_score = ((avg_volatility_increase - 1.0) * 100.0).min(100.0).max(0.0);
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

    tradable_events.sort_by(|a, b| b.tradability_score.partial_cmp(&a.tradability_score).unwrap_or(std::cmp::Ordering::Equal));
    tradable_events
}

pub fn compute_pair_straddle_rates(archives: &[crate::models::Archive]) -> Vec<StraddleSuccessRate> {
    global_analyzer_straddle_calc::compute_pair_straddle_rates(archives)
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
                    let entry = event_timing_stats
                        .entry(event.name.clone())
                        .or_insert((0, 0.0, 0.0, Vec::new()));
                    
                    entry.0 += event.count;
                    entry.1 += event.volatility_before;
                    entry.2 += event.volatility_after;
                    
                    if !entry.3.contains(&pair) {
                        entry.3.push(pair.clone());
                    }
                }
            },
            Err(e) => {
                warn!("Erreur lecture archive corrélation paire pour timing {}: {}", archive.id, e);
            }
        }
    }

    let mut time_windows: Vec<OptimalTimeWindow> = event_timing_stats
        .into_iter()
        .map(|(event_type, (count, sum_before, sum_after, pairs))| {
            let avg_before_vol = if count > 0 { sum_before / count as f64 } else { 0.0 };
            let avg_after_vol = if count > 0 { sum_after / count as f64 } else { 0.0 };

            let avg_peak_time_minutes = if avg_after_vol > avg_before_vol {
                10.0 + (avg_after_vol / (avg_before_vol + 0.01)) * 5.0
            } else {
                2.0
            };

            let avg_entry_window_minutes = if avg_before_vol > 0.1 {
                15.0
            } else {
                5.0
            };

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

    time_windows.sort_by(|a, b| b.consistency_score.partial_cmp(&a.consistency_score).unwrap_or(std::cmp::Ordering::Equal));
    time_windows
}
