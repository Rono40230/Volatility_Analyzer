// services/global_analyzer.rs - Moteur d'analyse statistique globale
use super::global_analyzer_helpers::calculate_temporal_weight;
use super::global_analyzer_metrics::*;
use super::global_analyzer_types::*;
use crate::models::{AnalysisFilters, GlobalAnalysisResult};
use crate::services::ArchiveService;
use tracing::info;

pub struct GlobalAnalyzer {
    archive_service: ArchiveService,
}

impl GlobalAnalyzer {
    pub fn new(archive_service: ArchiveService) -> Self {
        GlobalAnalyzer { archive_service }
    }

    pub fn analyze_all_archives(
        &self,
        filters: Option<AnalysisFilters>,
    ) -> Result<GlobalAnalysisResult, String> {
        let archives = self
            .archive_service
            .list_archives()
            .map_err(|e| format!("Erreur lors de la récupération des archives: {}", e))?;

        let total_archives = archives.len();
        info!("Début analyse globale sur {} archives", total_archives);

        let (weighted_data, filtered_archives) =
            self.filter_and_weight_archives(&archives, filters.as_ref())?;

        info!(
            "Archives valides pour analyse : {}/{}",
            weighted_data.len(),
            total_archives
        );
        info!(
            "Archives filtrées pour analyses avancées : {}",
            filtered_archives.len()
        );

        if weighted_data.is_empty() && filtered_archives.is_empty() {
            return Err("Aucune archive compatible trouvée après filtrage.".to_string());
        }

        Ok(GlobalAnalysisResult {
            total_analyses: weighted_data.len(),
            total_days_analyzed: 0,
            filters_applied: filters,
            global_stats: compute_global_stats(&weighted_data),
            best_pairs: compute_best_pairs(&weighted_data),
            golden_hours: compute_golden_hours(&weighted_data),
            event_impacts: vec![],
            tradable_events: compute_tradable_events(&filtered_archives),
            pair_straddle_rates: compute_pair_straddle_rates(&filtered_archives),
            optimal_time_windows: compute_optimal_time_windows(&filtered_archives),
            generated_at: chrono::Local::now().to_string(),
        })
    }

    fn filter_and_weight_archives(
        &self,
        archives: &[crate::models::Archive],
        filters: Option<&AnalysisFilters>,
    ) -> Result<(Vec<WeightedArchiveData>, Vec<crate::models::Archive>), String> {
        let mut weighted_data: Vec<WeightedArchiveData> = Vec::new();
        let mut filtered_archives: Vec<crate::models::Archive> = Vec::new();

        let (start_date, end_date, target_pairs) = if let Some(f) = filters {
            (
                f.start_date.as_deref(),
                f.end_date.as_deref(),
                f.pairs.as_ref(),
            )
        } else {
            (None, None, None)
        };

        for archive in archives {
            if !self.passes_date_filter(archive, start_date, end_date) {
                continue;
            }

            let mut current_analyzable_data: Option<AnalyzableArchiveData> = None;
            match serde_json::from_str::<ArchiveWrapper>(&archive.data_json) {
                Ok(wrapper) => {
                    if !wrapper.analysis_result.symbol.is_empty() {
                        current_analyzable_data = Some(wrapper.analysis_result);
                    }
                }
                Err(_) => {
                    if let Ok(result) =
                        serde_json::from_str::<AnalyzableArchiveData>(&archive.data_json)
                    {
                        if !result.symbol.is_empty() {
                            current_analyzable_data = Some(result);
                        }
                    }
                }
            }

            if !self.passes_pair_filter(&current_analyzable_data, archive, target_pairs) {
                continue;
            }

            if let Some(data) = current_analyzable_data {
                let created_at_dt =
                    chrono::NaiveDateTime::parse_from_str(&archive.created_at, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or_else(|_| {
                            // Fallback: utiliser l'époque Unix (1970-01-01)
                            chrono::NaiveDateTime::new(
                                chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                                    .expect("epoch date is valid"),
                                chrono::NaiveTime::from_hms_opt(0, 0, 0)
                                    .expect("epoch time is valid"),
                            )
                        });
                let weight = calculate_temporal_weight(created_at_dt);
                info!(
                    "Archive {} lue avec succès: {} (poids: {:.2})",
                    archive.id, data.symbol, weight
                );
                weighted_data.push(WeightedArchiveData {
                    data,
                    weight,
                    created_at: created_at_dt,
                });
            }
            filtered_archives.push(archive.clone());
        }

        Ok((weighted_data, filtered_archives))
    }

    fn passes_date_filter(
        &self,
        archive: &crate::models::Archive,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> bool {
        let archive_dt =
            chrono::NaiveDateTime::parse_from_str(&archive.created_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| {
                    // Fallback: utiliser l'époque Unix (1970-01-01)
                    chrono::NaiveDateTime::new(
                        chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch date is valid"),
                        chrono::NaiveTime::from_hms_opt(0, 0, 0).expect("epoch time is valid"),
                    )
                });
        if let Some(start) = start_date {
            if let Ok(start_dt) = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d") {
                let start_datetime = start_dt.and_hms_opt(0, 0, 0).unwrap_or_default();
                if archive_dt < start_datetime {
                    return false;
                }
            }
        }
        if let Some(end) = end_date {
            if let Ok(end_dt) = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d") {
                let end_datetime = end_dt.and_hms_opt(23, 59, 59).unwrap_or_default();
                if archive_dt > end_datetime {
                    return false;
                }
            }
        }
        true
    }

    fn passes_pair_filter(
        &self,
        data: &Option<AnalyzableArchiveData>,
        archive: &crate::models::Archive,
        target_pairs: Option<&Vec<String>>,
    ) -> bool {
        if let Some(pairs) = target_pairs {
            if !pairs.is_empty() {
                let mut found = false;
                if let Some(d) = data {
                    if pairs.contains(&d.symbol) {
                        found = true;
                    }
                } else {
                    for pair in pairs {
                        if archive
                            .data_json
                            .contains(&format!("\"pair\":\"{}\"", pair))
                            || archive
                                .data_json
                                .contains(&format!("\"symbol\":\"{}\"", pair))
                        {
                            found = true;
                            break;
                        }
                    }
                }
                return found;
            }
        }
        true
    }
}
