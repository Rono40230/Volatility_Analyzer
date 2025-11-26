// services/volatility/best_quarter_finder.rs - Détection du meilleur quarter (15 min)
// Module séparé pour respecter la limite de taille (metrics.rs < 300L)

use crate::models::Stats15Min;

/// Détecteur du meilleur quarter pour stratégie STRADDLE
pub(super) struct BestQuarterFinder;

impl BestQuarterFinder {
    /// Trouve le meilleur quarter (15 min) pour stratégie STRADDLE (V3)
    ///
    /// OPTIMISATION STRADDLE (V3) :
    /// - Priorité ABSOLUE à la Volatilité (40% pondération)
    /// - Range + ATR secondaires (30% + 20%)
    /// - Direction Strength (10%)
    /// - Pénalité pour Whipsaw (volatilité chaotique)
    /// - Straddle cherche VOLATILITÉ ÉLEVÉE, granularité 15 min
    ///
    /// Retourne (hour, quarter) du meilleur moment de la journée
    pub(super) fn find_best_quarter(stats_15min: &[Stats15Min]) -> Option<(u8, u8)> {
        const RANGE_IDEAL: f64 = 0.0025; // 25 pips = référence 100%
        const ATR_IDEAL: f64 = 0.0020;
        const VOLATILITY_IDEAL: f64 = 25.0; // 25% = référence 100% (en pourcentage)
        const DIRECTION_STRENGTH_IDEAL: f64 = 20.0; // 20% = référence 100% (en pourcentage)

        if stats_15min.is_empty() {
            return None;
        }

        let mut scored_quarters: Vec<((u8, u8), f64)> = stats_15min
            .iter()
            .filter(|q| q.candle_count > 0)
            .map(|q| {
                // Score composite STRADDLE : Volatilité (40%) + Range (30%) + ATR (20%) + Direction (10%)
                // NOTE: volatility_mean et volume_imbalance_mean sont en format pourcentage (0-100)
                let volatility_score = (q.volatility_mean / VOLATILITY_IDEAL).min(1.0) * 40.0;
                let range_score = (q.range_mean / RANGE_IDEAL).min(1.0) * 30.0;
                let atr_score = (q.atr_mean / ATR_IDEAL).min(1.0) * 20.0;
                let direction_score = (q.volume_imbalance_mean / DIRECTION_STRENGTH_IDEAL).min(1.0) * 10.0;

                let mut total_score = volatility_score + range_score + atr_score + direction_score;

                // BONUS: Volatilité excellente (> 25% = parfait pour straddle)
                if q.volatility_mean > 25.0 {
                    total_score += 20.0;
                } else if q.volatility_mean > 20.0 {
                    // Volatilité très bonne (20-25%)
                    total_score += 10.0;
                }

                // PÉNALITÉ MAJEURE: Volatilité trop faible (< 15% = trop calme pour straddle)
                // Straddle ne profite PAS d'un marché endormi
                if q.volatility_mean < 15.0 {
                    total_score -= 30.0; // Pénalité sévère
                }

                // PÉNALITÉ: Volatilité chaotique (Breakout % très élevé + BodyRange faible)
                // = fausses cassures = Whipsaw élevé = danger pour straddle
                if q.breakout_percentage > 20.0 && q.body_range_mean < 25.0 {
                    total_score -= 12.0;
                }

                // PÉNALITÉ: Noise ratio très élevé (> 3.5 = trop de rejet)
                if q.noise_ratio_mean > 3.5 {
                    total_score -= 10.0;
                }

                // BONUS: Signal propre avec haute volatilité (idéal straddle)
                if q.noise_ratio_mean < 2.0 && q.volatility_mean > 20.0 {
                    total_score += 15.0;
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
