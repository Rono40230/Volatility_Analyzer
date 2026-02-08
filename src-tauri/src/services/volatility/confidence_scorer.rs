// services/volatility/confidence_scorer.rs - Calcul du score de confiance
// Module séparé pour respecter la limite de taille (metrics.rs < 300L)

use crate::models::GlobalMetrics;

/// Calculateur du score de confiance GLOBAL (0-100)
#[allow(clippy::doc_lazy_continuation)]
pub(super) struct ConfidenceScorer;

impl ConfidenceScorer {
    /// Calcule le score de confiance GLOBAL (0-100)
    ///
    /// PHILOSOPHIE :
    /// Ce score mesure "à quel point je peux CONFIER ma stratégie STRADDLE scalping
    /// à cette paire pendant cette période (hourly_stats agrégées)".
    ///
    /// ADAPTATION FOREX M1 :
    /// - Seuils basés sur données 2024 (EURUSD, GBPUSD, cryptos)
    /// - M1 = 1 minute → range petit mais volatilité soutenue = clé
    /// - Scalping = décisions rapides, volatilité CONSTANTE > spike isolé
    ///
    /// FORMULE (max 100 points) :
    ///
    /// 1. ATR (30 pts) - Volatilité soutenue
    ///    > 25 pips (0.00025) = 30 pts : volatilité excellente
    ///    > 15-25 pips = 25 pts
    ///    > 10-15 pips = 20 pts
    ///    > 5-10 pips = 10 pts
    ///    > → POURQUOI ATR ? Filtre les spikes isolés, mesure volatilité CONSTANTE
    ///
    /// 2. Body Range (25 pts) - Directionnalité des bougies
    ///    > 45% = 25 pts : mouvements forts, pas du bruit
    ///    > 35-45% = 20 pts
    ///    > 25-35% = 15 pts
    ///    > 15-25% = 8 pts
    ///    > → POURQUOI BodyRange ? Signal/bruit ratio, clé pour scalping
    ///
    /// 3. Volatilité % (25 pts) - BONUS si marché bouge bien
    ///    > 30% = 25 pts : crypto-like volatility
    ///    > 20-30% = 20 pts
    ///    > 10-20% = 15 pts
    ///    > 5-10% = 8 pts
    ///    > → POURQUOI ce bonus ? Scalping a BESOIN de mouvement
    ///
    /// 4. Noise Ratio (10 pts) - Ratio bruit/signal
    ///    > <2.0 = 10 pts : signal propre
    ///    > <3.0 = 7 pts
    ///    > <4.0 = 4 pts
    ///    > → POURQUOI Noise ? Élimine les false signals, confirme signal/bruit
    ///
    /// 5. Breakout % (10 pts) - % de bougies "significatives"
    ///    > 15% = 10 pts : beaucoup de vrais mouvements
    ///    > 10% = 7 pts
    ///    > 5% = 4 pts
    ///    > → POURQUOI Breakout ? Scalping veut des CASSURES, pas du sideways
    ///
    /// 6. Bonus Données (5 pts) - Si assez de candles
    ///    > 100k candles = 5 pts
    ///    > 50k = 3 pts
    ///    > → POURQUOI ? Plus de données = plus fiable
    ///
    /// INTERPRÉTATION :
    /// > - 80-100 : ✅ EXCELLENT - Scalpe agressivement
    /// > - 65-80  : 🟢 BON - Scalpe normalement
    /// > - 50-65  : 🟡 PRUDENT - Scalpe avec stop serrés
    /// > - 35-50  : 🟠 RISKY - Très prudent, breakouts only
    /// > - 0-35   : ❌ MAUVAIS - Ne pas trader
    ///
    /// EXEMPLE : EURUSD 10:00-11:00 UTC
    /// > - ATR 2.5 (Pips) → 30 pts (excellent volatilité)
    /// > - BodyRange 52% → 25 pts (très directif)
    /// > - Volatilité 0.25 → 25 pts (bonus mouvement)
    /// > - NoiseRatio 1.8 → 10 pts (signal propre)
    /// > - BreakoutPct 18% → 10 pts (beaucoup de cassures)
    /// > - Bonus → 5 pts (données suffisantes)
    /// > = TOTAL 105 → capped à 100 = "EXCELLENT, scalpe agressif"
    pub(super) fn calculer_score_confiance(metrics: &GlobalMetrics) -> f64 {
        let mut score: f64 = 0.0;

        // 1. Score ATR (30 points max) - Seuils normalisés (1.0 = 1 Pip/Point)
        // ATR Forex M1 typique : 1.0 - 3.0 pips
        if metrics.mean_atr > 2.5 {
            score += 30.0; // Excellent : > 2.5 pips/points
        } else if metrics.mean_atr > 1.5 {
            score += 25.0; // Très bon : 1.5-2.5 pips/points
        } else if metrics.mean_atr > 1.0 {
            score += 20.0; // Bon : 1.0-1.5 pips/points
        } else if metrics.mean_atr > 0.5 {
            score += 10.0; // Acceptable : 0.5-1.0 pips/points
        }

        // 2. Score Body Range (25 points max) - Seuils réalistes
        // Body Range Forex : 25-45% est normal, >45% est excellent
        if metrics.mean_body_range > 45.0 {
            score += 25.0; // Excellent : mouvements directionnels forts
        } else if metrics.mean_body_range > 35.0 {
            score += 20.0; // Très bon
        } else if metrics.mean_body_range > 25.0 {
            score += 15.0; // Bon
        } else if metrics.mean_body_range > 15.0 {
            score += 8.0; // Acceptable
        }

        // 3. Score Volatilité (25 points max) - BONUS si volatile
        // Plus c'est volatil, MIEUX c'est pour le scalping !
        if metrics.mean_volatility > 0.30 {
            score += 25.0; // Excellent : cryptos, exotiques
        } else if metrics.mean_volatility > 0.20 {
            score += 20.0; // Très bon : paires majeures volatiles
        } else if metrics.mean_volatility > 0.10 {
            score += 15.0; // Bon : volatilité correcte
        } else if metrics.mean_volatility > 0.05 {
            score += 8.0; // Acceptable
        }

        // 4. Score Noise Ratio (10 points max) - Signal/Bruit
        if metrics.mean_noise_ratio < 2.0 {
            score += 10.0; // Excellent : signal propre
        } else if metrics.mean_noise_ratio < 3.0 {
            score += 7.0; // Bon
        } else if metrics.mean_noise_ratio < 4.0 {
            score += 4.0; // Acceptable
        }

        // 5. Score Breakout % (10 points max) - CRITIQUE pour Straddle
        // % de bougies qui cassent significativement (>P80 ATR)
        if metrics.mean_breakout_percentage > 15.0 {
            score += 10.0; // Excellent : mouvements forts fréquents
        } else if metrics.mean_breakout_percentage > 10.0 {
            score += 7.0; // Très bon
        } else if metrics.mean_breakout_percentage > 5.0 {
            score += 4.0; // Acceptable
        }

        // 6. Bonus données suffisantes (5 points max)
        if metrics.total_candles > 100000 {
            score += 5.0; // Données suffisantes pour fiabilité
        } else if metrics.total_candles > 50000 {
            score += 3.0;
        }

        // 7. PÉNALITÉ: ATR élevé MAIS Noise élevé (contradiction)
        // Volatilité chaotique = mauvais pour scalping propre
        // Seuil ATR en PIPS normalisés (pas en prix brut)
        if metrics.mean_atr > 2.0 && metrics.mean_noise_ratio > 3.0 {
            score -= 15.0; // Volatilité mais signal chaotique = danger
        }

        // 8. PÉNALITÉ: BodyRange fort MAIS peu de Breakouts (indécision)
        // Bougies directionnelles mais pas de cassures = signal faible
        if metrics.mean_body_range > 40.0 && metrics.mean_breakout_percentage < 8.0 {
            score -= 10.0; // Contrainte = trading moins net
        }

        // 10. PÉNALITÉ: Trop de fausses cassures (volatilité erratique)
        // Breakout % très élevé + BodyRange faible = chaos, pas de direction
        if metrics.mean_breakout_percentage > 25.0 && metrics.mean_body_range < 30.0 {
            score -= 8.0; // Volatilité instable/chaotique = à éviter
        }

        score.clamp(0.0, 100.0) // Clamp entre 0 et 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_zero_metrics() {
        let metrics = GlobalMetrics {
            mean_atr: 0.0,
            mean_max_true_range: 0.0,
            mean_volatility: 0.0,
            mean_body_range: 0.0,
            mean_noise_ratio: 10.0,
            mean_breakout_percentage: 0.0,
            mean_volume_imbalance: 0.0,
            mean_range: 0.0,
            total_candles: 1000,
        };

        let score = ConfidenceScorer::calculer_score_confiance(&metrics);
        assert!(
            score < 20.0,
            "Mauvaises métriques doivent donner score < 20, obtenu {}",
            score
        );
    }

