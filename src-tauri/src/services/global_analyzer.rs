// services/global_analyzer.rs - Moteur d'analyse statistique globale
use crate::models::{
    AnalysisFilters, BestPair, EventImpact, GlobalAnalysisResult, GlobalStats, GoldenHour, TradableEventType, StraddleSuccessRate, OptimalTimeWindow,
};
use crate::services::ArchiveService;
use super::global_analyzer_types::*;
use super::global_analyzer_helpers::*;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct GlobalAnalyzer {
    archive_service: ArchiveService,
}

impl GlobalAnalyzer {
    pub fn new(archive_service: ArchiveService) -> Self {
        GlobalAnalyzer { archive_service }
    }

    pub fn analyze_all_archives(&self, filters: Option<AnalysisFilters>) -> Result<GlobalAnalysisResult, String> {
        let archives = self.archive_service.list_archives()
            .map_err(|e| format!("Erreur lors de la récupération des archives: {}", e))?;

        let total_archives = archives.len();
        info!("Début analyse globale sur {} archives", total_archives);

        let mut weighted_data: Vec<WeightedArchiveData> = Vec::new();
        let mut filtered_archives: Vec<crate::models::Archive> = Vec::new();

        // Préparer les filtres
        let (start_date, end_date, target_pairs) = if let Some(ref f) = filters {
            (
                f.start_date.as_deref(),
                f.end_date.as_deref(),
                f.pairs.as_ref()
            )
        } else {
            (None, None, None)
        };

        for archive in archives {
            // 1. Filtre par date
            if let Some(start) = start_date {
                if let Ok(start_dt) = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d") {
                    let start_datetime = start_dt.and_hms_opt(0, 0, 0).unwrap_or_default();
                    if archive.created_at < start_datetime {
                        continue;
                    }
                }
            }
            if let Some(end) = end_date {
                if let Ok(end_dt) = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d") {
                    let end_datetime = end_dt.and_hms_opt(23, 59, 59).unwrap_or_default();
                    if archive.created_at > end_datetime {
                        continue;
                    }
                }
            }

            // Désérialisation
            let mut current_analyzable_data: Option<AnalyzableArchiveData> = None;

            match serde_json::from_str::<ArchiveWrapper>(&archive.data_json) {
                Ok(wrapper) => {
                    if !wrapper.analysis_result.symbol.is_empty() {
                        current_analyzable_data = Some(wrapper.analysis_result);
                    }
                },
                Err(_) => {
                    match serde_json::from_str::<AnalyzableArchiveData>(&archive.data_json) {
                        Ok(result) => {
                            if !result.symbol.is_empty() {
                                current_analyzable_data = Some(result);
                            }
                        },
                        Err(_) => {}
                    }
                }
            }

            // 2. Filtre par paire
            let mut passed_pair_filter = true;
            if let Some(pairs) = target_pairs {
                if !pairs.is_empty() {
                    let mut found_pair_in_archive = false;
                    
                    if let Some(ref data) = current_analyzable_data {
                        if pairs.contains(&data.symbol) {
                            found_pair_in_archive = true;
                        }
                    } else {
                        let json_str = &archive.data_json;
                        for pair in pairs {
                            if json_str.contains(&format!("\"pair\":\"{}\"", pair)) || json_str.contains(&format!("\"symbol\":\"{}\"", pair)) {
                                found_pair_in_archive = true;
                                break;
                            }
                        }
                    }
                    
                    if !found_pair_in_archive {
                        passed_pair_filter = false;
                    }
                }
            }

