// services/global_analyzer_helpers.rs - Fonctions helper pour l'analyseur global

pub fn calculer_poids_temporel(archive_date: chrono::NaiveDateTime) -> f64 {
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
    let n = values.len() as f64;
    let mean: f64 = values.iter().sum::<f64>() / n;
    // Bessel's correction : diviser par (n-1) pour variance échantillon
    let variance: f64 =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();
    (value - mean).abs() > 3.0 * std_dev
}