    #[test]
    fn test_confidence_perfect_metrics() {
        let metrics = GlobalMetrics {
            mean_atr: 3.0, // 3.0 pips (normalized)
            mean_max_true_range: 0.0,
            mean_volatility: 0.35,
            mean_body_range: 50.0,
            mean_noise_ratio: 1.5,
            mean_breakout_percentage: 20.0,
            mean_volume_imbalance: 0.05,
            mean_range: 8.0,
            total_candles: 200000,
        };

        let score = ConfidenceScorer::calculer_score_confiance(&metrics);
        assert!(
            score >= 80.0,
            "Excellentes métriques doivent donner score >= 80, obtenu {}",
            score
        );
    }

    #[test]
    fn test_confidence_bounds() {
        let test_cases = vec![
            (2.5, 0.05),
            (1.0, 0.15),
            (2.0, 0.30),
            (3.0, 0.50),
            (10.0, 0.70),
        ];

        for (atr, volatility) in test_cases {
            let metrics = GlobalMetrics {
                mean_atr: atr,
                mean_max_true_range: 0.0,
                mean_volatility: volatility,
                mean_body_range: 40.0,
                mean_noise_ratio: 2.0,
                mean_breakout_percentage: 12.0,
                mean_volume_imbalance: 0.05,
                mean_range: 0.0008,
                total_candles: 100000,
            };
            let score = ConfidenceScorer::calculer_score_confiance(&metrics);
            assert!(
                score <= 100.0,
                "Score ne doit pas dépasser 100. ATR={}, Vol={}, Score={}",
                atr,
                volatility,
                score
            );
        }
    }
}
