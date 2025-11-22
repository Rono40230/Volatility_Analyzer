// services/global_analyzer_helpers.rs - Fonctions helper pour l'analyseur global
use super::global_analyzer_types::*;
use crate::models::AnalysisFilters;

pub fn calculate_temporal_weight(archive_date: chrono::NaiveDateTime) -> f64 {
    let now = chrono::Local::now().naive_local();
    let age_days = (now - archive_date).num_days() as f64;
    let age_months = age_days / 30.0;
    
    if age_months < 3.0 { 1.0 }
    else if age_months < 6.0 { 0.7 }
    else { 0.4 }
}

pub fn is_outlier(value: f64, values: &[f64]) -> bool {
    if values.len() < 3 { return false; }
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();
    (value - mean).abs() > 3.0 * std_dev
}

pub fn apply_date_filters(archive_date: chrono::NaiveDateTime, filters: &Option<AnalysisFilters>) -> bool {
    if let Some(ref f) = filters {
        if let Some(start) = &f.start_date {
            if let Ok(start_dt) = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d") {
                let start_datetime = start_dt.and_hms_opt(0, 0, 0).unwrap_or_default();
                if archive_date < start_datetime { return false; }
            }
        }
        if let Some(end) = &f.end_date {
            if let Ok(end_dt) = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d") {
                let end_datetime = end_dt.and_hms_opt(23, 59, 59).unwrap_or_default();
                if archive_date > end_datetime { return false; }
            }
        }
    }
    true
}

pub fn deserialize_archive_data(data_json: &str) -> Option<AnalyzableArchiveData> {
    if let Ok(wrapper) = serde_json::from_str::<ArchiveWrapper>(data_json) {
        if !wrapper.analysis_result.symbol.is_empty() {
            return Some(wrapper.analysis_result);
        }
    }
    if let Ok(result) = serde_json::from_str::<AnalyzableArchiveData>(data_json) {
        if !result.symbol.is_empty() {
            return Some(result);
        }
    }
    None
}

pub fn matches_pair_filter(data: &Option<AnalyzableArchiveData>, json_str: &str, target_pairs: Option<&Vec<String>>) -> bool {
    if let Some(pairs) = target_pairs {
        if pairs.is_empty() { return true; }
        
        if let Some(ref d) = data {
            if pairs.contains(&d.symbol) {
                return true;
            }
        } else {
            for pair in pairs {
                if json_str.contains(&format!("\"pair\":\"{}\"", pair)) || json_str.contains(&format!("\"symbol\":\"{}\"", pair)) {
                    return true;
                }
            }
        }
        false
    } else {
        true
    }
}
