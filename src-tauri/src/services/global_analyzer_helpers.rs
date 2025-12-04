// services/global_analyzer_helpers.rs - Fonctions helper pour l'analyseur global

pub fn calculate_temporal_weight(archive_date: chrono::NaiveDateTime) -> f64 {
    let now = chrono::Local::now().naive_local();
    let age_days = (now - archive_date).num_days() as f64;
    let age_months = age_days / 30.0;

    if age_months < 3.0 {
        1.0
    } else if age_months < 6.0 {
        0.7
    } else {
        0.4
    }
}

pub fn is_outlier(value: f64, values: &[f64]) -> bool {
    if values.len() < 3 {
        return false;
    }
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();
    (value - mean).abs() > 3.0 * std_dev
}
