// services/volatility/best_quarter_finder.rs - Détection du meilleur quarter (15 min)
// Module séparé pour respecter la limite de taille (metrics.rs < 300L)

use crate::models::Stats15Min;

/// Détecteur du meilleur quarter pour stratégie STRADDLE
pub(super) struct BestQuarterFinder;

impl BestQuarterFinder {
    /// Trouve le meilleur quarter (15 min) pour stratégie STRADDLE (V4 - Optimisée Straddle)
    ///
    /// OPTIMISATION STRADDLE (V4) - Non-directionnel, profit sur volatilité bidirectionnelle:
    /// - Volatilité ÉLEVÉE (50% pondération) = source de profit
    /// - Noise Ratio BAS (20% pondération inverse) = entrées fiables, pas de rejets
    /// - Body Range BAS (15% pondération inverse) = marché indécis, volatilité aléatoire (pas directionnel)
    /// - Direction Strength BAS (10% pondération inverse) = pas de tendance préférentielle
    /// - Range (5% pondération) = mouvement exploitable
    ///
    /// Le Straddle diffère du Scalping:
    /// - Straddle = veut volatilité SANS direction (bidirectionnelle)
    /// - Scalping = veut direction CLAIRE avec peu de bruit
    ///
    /// Retourne (hour, quarter) du meilleur moment de la journée
    pub(super) fn find_best_quarter(stats_15min: &[Stats15Min]) -> Option<(u8, u8)> {
        const VOLATILITY_IDEAL: f64 = 50.0; // 50% = volatilité cible pour Straddle
        const RANGE_IDEAL: f64 = 0.0025; // 25 pips = référence
        const NOISE_IDEAL: f64 = 2.0; // < 2.0 = signal propre

        if stats_15min.is_empty() {
            return None;
        }

        let mut scored_quarters: Vec<((u8, u8), f64)> = stats_15min
            .iter()
            .filter(|q| q.candle_count > 0)
            .map(|q| {
                // ============================================
                // SCORE COMPOSITE STRADDLE (V4)
                // ============================================
                
                // 1. VOLATILITÉ (50%) - PRIMARY
                // Straddle profite de volatilité élevée, peu importe la direction
                let volatility_score = (q.volatility_mean / VOLATILITY_IDEAL).min(1.0) * 50.0;
                
                // 2. NOISE RATIO INVERSE (20%) - LOWER IS BETTER
                // Bruit bas = entrées fiables, moins de fausses mèches
                // Si noise > 3.5 : très mauvais (rejets constants)
                // Si noise < 2.0 : excellent (signal propre)
                let noise_score = if q.noise_ratio_mean < NOISE_IDEAL {
                    // Très bon : +20 pts si bruit < 2.0
                    20.0
                } else if q.noise_ratio_mean < 2.5 {
                    // Bon : +15 pts si bruit 2.0-2.5
                    15.0
                } else if q.noise_ratio_mean < 3.0 {
                    // Acceptable : +10 pts
                    10.0
                } else if q.noise_ratio_mean < 3.5 {
                    // Mauvais : +5 pts
                    5.0
                } else {
                    // Très mauvais : 0 pts, pénalité -10
                    0.0
                };
                
                // 3. BODY RANGE INVERSE (15%) - LOWER IS BETTER FOR STRADDLE
                // Body Range ÉLEVÉ = marché directionnel (mauvais pour Straddle)
                // Body Range BAS (30-40%) = marché indécis = volatilité aléatoire (idéal)
                // Formule inverse : plus le body range est bas, meilleur le score
                let body_range_inverse = (100.0 - q.body_range_mean).max(0.0); // 0-70
                let body_range_score = (body_range_inverse / 60.0).min(1.0) * 15.0; // Max 15 pts
                
                // 4. DIRECTION STRENGTH INVERSE (10%) - LOWER IS BETTER
                // Direction Strength ÉLEVÉE = tendance claire (mauvais pour Straddle)
                // Direction Strength BASSE = mouvements aléatoires (idéal)
                // Formule inverse : si direction_strength < 10%, score max
                let direction_inverse = (20.0 - q.volume_imbalance_mean).max(0.0);
                let direction_score = (direction_inverse / 20.0).min(1.0) * 10.0; // Max 10 pts
                
                // 5. RANGE (5%) - SECONDARY
                let range_score = (q.range_mean / RANGE_IDEAL).min(1.0) * 5.0;
                
                let mut total_score = volatility_score + noise_score + body_range_score + direction_score + range_score;
                
                // ============================================
                // BONUS / PÉNALITÉS
                // ============================================
                
                // BONUS: Volatilité excellente (> 50%)
                if q.volatility_mean > 50.0 {
                    total_score += 15.0;
                }
                
                // BONUS: Volatilité bidirectionnelle idéale (35-40% body range = indécision)
                if q.body_range_mean > 35.0 && q.body_range_mean < 45.0 {
                    total_score += 10.0; // Marché vraiment indécis
                }
                
                // BONUS: Signal ultra-propre (noise < 2.0 AND body range équilibré)
                if q.noise_ratio_mean < 2.0 && q.body_range_mean < 45.0 {
                    total_score += 12.0;
                }
                
                // PÉNALITÉ MAJEURE: Tendance directionnelle très forte (body range > 55%)
                // Mauvais pour Straddle (profit réduit ou risque de perte)
                if q.body_range_mean > 55.0 {
                    total_score -= 25.0;
                }
                
                // PÉNALITÉ: Bruit excessif (noise > 3.5)
                if q.noise_ratio_mean > 3.5 {
                    total_score -= 15.0;
                }
                
                // PÉNALITÉ: Direction Strength très élevée (> 17%)
                // Indica que le marché a une préférence directionnelle forte
                if q.volume_imbalance_mean > 17.0 {
                    total_score -= 12.0;
                }
                
                // PÉNALITÉ: Volatilité trop basse (< 25% = peu de profit)
                if q.volatility_mean < 25.0 {
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
