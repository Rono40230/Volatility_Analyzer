use std::collections::HashMap;

/// Entry timing helper - group by offset and calculate metrics
pub fn calculate_entry_timing_metrics(
    results: &[(i16, f64, bool)],
) -> HashMap<i16, (f64, f64, f64, usize)> {
    let mut offset_groups: HashMap<i16, Vec<(f64, bool)>> = HashMap::new();

    for &(offset, pnl, is_win) in results {
        offset_groups
            .entry(offset)
            .or_insert_with(Vec::new)
            .push((pnl, is_win));
    }

    let mut metrics = HashMap::new();
    for offset in &[-10i16, -5, 0, 3] {
        if let Some(trades) = offset_groups.get(offset) {
            let win_count = trades.iter().filter(|(_, w)| *w).count();
            let win_rate = (win_count as f64 / trades.len() as f64) * 100.0;
            let avg_profit = trades.iter().map(|(p, _)| p).sum::<f64>() / trades.len() as f64;
            let whipsaw_rate = ((trades.len() - win_count) as f64 / trades.len() as f64) * 100.0;
            metrics.insert(*offset, (win_rate, avg_profit, whipsaw_rate, trades.len()));
        }
    }
    metrics
}

/// Calculate quality score from win rate, profit, and whipsaw rate
pub fn calculate_quality_score(win_rate: f64, avg_profit: f64, whipsaw_rate: f64) -> f64 {
    (win_rate * 0.5)
        + (avg_profit.clamp(-30.0, 30.0) * 1.33 * 0.4)
        + ((100.0 - whipsaw_rate) * 0.1)
}

/// Directional bias helper - calculate metrics from counts
pub fn calculate_bias_metrics(up_wins: usize, down_wins: usize, total_events: usize) -> (f64, f64, &'static str) {
    let total_wins = up_wins + down_wins;
    if total_wins == 0 {
        return (0.0, 0.0, "Neutral");
    }

    let bias_value = ((up_wins as f64 - down_wins as f64) / total_wins as f64);
    let asymmetry_percent = (bias_value.abs() * 100.0).round();

    let classification = if bias_value.abs() > 0.3 {
        if bias_value > 0.0 {
            "UpBiased"
        } else {
            "DownBiased"
        }
    } else {
        "Neutral"
    };

    (bias_value, asymmetry_percent, classification)
}

/// Confidence level based on event count
pub fn get_confidence_level(total_events: usize) -> &'static str {
    match total_events {
        0..=2 => "Very Low",
        3..=4 => "Low",
        5..=7 => "Medium",
        8..=11 => "High",
        _ => "Very High",
    }
}

/// Whipsaw root cause helper
pub fn classify_whipsaw_type(early_count: usize, late_count: usize) -> &'static str {
    if early_count > late_count {
        "Early"
    } else if late_count > early_count {
        "Late"
    } else {
        "Balanced"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_timing_grouping() {
        let results = vec![
            (0i16, 10.5, true),
            (0, 15.3, true),
            (0, -5.0, false),
            (-5, 8.0, true),
            (-5, -12.0, false),
        ];

        let metrics = calculate_entry_timing_metrics(&results);
        assert!(metrics.contains_key(&0));
        assert!(metrics.contains_key(&-5));
    }

    #[test]
    fn test_quality_score() {
        let score = calculate_quality_score(70.0, 15.0, 30.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_bias_calculation() {
        let (bias, asymmetry, classification) = calculate_bias_metrics(8, 2, 10);
        assert_eq!(classification, "UpBiased");
        assert!(asymmetry > 50.0);
    }

    #[test]
    fn test_confidence_levels() {
        assert_eq!(get_confidence_level(1), "Very Low");
        assert_eq!(get_confidence_level(5), "Medium");
        assert_eq!(get_confidence_level(15), "Very High");
    }

    #[test]
    fn test_whipsaw_classification() {
        assert_eq!(classify_whipsaw_type(10, 3), "Early");
        assert_eq!(classify_whipsaw_type(5, 7), "Late");
        assert_eq!(classify_whipsaw_type(5, 5), "Balanced");
    }
}
