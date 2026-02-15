// services/volatility/confidence_scorer.rs - Calcul du score de confiance
// Module séparé pour respecter la limite de taille (metrics.rs < 300L)

use crate::models::{GlobalMetrics, ConfidenceBreakdown};

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
    pub(super) fn calculer_score_confiance_breakdown(metrics: &GlobalMetrics) -> ConfidenceBreakdown {
        let mut breakdown = ConfidenceBreakdown::new();

        // 1. Score ATR (30 points max)
        breakdown.atr_score = metrics.mean_atr;
        if metrics.mean_atr > 2.5 {
            breakdown.atr_points = 30.0;
            breakdown.atr_reasoning = format!("Excellent: {:.2} pips (> 2.5)", metrics.mean_atr);
        } else if metrics.mean_atr > 1.5 {
            breakdown.atr_points = 25.0;
            breakdown.atr_reasoning = format!("Très bon: {:.2} pips (1.5-2.5)", metrics.mean_atr);
        } else if metrics.mean_atr > 1.0 {
            breakdown.atr_points = 20.0;
            breakdown.atr_reasoning = format!("Bon: {:.2} pips (1.0-1.5)", metrics.mean_atr);
        } else if metrics.mean_atr > 0.5 {
            breakdown.atr_points = 10.0;
            breakdown.atr_reasoning = format!("Acceptable: {:.2} pips (0.5-1.0)", metrics.mean_atr);
        } else {
            breakdown.atr_reasoning = format!("Faible: {:.2} pips (< 0.5)", metrics.mean_atr);
        }

        // 2. Score Body Range (25 points max)
        breakdown.body_range_score = metrics.mean_body_range;
        if metrics.mean_body_range > 45.0 {
            breakdown.body_range_points = 25.0;
            breakdown.body_range_reasoning = format!("Excellent: {:.1}% (> 45%)", metrics.mean_body_range);
        } else if metrics.mean_body_range > 35.0 {
            breakdown.body_range_points = 20.0;
            breakdown.body_range_reasoning = format!("Très bon: {:.1}% (35-45%)", metrics.mean_body_range);
        } else if metrics.mean_body_range > 25.0 {
            breakdown.body_range_points = 15.0;
            breakdown.body_range_reasoning = format!("Bon: {:.1}% (25-35%)", metrics.mean_body_range);
        } else if metrics.mean_body_range > 15.0 {
            breakdown.body_range_points = 8.0;
            breakdown.body_range_reasoning = format!("Acceptable: {:.1}% (15-25%)", metrics.mean_body_range);
        } else {
            breakdown.body_range_reasoning = format!("Faible: {:.1}% (< 15%)", metrics.mean_body_range);
        }

        // 3. Score Volatility (25 points max)
        breakdown.volatility_score = metrics.mean_volatility;
        if metrics.mean_volatility > 30.0 {
            breakdown.volatility_points = 25.0;
            breakdown.volatility_reasoning = format!("Excellent: {:.1}% (> 30%)", metrics.mean_volatility);
        } else if metrics.mean_volatility > 20.0 {
            breakdown.volatility_points = 20.0;
            breakdown.volatility_reasoning = format!("Très bon: {:.1}% (20-30%)", metrics.mean_volatility);
        } else if metrics.mean_volatility > 10.0 {
            breakdown.volatility_points = 15.0;
            breakdown.volatility_reasoning = format!("Bon: {:.1}% (10-20%)", metrics.mean_volatility);
        } else if metrics.mean_volatility > 5.0 {
            breakdown.volatility_points = 8.0;
            breakdown.volatility_reasoning = format!("Acceptable: {:.1}% (5-10%)", metrics.mean_volatility);
        } else {
            breakdown.volatility_reasoning = format!("Faible: {:.1}% (< 5%)", metrics.mean_volatility);
        }

        // 4. Score Noise Ratio (10 points max)
        breakdown.noise_ratio_score = metrics.mean_noise_ratio;
        if metrics.mean_noise_ratio < 2.0 {
            breakdown.noise_ratio_points = 10.0;
            breakdown.noise_ratio_reasoning = format!("Excellent: {:.2}x (< 2.0)", metrics.mean_noise_ratio);
        } else if metrics.mean_noise_ratio < 3.0 {
            breakdown.noise_ratio_points = 7.0;
            breakdown.noise_ratio_reasoning = format!("Bon: {:.2}x (2.0-3.0)", metrics.mean_noise_ratio);
        } else if metrics.mean_noise_ratio < 4.0 {
            breakdown.noise_ratio_points = 4.0;
            breakdown.noise_ratio_reasoning = format!("Acceptable: {:.2}x (3.0-4.0)", metrics.mean_noise_ratio);
        } else {
            breakdown.noise_ratio_reasoning = format!("Mauvais: {:.2}x (> 4.0)", metrics.mean_noise_ratio);
        }

        // 5. Score Breakout % (10 points max)
        breakdown.breakout_score = metrics.mean_breakout_percentage;
        if metrics.mean_breakout_percentage > 15.0 {
            breakdown.breakout_points = 10.0;
            breakdown.breakout_reasoning = format!("Excellent: {:.1}% (> 15%)", metrics.mean_breakout_percentage);
        } else if metrics.mean_breakout_percentage > 10.0 {
            breakdown.breakout_points = 7.0;
            breakdown.breakout_reasoning = format!("Très bon: {:.1}% (10-15%)", metrics.mean_breakout_percentage);
        } else if metrics.mean_breakout_percentage > 5.0 {
            breakdown.breakout_points = 4.0;
            breakdown.breakout_reasoning = format!("Acceptable: {:.1}% (5-10%)", metrics.mean_breakout_percentage);
        } else {
            breakdown.breakout_reasoning = format!("Faible: {:.1}% (< 5%)", metrics.mean_breakout_percentage);
        }

        // 6. Bonus données
        if metrics.total_candles > 100000 {
            breakdown.bonus_points = 5.0;
            breakdown.bonus_reasoning = format!("Données excellentes: {:.0}k candles", metrics.total_candles as f64 / 1000.0);
        } else if metrics.total_candles > 50000 {
            breakdown.bonus_points = 3.0;
            breakdown.bonus_reasoning = format!("Données bonnes: {:.0}k candles", metrics.total_candles as f64 / 1000.0);
        } else {
            breakdown.bonus_reasoning = format!("Données limitées: {:.0}k candles", metrics.total_candles as f64 / 1000.0);
        }

        // 7. Pénalités
        if metrics.mean_atr > 2.0 && metrics.mean_noise_ratio > 3.0 {
            breakdown.add_penalty(-15.0, format!(
                "Chaos: ATR {:.2} pips + Noise {:.2}x (volatilité chaotique)",
                metrics.mean_atr, metrics.mean_noise_ratio
            ));
        }

        if metrics.mean_body_range > 40.0 && metrics.mean_breakout_percentage < 8.0 {
            breakdown.add_penalty(-10.0, format!(
                "Indécision: Body Range {:.1}% mais Breakout {:.1}% (bougies directionnelles sans cassures)",
                metrics.mean_body_range, metrics.mean_breakout_percentage
            ));
        }

        if metrics.mean_breakout_percentage > 25.0 && metrics.mean_body_range < 30.0 {
            breakdown.add_penalty(-8.0, format!(
                "Volatilité instable: Breakout {:.1}% mais Body Range {:.1}% (chaos)",
                metrics.mean_breakout_percentage, metrics.mean_body_range
            ));
        }

        breakdown.finalize();
        breakdown
    }

    pub(super) fn calculer_score_confiance(metrics: &GlobalMetrics) -> f64 {
        Self::calculer_score_confiance_breakdown(metrics).total
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
            mean_atr: 3.0,
            mean_max_true_range: 0.0,
            mean_volatility: 35.0,
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
    fn test_breakdown_has_reasoning() {
        let metrics = GlobalMetrics {
            mean_atr: 2.5,
            mean_max_true_range: 5.0,
            mean_volatility: 25.0,
            mean_body_range: 48.0,
            mean_noise_ratio: 1.8,
            mean_breakout_percentage: 18.0,
            mean_volume_imbalance: 0.05,
            mean_range: 3.5,
            total_candles: 150000,
        };

        let bd = ConfidenceScorer::calculer_score_confiance_breakdown(&metrics);
        assert!(!bd.atr_reasoning.is_empty());
        assert!(!bd.body_range_reasoning.is_empty());
        assert!(!bd.volatility_reasoning.is_empty());
        assert!(!bd.noise_ratio_reasoning.is_empty());
        assert!(!bd.breakout_reasoning.is_empty());
        println!("{}", bd.report());
    }
}
