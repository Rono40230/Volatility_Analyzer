// services/global_analyzer_metrics.rs
// Calcul des métriques globales (extracted compute functions)

use super::global_analyzer_event_analysis;
use super::global_analyzer_helpers::*;
use super::global_analyzer_types::*;
use crate::models::{BestPair, GlobalStats, GoldenHour, OptimalTimeWindow};
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
        most_frequent_recommendation: compute_recommendation(avg_volatility, avg_confidence),
    }
}

pub fn compute_best_pairs(results: &[WeightedArchiveData]) -> Vec<BestPair> {
    let mut pair_stats: HashMap<String, (f64, f64, f64, usize)> = HashMap::new();

    for r in results {
        let vol = r
            .data
            .global_metrics
            .as_ref()
            .map(|m| m.mean_volatility)
            .unwrap_or(0.0);
        let entry = pair_stats
            .entry(r.data.symbol.clone())
            .or_insert((0.0, 0.0, 0.0, 0));
        entry.0 += vol * r.weight;
        entry.1 += r.data.confidence_score * r.weight;
        entry.2 += r.weight;
        entry.3 += 1;
    }

    let mut best_pairs: Vec<BestPair> = pair_stats
        .into_iter()
        .map(|(symbol, (weighted_vol, weighted_conf, total_weight, count))| {
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
                analysis_count: count,
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
    let mut hour_volatilities: HashMap<u8, (f64, f64)> = HashMap::new(); // (sum_vol*weight, sum_weight)
    let mut total_weight = 0.0;

    for r in results {
        let vol = r
            .data
            .global_metrics
            .as_ref()
            .map(|m| m.mean_volatility)
            .unwrap_or(0.0);
        for &hour in &r.data.best_hours {
            *hour_weights.entry(hour).or_insert(0.0) += r.weight;
            let entry = hour_volatilities.entry(hour).or_insert((0.0, 0.0));
            entry.0 += vol * r.weight;
            entry.1 += r.weight;
        }
        total_weight += r.weight * r.data.best_hours.len() as f64;
    }

    let mut golden_hours: Vec<GoldenHour> = hour_weights
        .into_iter()
        .map(|(hour, weight)| {
            let avg_vol = hour_volatilities
                .get(&hour)
                .map(|(sum, w)| if *w > 0.0 { sum / w } else { 0.0 })
                .unwrap_or(0.0);
            GoldenHour {
                hour,
                score: weight,
                avg_volatility: avg_vol,
                reliability: if total_weight > 0.0 {
                    (weight / total_weight) * 100.0
                } else {
                    0.0
                },
            }
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

pub fn compute_optimal_time_windows(archives: &[crate::models::Archive]) -> Vec<OptimalTimeWindow> {
    global_analyzer_event_analysis::compute_optimal_time_windows(archives)
}

/// Recommandation dynamique basée sur volatilité et confiance moyennes
fn compute_recommendation(avg_volatility: f64, avg_confidence: f64) -> String {
    let high_vol = avg_volatility > 0.5;
    let high_conf = avg_confidence > 60.0;
    match (high_vol, high_conf) {
        (true, true) => "Forte volatilité + Haute confiance".to_string(),
        (true, false) => "Forte volatilité + Prudence".to_string(),
        (false, true) => "Faible volatilité + Haute confiance".to_string(),
        (false, false) => "Attendre / Pas de trade".to_string(),
    }
}

/// Calcule le nombre de jours couverts par les archives (date min → date max)
pub fn compute_total_days(results: &[WeightedArchiveData]) -> usize {
    if results.is_empty() {
        return 0;
    }
    let min_date = results.iter().map(|r| r.created_at).min();
    let max_date = results.iter().map(|r| r.created_at).max();
    match (min_date, max_date) {
        (Some(min), Some(max)) => {
            let diff = max.signed_duration_since(min);
            // Au minimum 1 jour si on a des données
            std::cmp::max(diff.num_days() as usize, 1)
        }
        _ => 0,
    }
}

/// Calcule les impacts d'événements depuis les archives de corrélation
pub fn compute_event_impacts(archives: &[crate::models::Archive]) -> Vec<crate::models::EventImpact> {
    use super::global_analyzer_types::EventImpactArchive;

    let mut event_map: HashMap<String, (f64, usize, String)> = HashMap::new(); // (sum_impact, count, currency)

    for archive in archives {
        if let Ok(event_archive) = serde_json::from_str::<EventImpactArchive>(&archive.data_json) {
            let ei = &event_archive.event_impact;
            if ei.pair_impacts.is_empty() {
                continue;
            }
            // Calculer l'impact moyen sur toutes les paires pour cet événement
            let avg_impact: f64 = ei.pair_impacts.iter()
                .map(|p| (p.event_volatility - p.baseline_volatility).max(0.0))
                .sum::<f64>() / ei.pair_impacts.len() as f64;

            let entry = event_map
                .entry(ei.event_name.clone())
                .or_insert((0.0, 0, ei.currency.clone()));
            entry.0 += avg_impact;
            entry.1 += 1;
        }
    }

    let mut impacts: Vec<crate::models::EventImpact> = event_map
        .into_iter()
        .map(|(event_name, (sum_impact, count, currency))| {
            let avg = if count > 0 { sum_impact / count as f64 } else { 0.0 };
            let impact_level = if avg > 5.0 {
                "High".to_string()
            } else if avg > 2.0 {
                "Medium".to_string()
            } else {
                "Low".to_string()
            };
            crate::models::EventImpact {
                event_name,
                currency,
                avg_impact_pips: avg,
                occurrence_count: count,
                impact_level,
            }
        })
        .collect();

    impacts.sort_by(|a, b| b.avg_impact_pips.partial_cmp(&a.avg_impact_pips).unwrap_or(std::cmp::Ordering::Equal));
    impacts
}
