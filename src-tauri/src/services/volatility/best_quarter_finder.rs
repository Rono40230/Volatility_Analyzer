// services/volatility/best_quarter_finder.rs - Détection du meilleur quarter (15 min)
// Module séparé pour respecter la limite de taille (metrics.rs < 300L)

use crate::models::Stats15Min;

/// Détecteur du meilleur quarter pour stratégie STRADDLE
pub(super) struct BestQuarterFinder;

impl BestQuarterFinder {
    /// Trouve le meilleur quarter (15 min) pour stratégie STRADDLE (V5 - Breakout Straddle)
    ///
    /// OPTIMISATION STRADDLE (V6 - High Volatility Hunter):
    /// - Volatilité ÉLEVÉE (40% pondération) = source de profit
    /// - Range (Pips) (30% pondération) = amplitude réelle du mouvement
    /// - Noise Ratio BAS (15% pondération inverse) = entrées fiables
    /// - Body Range ÉLEVÉ (10% pondération) = mouvement directionnel
    /// - Direction Strength (5% pondération) = tendance
    ///
    /// Changement V6: Priorité absolue à l'amplitude (Pips) et Volatilité.
    /// On accepte plus de bruit (Noise) si le mouvement est puissant.
    ///
    /// Retourne (hour, quarter) du meilleur moment de la journée
    pub(super) fn find_best_quarter(stats_15min: &[Stats15Min]) -> Option<(u8, u8)> {
        const VOLATILITY_IDEAL: f64 = 2.0; // 2.0% = volatilité cible (Forex)
        const RANGE_IDEAL: f64 = 25.0; // 25 pips = référence (CORRIGÉ: Unité Pips, pas Prix)

        if stats_15min.is_empty() {
            return None;
        }

        let mut scored_quarters: Vec<((u8, u8), f64)> = stats_15min
            .iter()
            .filter(|q| q.candle_count > 0)
            .map(|q| {
                // ============================================
                // SCORE COMPOSITE STRADDLE (V6 - High Volatility)
                // ============================================

                // 1. VOLATILITÉ (40%) - PRIMARY
                let volatility_score = (q.volatility_mean / VOLATILITY_IDEAL).min(1.0) * 40.0;

                // 2. RANGE RÉEL (30%) - NEW PRIMARY
                // On veut des Pips ! Un mouvement de 50 pips "sale" vaut mieux qu'un 10 pips "propre"
                // FIX: Utilisation de max_true_range (Spike) au lieu de range_mean (Moyenne)
                // FIX: Suppression du cap .min(1.5) pour laisser les gros spikes gagner
                let range_score = (q.max_true_range / RANGE_IDEAL) * 30.0;

                // 3. NOISE RATIO INVERSE (15%) - REDUCED WEIGHT
                // On tolère plus de bruit pour plus de gain
                let noise_score = if q.noise_ratio_mean < 2.0 {
                    15.0 // Excellent
                } else if q.noise_ratio_mean < 3.0 {
                    10.0 // Bon
                } else if q.noise_ratio_mean < 4.0 {
                    5.0 // Acceptable si grosse volatilité
                } else {
                    0.0 // Trop bruyant
                };

                // 4. BODY RANGE (10%) - REDUCED WEIGHT
                let body_range_score = (q.body_range_mean / 60.0).min(1.0) * 10.0;

                // 5. DIRECTION STRENGTH (5%) - REDUCED WEIGHT
                let direction_score = (q.volume_imbalance_mean / 15.0).min(1.0) * 5.0;

                let mut total_score = volatility_score
                    + range_score
                    + noise_score
                    + body_range_score
                    + direction_score;

                // ============================================
                // BONUS / PÉNALITÉS (AJUSTÉS FOREX)
                // ============================================

                // BONUS: Volatilité excellente (> 2.0%)
                if q.volatility_mean > 2.0 {
                    total_score += 10.0;
                }

                // BONUS: Gros mouvement (> 25 pips)
                if q.max_true_range > RANGE_IDEAL {
                    total_score += 15.0;
                }

                // PÉNALITÉ: Bruit excessif (noise > 4.0) - Seuil augmenté
                if q.noise_ratio_mean > 4.0 {
                    total_score -= 15.0;
                }

                // PÉNALITÉ: Volatilité anémique (< 0.5%) - Seuil baissé pour Forex
                // FIX: Désactivé si le Spike est significatif (> 15 pips)
                if q.volatility_mean < 0.5 && q.max_true_range < 15.0 {
                    total_score -= 20.0;
                }

                ((q.hour, q.quarter), total_score)
            })
            .collect();

        // Trier par score composite décroissant
        scored_quarters.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Retourner LE meilleur quarter
        scored_quarters.first().map(|(coords, _)| *coords)
    }
}
