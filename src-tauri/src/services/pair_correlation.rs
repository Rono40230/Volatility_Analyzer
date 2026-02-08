// services/pair_correlation.rs
// Corrélation inter-paires pour détecter les expositions doublées.
// Si deux paires corrélées > 0.7 sont tradées simultanément,
// l'exposition réelle est doublée (ex: EURUSD + GBPUSD ≈ 0.85).

use serde::{Deserialize, Serialize};

/// Corrélation connue entre deux paires (lookup statique).
/// Valeurs calibrées sur corrélations historiques moyennes 20 jours.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairCorrelationEntry {
    pub pair_a: String,
    pub pair_b: String,
    /// Corrélation de Pearson [-1.0, 1.0]
    pub correlation: f64,
}

/// Warning si deux paires corrélées sont tradées ensemble
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationWarning {
    pub pair_a: String,
    pub pair_b: String,
    pub correlation: f64,
    pub risk_level: String,
    pub message: String,
}

/// Résultat de l'analyse de corrélation inter-paires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairCorrelationResult {
    pub warnings: Vec<CorrelationWarning>,
    pub diversification_score: f64,
    pub pairs_analyzed: usize,
}

/// Table de corrélation statique entre paires Forex majeures.
/// Calibrée sur moyennes historiques 20 jours (2020-2024).
/// Source: données empiriques Forex Factory / TradingView.
const KNOWN_CORRELATIONS: &[(&str, &str, f64)] = &[
    // Haute corrélation positive (exposition doublée)
    ("EURUSD", "GBPUSD", 0.85),
    ("EURUSD", "AUDUSD", 0.70),
    ("GBPUSD", "AUDUSD", 0.65),
    ("EURJPY", "GBPJPY", 0.90),
    ("USDJPY", "EURJPY", 0.75),
    ("USDJPY", "GBPJPY", 0.80),
    // Corrélation négative (couverture naturelle)
    ("EURUSD", "USDCHF", -0.95),
    ("GBPUSD", "USDCHF", -0.85),
    ("EURUSD", "USDJPY", -0.30),
    // Indices corrélés
    ("US30", "USTEC", 0.90),
    ("GER40", "US30", 0.70),
];

/// Cherche la corrélation connue entre deux paires.
fn lookup_correlation(pair_a: &str, pair_b: &str) -> Option<f64> {
    let a = pair_a.to_uppercase();
    let b = pair_b.to_uppercase();
    for &(p1, p2, corr) in KNOWN_CORRELATIONS {
        if (a == p1 && b == p2) || (a == p2 && b == p1) {
            return Some(corr);
        }
    }
    None
}

/// Analyse un ensemble de paires pour détecter les corrélations dangereuses.
/// Seuil de warning : |corrélation| > 0.7
pub fn analyze_pair_correlations(pairs: &[String]) -> PairCorrelationResult {
    let mut warnings: Vec<CorrelationWarning> = Vec::new();
    let mut max_abs_corr = 0.0_f64;
    let mut total_abs_corr = 0.0_f64;
    let mut pair_count = 0;

    for i in 0..pairs.len() {
        for j in (i + 1)..pairs.len() {
            if let Some(corr) = lookup_correlation(&pairs[i], &pairs[j]) {
                let abs_corr = corr.abs();
                max_abs_corr = max_abs_corr.max(abs_corr);
                total_abs_corr += abs_corr;
                pair_count += 1;

                if abs_corr > 0.7 {
                    let (risk_level, message) = if abs_corr >= 0.9 {
                        ("Critique".to_string(), format!(
                            "⛔ {} et {} sont quasi-identiques (r={:.2}). Exposition doublée !",
                            pairs[i], pairs[j], corr
                        ))
                    } else if abs_corr >= 0.8 {
                        ("Élevé".to_string(), format!(
                            "🔴 {} et {} fortement corrélées (r={:.2}). Réduire la taille.",
                            pairs[i], pairs[j], corr
                        ))
                    } else {
                        ("Modéré".to_string(), format!(
                            "🟡 {} et {} corrélées (r={:.2}). Attention au risque cumulé.",
                            pairs[i], pairs[j], corr
                        ))
                    };

                    warnings.push(CorrelationWarning {
                        pair_a: pairs[i].clone(),
                        pair_b: pairs[j].clone(),
                        correlation: corr,
                        risk_level,
                        message,
                    });
                }
            }
        }
    }

    // Score de diversification : 100 = parfaitement diversifié, 0 = tout corrélé
    let avg_corr = if pair_count > 0 { total_abs_corr / pair_count as f64 } else { 0.0 };
    let diversification_score = ((1.0 - avg_corr) * 100.0).clamp(0.0, 100.0);

    PairCorrelationResult {
        warnings,
        diversification_score,
        pairs_analyzed: pairs.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_correlation_detected() {
        let pairs = vec!["EURUSD".into(), "GBPUSD".into()];
        let result = analyze_pair_correlations(&pairs);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].correlation, 0.85);
        assert!(result.diversification_score < 30.0);
    }

    #[test]
    fn test_no_correlation_between_unrelated_pairs() {
        let pairs = vec!["EURUSD".into(), "XAUUSD".into()];
        let result = analyze_pair_correlations(&pairs);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_single_pair_no_warnings() {
        let pairs = vec!["EURUSD".into()];
        let result = analyze_pair_correlations(&pairs);
        assert!(result.warnings.is_empty());
        assert_eq!(result.diversification_score, 100.0);
    }

    #[test]
    fn test_critical_correlation() {
        let pairs = vec!["EURJPY".into(), "GBPJPY".into()];
        let result = analyze_pair_correlations(&pairs);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].risk_level == "Critique");
    }

    #[test]
    fn test_multiple_pairs() {
        let pairs = vec!["EURUSD".into(), "GBPUSD".into(), "USDJPY".into()];
        let result = analyze_pair_correlations(&pairs);
        // EURUSD-GBPUSD (0.85) + EURUSD-USDJPY (-0.30 abs < 0.7)
        assert_eq!(result.warnings.len(), 1);
    }
}
