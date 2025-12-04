// services/movement_quality_analyzer.rs
// Analyse la qualité du mouvement (propreté, directionnalité)

use crate::models::Candle;

#[derive(Debug, Clone, Default)]
pub struct MovementQualityResult {
    #[allow(dead_code)]
    pub trend_score: f64, // 0-100: Force de la tendance
    #[allow(dead_code)]
    pub smoothness_score: f64, // 0-100: Régularité du mouvement (peu de retracements)
    #[allow(dead_code)]
    pub candle_consistency: f64, // 0-100: Cohérence des bougies (couleur uniforme)
    pub overall_quality: f64,  // Moyenne pondérée
    pub quality_label: String, // "Excellent", "Bon", "Moyen", "Faible"
}

/// Analyse la qualité du mouvement sur un ensemble de bougies
pub fn analyze_movement_quality(candles: &[Candle]) -> MovementQualityResult {
    if candles.len() < 5 {
        return MovementQualityResult::default();
    }

    // 1. Trend Score: Basé sur la pente de régression linéaire ou simplement Higher Highs / Lower Lows
    let trend_score = calculate_trend_score(candles);

    // 2. Smoothness Score: Basé sur la somme des retracements inverses
    let smoothness_score = calculate_smoothness_score(candles);

    // 3. Candle Consistency: % de bougies de la même couleur que la tendance
    let candle_consistency = calculate_candle_consistency(candles);

    // Score global
    let overall = (trend_score * 0.4) + (smoothness_score * 0.3) + (candle_consistency * 0.3);

    let label = if overall >= 80.0 {
        "Excellent".to_string()
    } else if overall >= 60.0 {
        "Bon".to_string()
    } else if overall >= 40.0 {
        "Moyen".to_string()
    } else {
        "Faible".to_string()
    };

    MovementQualityResult {
        trend_score,
        smoothness_score,
        candle_consistency,
        overall_quality: overall,
        quality_label: label,
    }
}

fn calculate_trend_score(candles: &[Candle]) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }
    let start_price = candles.first().map(|c| c.open).unwrap_or(0.0);
    let end_price = candles.last().map(|c| c.close).unwrap_or(0.0);
    let is_uptrend = end_price > start_price;

    let mut score = 0.0;
    let total_points = (candles.len() - 1) as f64;

    for i in 1..candles.len() {
        let curr = &candles[i];
        let prev = &candles[i - 1];

        if is_uptrend {
            if curr.high > prev.high {
                score += 1.0;
            }
            if curr.low > prev.low {
                score += 1.0;
            }
        } else {
            if curr.low < prev.low {
                score += 1.0;
            }
            if curr.high < prev.high {
                score += 1.0;
            }
        }
    }

    // Normaliser sur 100 (score max possible = total_points * 2)
    (score / (total_points * 2.0)) * 100.0
}

fn calculate_smoothness_score(candles: &[Candle]) -> f64 {
    // Somme des mouvements absolus vs mouvement net
    if candles.is_empty() {
        return 0.0;
    }
    let mut total_movement = 0.0;
    let start_price = candles.first().map(|c| c.open).unwrap_or(0.0);
    let end_price = candles.last().map(|c| c.close).unwrap_or(0.0);
    let net_movement = (end_price - start_price).abs();

    for candle in candles {
        total_movement += candle.high - candle.low;
    }

    if total_movement == 0.0 {
        return 0.0;
    }

    // Ratio d'efficacité (Efficiency Ratio)
    let efficiency = net_movement / total_movement;

    // Normaliser: efficiency 1.0 = 100, 0.0 = 0
    efficiency * 100.0
}

fn calculate_candle_consistency(candles: &[Candle]) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }
    let start_price = candles.first().map(|c| c.open).unwrap_or(0.0);
    let end_price = candles.last().map(|c| c.close).unwrap_or(0.0);
    let is_uptrend = end_price > start_price;

    let mut consistent_candles = 0;
    for candle in candles {
        let is_green = candle.close > candle.open;
        if (is_uptrend && is_green) || (!is_uptrend && !is_green) {
            consistent_candles += 1;
        }
    }

    (consistent_candles as f64 / candles.len() as f64) * 100.0
}