            if passed_pair_filter {
                if let Some(data) = current_analyzable_data {
                    // Calculer le poids temporel
                    let weight = calculate_temporal_weight(archive.created_at);
                    
                    info!("Archive {} lue avec succès: {} (poids: {:.2})", archive.id, data.symbol, weight);
                    
                    weighted_data.push(WeightedArchiveData {
                        data,
                        weight,
                        created_at: archive.created_at,
                    });
                }
                filtered_archives.push(archive);
            }
        }

        info!("Archives valides pour analyse : {}/{}", weighted_data.len(), total_archives);
        info!("Archives filtrées pour analyses avancées : {}", filtered_archives.len());

        if weighted_data.is_empty() && filtered_archives.is_empty() {
            return Err("Aucune archive compatible trouvée après filtrage.".to_string());
        }

        // 2. Calcul des métriques sur les données pondérées
        Ok(GlobalAnalysisResult {
            total_analyses: weighted_data.len(),
            total_days_analyzed: 0,
            filters_applied: filters,
            global_stats: self.compute_global_stats(&weighted_data),
            best_pairs: self.compute_best_pairs(&weighted_data),
            golden_hours: self.compute_golden_hours(&weighted_data),
            event_impacts: vec![],
            tradable_events: self.compute_tradable_events(&filtered_archives),
            pair_straddle_rates: self.compute_pair_straddle_rates(&filtered_archives),
            optimal_time_windows: self.compute_optimal_time_windows(&filtered_archives),
            generated_at: chrono::Local::now().to_string(),
        })
    }

    fn compute_global_stats(&self, results: &[WeightedArchiveData]) -> GlobalStats {
        if results.is_empty() {
            return GlobalStats {
                average_volatility: 0.0,
                average_confidence: 0.0,
                most_analyzed_pair: "-".to_string(),
                most_frequent_recommendation: "-".to_string(),
            };
        }

        // Collecter toutes les volatilités et confidences pour détecter outliers
        let all_volatilities: Vec<f64> = results.iter()
            .map(|r| r.data.global_metrics.as_ref().map(|m| m.mean_volatility).unwrap_or(0.0))
            .collect();
        
        let all_confidences: Vec<f64> = results.iter()
            .map(|r| r.data.confidence_score)
            .collect();

        // Calcul pondéré avec exclusion outliers
        let mut weighted_volatility_sum = 0.0;
        let mut weighted_confidence_sum = 0.0;
        let mut total_weight = 0.0;

        for r in results {
            let volatility = r.data.global_metrics.as_ref().map(|m| m.mean_volatility).unwrap_or(0.0);
            let confidence = r.data.confidence_score;
            
            // Exclure les outliers
            if !is_outlier(volatility, &all_volatilities) && !is_outlier(confidence, &all_confidences) {
                weighted_volatility_sum += volatility * r.weight;
                weighted_confidence_sum += confidence * r.weight;
                total_weight += r.weight;
            }
        }

        let avg_volatility = if total_weight > 0.0 { weighted_volatility_sum / total_weight } else { 0.0 };
        let avg_confidence = if total_weight > 0.0 { weighted_confidence_sum / total_weight } else { 0.0 };

        // Trouver la paire la plus analysée
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

    fn compute_best_pairs(&self, results: &[WeightedArchiveData]) -> Vec<BestPair> {
        let mut pair_stats: HashMap<String, (f64, f64, f64)> = HashMap::new(); // (weighted_vol, weighted_conf, total_weight)

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

    fn compute_golden_hours(&self, results: &[WeightedArchiveData]) -> Vec<GoldenHour> {
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

    fn compute_tradable_events(&self, archives: &[crate::models::Archive]) -> Vec<TradableEventType> {
        let mut event_stats: HashMap<String, (f64, f64, Vec<String>)> = HashMap::new(); // (sum_weighted_ratio, total_weight, pairs)

        for archive in archives {
            // On ne traite que les archives de corrélation événement/paire
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

                    // Calculer le ratio moyen event_volatility / baseline_volatility
                    let avg_ratio: f64 = pair_impacts.iter()
                        .map(|p| {
                            if p.baseline_volatility > 0.0 {
                                p.event_volatility / p.baseline_volatility
                            } else {
                                1.0
                            }
                        })
                        .sum::<f64>() / pair_impacts.len() as f64;

                    // Collecter les paires affectées
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

        // Convertir en TradableEventType et calculer le score
        let mut tradable_events: Vec<TradableEventType> = event_stats
            .into_iter()
            .map(|(event_name, (sum_ratio, count, mut pairs))| {
                let avg_volatility_increase = sum_ratio / count;
                
                // Score de tradabilité : (ratio - 1) * 100, plafonné à 100
                // Un ratio de 2.0 (volatilité doublée) = score de 100
                let tradability_score = ((avg_volatility_increase - 1.0) * 100.0).min(100.0).max(0.0);

                // Dédupliquer les paires
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

        // Trier par score décroissant
        tradable_events.sort_by(|a, b| b.tradability_score.partial_cmp(&a.tradability_score).unwrap_or(std::cmp::Ordering::Equal));
        tradable_events
    }

    fn compute_pair_straddle_rates(&self, archives: &[crate::models::Archive]) -> Vec<StraddleSuccessRate> {
        let mut pair_stats: HashMap<String, (Vec<f64>, Vec<String>)> = HashMap::new(); // (volatilities, event_names)

        for archive in archives {
            // On ne traite que les archives de corrélation paire/événement
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

                    // Collecter les volatilités et noms d'événements
                    let volatilities: Vec<f64> = events.iter()
                        .map(|e| e.volatility_total)
                        .collect();

                    let event_names: Vec<String> = events.iter()
                        .map(|e| e.name.clone())
                        .collect();

                    let entry = pair_stats.entry(pair).or_insert((Vec::new(), Vec::new()));
                    entry.0.extend(volatilities);
                    entry.1.extend(event_names);
                },
                Err(e) => {
                    warn!("Erreur lecture archive corrélation paire {}: {}", archive.id, e);
                }
            }
        }

        // Convertir en StraddleSuccessRate et calculer les métriques
        let mut straddle_rates: Vec<StraddleSuccessRate> = pair_stats
            .into_iter()
            .map(|(pair, (volatilities, event_names))| {
                let total_events = volatilities.len();
                
                // Calculer la volatilité moyenne
                let avg_volatility = if total_events > 0 {
                    volatilities.iter().sum::<f64>() / total_events as f64
                } else {
                    0.0
                };

                // Calculer le directional_move_rate (% de mouvements avec volatilité > moyenne)
                let directional_moves = volatilities.iter()
                    .filter(|&&v| v > avg_volatility * 0.5) // Mouvement significatif si > 50% de la moyenne
                    .count();
                let directional_move_rate = if total_events > 0 {
                    (directional_moves as f64 / total_events as f64) * 100.0
                } else {
                    0.0
                };

                // Calculer le whipsaw_rate (% de mouvements faibles/erratiques)
                let whipsaw_moves = volatilities.iter()
                    .filter(|&&v| v < avg_volatility * 0.3) // Whipsaw si < 30% de la moyenne
                    .count();
                let whipsaw_rate = if total_events > 0 {
                    (whipsaw_moves as f64 / total_events as f64) * 100.0
                } else {
                    0.0
                };

                // Score de straddle : directional_move_rate - whipsaw_rate
                let straddle_score = (directional_move_rate - whipsaw_rate).max(0.0);

                // Trouver les top 3 événements les plus impactants (volatilité la plus élevée)
                let mut event_vol_pairs: Vec<(String, f64)> = event_names.iter()
                    .zip(volatilities.iter())
                    .map(|(name, &vol)| (name.clone(), vol))
                    .collect();
                event_vol_pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                event_vol_pairs.truncate(3);
                
                let top_events: Vec<String> = event_vol_pairs.into_iter()
                    .map(|(name, _)| name)
                    .collect();

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

        // Trier par straddle_score décroissant
        straddle_rates.sort_by(|a, b| b.straddle_score.partial_cmp(&a.straddle_score).unwrap_or(std::cmp::Ordering::Equal));
        straddle_rates
    }

    fn compute_optimal_time_windows(&self, archives: &[crate::models::Archive]) -> Vec<OptimalTimeWindow> {
        // HashMap: event_name -> (count, sum_before_vol, sum_after_vol, pairs)
        let mut event_timing_stats: HashMap<String, (usize, f64, f64, Vec<String>)> = HashMap::new();

        // Analyser les archives de corrélation paire/événement pour extraire les patterns temporels
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

        // Convertir en OptimalTimeWindow
        let mut time_windows: Vec<OptimalTimeWindow> = event_timing_stats
            .into_iter()
            .map(|(event_type, (count, sum_before, sum_after, pairs))| {
                let avg_before_vol = if count > 0 { sum_before / count as f64 } else { 0.0 };
                let avg_after_vol = if count > 0 { sum_after / count as f64 } else { 0.0 };

                // Estimation du peak time basée sur le ratio before/after
                // Si after > before, le pic est probablement après l'événement
                let avg_peak_time_minutes = if avg_after_vol > avg_before_vol {
                    // Pic après l'événement, estimé entre 5-15 minutes
                    10.0 + (avg_after_vol / (avg_before_vol + 0.01)) * 5.0
                } else {
                    // Pic pendant ou juste après l'événement
                    2.0
                };

                // Estimation de la fenêtre d'entrée optimale
                // Plus la volatilité before est élevée, plus on peut entrer tôt
                let avg_entry_window_minutes = if avg_before_vol > 0.1 {
                    15.0 // Entrée 15 min avant si volatilité pré-événement élevée
                } else {
                    5.0 // Entrée 5 min avant si volatilité pré-événement faible
                };

                // Estimation du retour à la normale
                // Basé sur la volatilité totale
                let total_vol = avg_before_vol + avg_after_vol;
                let avg_return_to_normal_minutes = if total_vol > 0.5 {
                    60.0 // Événements très volatils prennent ~1h pour se calmer
                } else if total_vol > 0.2 {
                    30.0 // Événements moyens ~30 min
                } else {
                    15.0 // Événements faibles ~15 min
                };

                // Score de consistance basé sur le nombre d'occurrences
                let consistency_score = if count >= 20 {
                    90.0 + (count as f64 / 10.0).min(10.0) // Max 100
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

        // Trier par consistency_score décroissant (événements les plus fiables en premier)
        time_windows.sort_by(|a, b| b.consistency_score.partial_cmp(&a.consistency_score).unwrap_or(std::cmp::Ordering::Equal));
        time_windows
    }

    fn compute_event_impacts(&self, _results: &[AnalyzableArchiveData]) -> Vec<EventImpact> {
        vec![]
    }
}
