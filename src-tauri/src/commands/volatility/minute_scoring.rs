// commands/volatility/minute_scoring.rs
// Logique de scoring des minutes (extracted)

use super::analyze_quarter_entry_timing_helpers::MinuteMetrics;

/// Score les métriques avec des seuils complexes
pub fn score_metrics(m: &MinuteMetrics) -> f64 {
    let mut score = 0.0;
    // Range scoring
    score += if m.range_percent > 2.5 {
        60.0
    } else if m.range_percent > 2.0 {
        50.0
    } else if m.range_percent > 1.5 {
        40.0
    } else if m.range_percent > 1.0 {
        20.0
    } else {
        0.0
    };
    // ATR scoring
    score += if m.atr_percent > 2.0 {
        25.0
    } else if m.atr_percent > 1.5 {
        20.0
    } else if m.atr_percent > 1.0 {
        15.0
    } else if m.atr_percent > 0.5 {
        8.0
    } else {
        0.0
    };
    // Body Range
    score += if m.body_range > 45.0 {
        15.0
    } else if m.body_range > 35.0 {
        12.0
    } else if m.body_range > 25.0 {
        8.0
    } else if m.body_range > 15.0 {
        3.0
    } else {
        0.0
    };
    // Noise Ratio
    score += if m.noise_ratio < 1.0 {
        8.0
    } else if m.noise_ratio < 1.5 {
        5.0
    } else if m.noise_ratio < 2.5 {
        2.0
    } else {
        0.0
    };
    // Volume + Breakout
    score += if m.volume_imbalance > 0.25 {
        8.0
    } else if m.volume_imbalance > 0.15 {
        5.0
    } else if m.volume_imbalance > 0.05 {
        2.0
    } else {
        0.0
    };
    score += if m.breakout_percent > 50.0 {
        12.0
    } else if m.breakout_percent > 25.0 {
        8.0
    } else if m.breakout_percent > 10.0 {
        4.0
    } else {
        0.0
    };
    score
}

/// Calcule le score de confiance (basé sur la consistance)
pub fn calculate_confidence(offsets: &[u8], optimal: u8) -> f64 {
    if offsets.is_empty() {
        return 0.0;
    }

    let optimal_i32 = optimal as i32;
    let variance: f64 = offsets
        .iter()
        .map(|&o| {
            let diff = (o as i32 - optimal_i32).abs() as f64;
            diff * diff
        })
        .sum::<f64>()
        / offsets.len() as f64;

    let std_dev = variance.sqrt();
    (100.0 - (std_dev * 10.0)).clamp(20.0, 100.0)
}
