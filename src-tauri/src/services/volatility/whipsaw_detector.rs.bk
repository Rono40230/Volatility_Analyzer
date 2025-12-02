// services/volatility/whipsaw_detector.rs - Détection de fausses cassures (whipsaw)
use crate::models::Candle;

/// Calcule la fréquence des whipsaws (double déclenchement)
///
/// Un whipsaw se produit quand :
/// 1. Buy stop = prix + offset
/// 2. Sell stop = prix - offset
/// 3. Les 2 se déclenchent dans les 15 minutes suivantes = PERTE GARANTIE
///
/// Formule : whipsaw_frequency = nombre_whipsaws / total_trades
pub fn calculate_whipsaw_frequency(candles: &[Candle], offset_pips: f64) -> WhipsawAnalysis {
    if candles.len() < 16 {
        return WhipsawAnalysis::default();
    }

    let mut whipsaw_count = 0;
    let mut total_trades = 0;
    let mut whipsaws = Vec::new();

    // Parcourir chaque candle comme point d'entrée potentiel
    for i in 0..candles.len() - 15 {
        let entry_candle = &candles[i];
        let entry_price = entry_candle.close;

        // Ordres Straddle
        let buy_stop = entry_price + (offset_pips / 10000.0);
        let sell_stop = entry_price - (offset_pips / 10000.0);

        // Analyser les 15 candles suivantes
        let follow_up_candles = &candles[i + 1..=i + 15];

        // Vérifier déclenchements
        let buy_triggered = follow_up_candles.iter().any(|c| c.high >= buy_stop);
        let sell_triggered = follow_up_candles.iter().any(|c| c.low <= sell_stop);

        total_trades += 1;

        if buy_triggered && sell_triggered {
            whipsaw_count += 1;

            // Enregistrer le détail du whipsaw
            let buy_trigger_time = follow_up_candles
                .iter()
                .position(|c| c.high >= buy_stop)
                .unwrap_or(0);
            let sell_trigger_time = follow_up_candles
                .iter()
                .position(|c| c.low <= sell_stop)
                .unwrap_or(0);

            whipsaws.push(WhipsawDetail {
                entry_index: i,
                entry_price,
                buy_stop,
                sell_stop,
                buy_trigger_index: i + 1 + buy_trigger_time,
                sell_trigger_index: i + 1 + sell_trigger_time,
            });
        }
    }

    let whipsaw_frequency = if total_trades > 0 {
        whipsaw_count as f64 / total_trades as f64
    } else {
        0.0
    };

    WhipsawAnalysis {
        total_trades,
        whipsaw_count,
        whipsaw_frequency,
        offset_pips,
        candles_analyzed: candles.len(),
        whipsaws,
        risk_level: calculate_risk_level(whipsaw_frequency),
    }
}

/// Évalue le niveau de risque basé sur la fréquence de whipsaw
fn calculate_risk_level(whipsaw_frequency: f64) -> WhipsawRiskLevel {
    if whipsaw_frequency < 0.1 {
        WhipsawRiskLevel::VeryLow // < 10%
    } else if whipsaw_frequency < 0.2 {
        WhipsawRiskLevel::Low // 10-20%
    } else if whipsaw_frequency < 0.35 {
        WhipsawRiskLevel::Moderate // 20-35%
    } else if whipsaw_frequency < 0.5 {
        WhipsawRiskLevel::High // 35-50%
    } else {
        WhipsawRiskLevel::VeryHigh // > 50%
    }
}

/// Résultat de l'analyse des whipsaws
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WhipsawAnalysis {
    pub total_trades: usize,
    pub whipsaw_count: usize,
    pub whipsaw_frequency: f64,
    pub offset_pips: f64,
    pub candles_analyzed: usize,
    pub whipsaws: Vec<WhipsawDetail>,
    pub risk_level: WhipsawRiskLevel,
}

impl Default for WhipsawAnalysis {
    fn default() -> Self {
        Self {
            total_trades: 0,
            whipsaw_count: 0,
            whipsaw_frequency: 0.0,
            offset_pips: 0.0,
            candles_analyzed: 0,
            whipsaws: Vec::new(),
            risk_level: WhipsawRiskLevel::Unknown,
        }
    }
}

/// Détail d'un whipsaw survenu
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WhipsawDetail {
    pub entry_index: usize,
    pub entry_price: f64,
    pub buy_stop: f64,
    pub sell_stop: f64,
    pub buy_trigger_index: usize,
    pub sell_trigger_index: usize,
}

/// Niveau de risque basé sur la fréquence de whipsaw
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhipsawRiskLevel {
    VeryLow,  // < 10%
    Low,      // 10-20%
    Moderate, // 20-35%
    High,     // 35-50%
    VeryHigh, // > 50%
    Unknown,  // Données insuffisantes
}

impl WhipsawRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WhipsawRiskLevel::VeryLow => "Très Bas",
            WhipsawRiskLevel::Low => "Bas",
            WhipsawRiskLevel::Moderate => "Modéré",
            WhipsawRiskLevel::High => "Élevé",
            WhipsawRiskLevel::VeryHigh => "Très Élevé",
            WhipsawRiskLevel::Unknown => "Inconnu",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            WhipsawRiskLevel::VeryLow => "#22c55e",  // Green
            WhipsawRiskLevel::Low => "#84cc16",      // Lime
            WhipsawRiskLevel::Moderate => "#eab308", // Yellow
            WhipsawRiskLevel::High => "#f97316",     // Orange
            WhipsawRiskLevel::VeryHigh => "#ef4444", // Red
            WhipsawRiskLevel::Unknown => "#6b7280",  // Gray
        }
    }
}

/// PHASE 7a-2: Whipsaw Root Cause Analysis
/// Classifier whipsaws en "early" (avant peak) et "late" (après peak)

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_candle(i: usize, high: f64, low: f64) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now() + chrono::Duration::minutes(i as i64),
            open: 1.0855,
            high,
            low,
            close: 1.0855 + (high - 1.0855) / 2.0,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_whipsaw_detection_basic() {
        let mut candles = Vec::new();
        for i in 0..20 {
            candles.push(create_test_candle(i, 1.0875, 1.0835));
        }
        let result = calculate_whipsaw_frequency(&candles, 50.0);
        assert!(result.total_trades > 0);
    }

    #[test]
    fn test_whipsaw_empty_data() {
        let candles = vec![];
        let result = calculate_whipsaw_frequency(&candles, 50.0);
        assert_eq!(result.total_trades, 0);
    }
}
