// services/global_analyzer_metrics.rs
// Calcul des métriques globales (extracted compute functions)

use super::global_analyzer_event_analysis;
use super::global_analyzer_helpers::*;
use super::global_analyzer_straddle_calc;
use super::global_analyzer_types::*;
use crate::models::{BestPair, GlobalStats, GoldenHour, OptimalTimeWindow, StraddleSuccessRate};
use std::collections::HashMap;

pub fn compute_global_stats(results: &[WeightedArchiveData]) -> GlobalStats {
    if results.is_empty() {
        return GlobalStats {
            average_volatility: 0.0,
            average_confidence: 0.0,
            most_analyzed_pair: "-".to_string(),
            most_frequent_recommendation: "-".to_string(),
        };
    }

    let all_volatilities: Vec<f64> = results
        .iter()
        .map(|r| {
            r.data
                .global_metrics
                .as_ref()
                .map(|m| m.mean_volatility)
                .unwrap_or(0.0)
        })
        .collect();

    let all_confidences: Vec<f64> = results.iter().map(|r| r.data.confidence_score).collect();

    let mut weighted_volatility_sum = 0.0;
    let mut weighted_confidence_sum = 0.0;
    let mut total_weight = 0.0;

    for r in results {
        let volatility = r
            .data
            .global_metrics
            .as_ref()
            .map(|m| m.mean_volatility)
            .unwrap_or(0.0);
        let confidence = r.data.confidence_score;

        if !is_outlier(volatility, &all_volatilities) && !is_outlier(confidence, &all_confidences) {
            weighted_volatility_sum += volatility * r.weight;
            weighted_confidence_sum += confidence * r.weight;
            total_weight += r.weight;
        }
    }

    let avg_volatility = if total_weight > 0.0 {
        weighted_volatility_sum / total_weight
    } else {
        0.0
    };
    let avg_confidence = if total_weight > 0.0 {
        weighted_confidence_sum / total_weight
    } else {
        0.0
    };

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
        let vol = r
            .data
            .global_metrics
            .as_ref()
            .map(|m| m.mean_volatility)
            .unwrap_or(0.0);
        let entry = pair_stats
            .entry(r.data.symbol.clone())
            .or_insert((0.0, 0.0, 0.0));
        entry.0 += vol * r.weight;
        entry.1 += r.data.confidence_score * r.weight;
        entry.2 += r.weight;
    }

    let mut best_pairs: Vec<BestPair> = pair_stats
        .into_iter()
        .map(|(symbol, (weighted_vol, weighted_conf, total_weight))| {
            let avg_vol = if total_weight > 0.0 {
                weighted_vol / total_weight
            } else {
                0.0
            };
            let avg_conf = if total_weight > 0.0 {
                weighted_conf / total_weight
            } else {
                0.0
            };
            BestPair {
                symbol,
                score: avg_vol * avg_conf * 100.0,
                avg_volatility: avg_vol,
                win_rate: avg_conf,
                analysis_count: total_weight as usize,
            }
        })
        .collect();

    best_pairs.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
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
        .map(|(hour, weight)| GoldenHour {
            hour,
            score: weight,
            avg_volatility: 0.0,
            reliability: if total_weight > 0.0 {
                (weight / total_weight) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    golden_hours.sort_by(|a, b| {
        b.reliability
            .partial_cmp(&a.reliability)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    golden_hours.truncate(24);
    golden_hours
}

pub fn compute_tradable_events(
    archives: &[crate::models::Archive],
) -> Vec<crate::models::TradableEventType> {
    global_analyzer_event_analysis::compute_tradable_events(archives)
}

pub fn compute_pair_straddle_rates(
    archives: &[crate::models::Archive],
) -> Vec<StraddleSuccessRate> {
    global_analyzer_straddle_calc::compute_pair_straddle_rates(archives)
}

pub fn compute_optimal_time_windows(archives: &[crate::models::Archive]) -> Vec<OptimalTimeWindow> {
    global_analyzer_event_analysis::compute_optimal_time_windows(archives)
}
