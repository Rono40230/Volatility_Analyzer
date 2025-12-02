// services/directional_bias_analyzer.rs - Analyser asymétrie UP vs DOWN
// PHASE 7a-5: Directional Bias Analyzer
// Mesurer la tendance inhérente des événements

use crate::models::VolatilityError;
use serde::{Deserialize, Serialize};

/// Classification de la tendance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiasType {
    UpBiased,   // bias > 0.3 (30% asymétrie)
    DownBiased, // bias < -0.3
    Neutral,    // -0.3 to +0.3
}

impl BiasType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BiasType::UpBiased => "UP_BIASED",
            BiasType::DownBiased => "DOWN_BIASED",
            BiasType::Neutral => "NEUTRAL",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            BiasType::UpBiased => "#3b82f6",   // Blue
            BiasType::DownBiased => "#ef4444", // Red
            BiasType::Neutral => "#8b5cf6",    // Purple
        }
    }
}

/// Résultat complet de l'analyse de biais
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalBiasAnalysis {
    pub up_wins_count: usize,
    pub down_wins_count: usize,
    pub whipsaw_count: usize,
    pub up_bias: f64,           // -1.0 to +1.0
    pub asymmetry_percent: f64, // 0-100
    pub classification: BiasType,
    pub sample_size: usize,
    pub recommendation: String,
}

impl DirectionalBiasAnalysis {
    /// Analyser les biais directionnels
    ///
    /// Input:
    ///   - up_wins: nombre de gagnants acheteurs (Buy Stop)
    ///   - down_wins: nombre de gagnants vendeurs (Sell Stop)
    ///   - whipsaws: nombre de faux déclenchements
    ///
    /// Output:
    ///   - Structure avec biais, asymétrie et recommandation
    pub fn analyze(up_wins: usize, down_wins: usize, whipsaws: usize) -> Self {
        let total_wins = up_wins + down_wins;
        let sample_size = total_wins + whipsaws;

        if total_wins == 0 {
            return Self {
                up_wins_count: 0,
                down_wins_count: 0,
                whipsaw_count: whipsaws,
                up_bias: 0.0,
                asymmetry_percent: 0.0,
                classification: BiasType::Neutral,
                sample_size,
                recommendation: "Insufficient data for bias analysis".to_string(),
            };
        }

        // Calculer le biais: (up - down) / total
        let up_bias = ((up_wins as f64) - (down_wins as f64)) / (total_wins as f64);

        // Asymétrie = |biais| * 100
        let asymmetry_percent = up_bias.abs() * 100.0;

        // Classifier
        let classification = if up_bias > 0.3 {
            BiasType::UpBiased
        } else if up_bias < -0.3 {
            BiasType::DownBiased
        } else {
            BiasType::Neutral
        };

        // Générer recommandation
        let recommendation = match classification {
            BiasType::UpBiased => format!(
                "📈 Event has UPWARD bias: {:.0}% asymmetry. Use for directional UP trades.",
                asymmetry_percent
            ),
            BiasType::DownBiased => format!(
                "📉 Event has DOWNWARD bias: {:.0}% asymmetry. Use for directional DOWN trades.",
                asymmetry_percent
            ),
            BiasType::Neutral => format!(
                "⚖️ Event is NEUTRAL: {:.0}% asymmetry. Safe for Straddle strategy.",
                asymmetry_percent
            ),
        };

        Self {
            up_wins_count: up_wins,
            down_wins_count: down_wins,
            whipsaw_count: whipsaws,
            up_bias,
            asymmetry_percent,
            classification,
            sample_size,
            recommendation,
        }
    }
}

/// Analyseur principal
pub struct DirectionalBiasAnalyzer;

impl DirectionalBiasAnalyzer {
    /// Analyser un ensemble de résultats de trading
    ///
    /// Input: backtests (Vec de (entry_side, outcome))
    ///   - entry_side: "BUY" ou "SELL"
    ///   - outcome: "WIN" ou "LOSS"
    pub fn analyze_backtests(
        backtests: &[(&str, &str)],
    ) -> Result<DirectionalBiasAnalysis, VolatilityError> {
        if backtests.is_empty() {
            return Ok(DirectionalBiasAnalysis::analyze(0, 0, 0));
        }

        let mut up_wins = 0;
        let mut down_wins = 0;
        let mut whipsaws = 0;

        for (side, outcome) in backtests {
            match (*side, *outcome) {
                ("BUY", "WIN") | ("LONG", "WIN") => up_wins += 1,
                ("SELL", "WIN") | ("SHORT", "WIN") => down_wins += 1,
                (_, "LOSS") | (_, "WHIPSAW") => whipsaws += 1,
                _ => {} // Ignore other states
            }
        }

        Ok(DirectionalBiasAnalysis::analyze(
            up_wins, down_wins, whipsaws,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bias_up_biased() {
        // 70% UP wins
        let analysis = DirectionalBiasAnalysis::analyze(70, 30, 0);
        assert_eq!(analysis.classification, BiasType::UpBiased);
        assert!(analysis.up_bias > 0.3);
        assert!(analysis.asymmetry_percent > 30.0);
        assert!(analysis.recommendation.contains("UPWARD"));
    }

    #[test]
    fn test_bias_down_biased() {
        // 70% DOWN wins
        let analysis = DirectionalBiasAnalysis::analyze(30, 70, 0);
        assert_eq!(analysis.classification, BiasType::DownBiased);
        assert!(analysis.up_bias < -0.3);
        assert!(analysis.recommendation.contains("DOWNWARD"));
    }

    #[test]
    fn test_bias_neutral() {
        // 50/50 balanced
        let analysis = DirectionalBiasAnalysis::analyze(50, 50, 0);
        assert_eq!(analysis.classification, BiasType::Neutral);
        assert!(analysis.up_bias.abs() <= 0.3);
        assert!(analysis.recommendation.contains("NEUTRAL"));
    }

    #[test]
    fn test_bias_with_whipsaws() {
        let analysis = DirectionalBiasAnalysis::analyze(60, 40, 20);
        assert_eq!(analysis.up_wins_count, 60);
        assert_eq!(analysis.down_wins_count, 40);
        assert_eq!(analysis.whipsaw_count, 20);
        assert_eq!(analysis.sample_size, 120);
    }

    #[test]
    fn test_bias_empty() {
        let analysis = DirectionalBiasAnalysis::analyze(0, 0, 0);
        assert_eq!(analysis.classification, BiasType::Neutral);
        assert_eq!(analysis.asymmetry_percent, 0.0);
    }

    #[test]
    fn test_analyzer_backtests_up_biased() {
        let backtests = vec![
            ("BUY", "WIN"),
            ("BUY", "WIN"),
            ("SELL", "LOSS"),
            ("BUY", "WIN"),
            ("SELL", "WIN"),
        ];
        let result = DirectionalBiasAnalyzer::analyze_backtests(&backtests);
        assert!(result.is_ok());
        let analysis = result.expect("analyze backtests");
        assert_eq!(analysis.up_wins_count, 3);
        assert_eq!(analysis.down_wins_count, 1);
    }

    #[test]
    fn test_analyzer_backtests_empty() {
        let backtests = vec![];
        let result = DirectionalBiasAnalyzer::analyze_backtests(&backtests);
        assert!(result.is_ok());
        let analysis = result.expect("analyze backtests");
        assert_eq!(analysis.sample_size, 0);
    }
}
