// services/volatility/best_hours_finder.rs - Détection des meilleures heures
// Module séparé pour respecter la limite de taille (metrics.rs < 300L)

use crate::models::HourlyStats;

/// Détecteur des meilleures heures pour stratégie STRADDLE
pub(super) struct BestHoursFinder;

impl BestHoursFinder {
    /// Trouve les meilleures heures pour stratégie STRADDLE scalping (V2)
    ///
    /// AMÉLIORATION V2 :
    /// - Score composite multi-dimensionnel (pas juste range > 25pips)
    /// - Bonus signal propre (Noise faible)
    /// - Pénalité volatilité chaotique (Breakout % élevé + BodyRange faible)
    /// - Détecte heures stables MÊME avec range < 25pips
    ///
    /// Logique :
    /// 1. Calculer score composite pour chaque heure
    ///    - Range (60% pondération)
    ///    - ATR (25%)
    ///    - BodyRange (15%)
    ///    - Bonus Noise faible (-2 pts)
    ///    - Pénalité chaos (Breakout élevé + BodyRange faible)
    /// 2. Retourner top 6 heures avec score le plus élevé
    pub(super) fn find_best_hours(hourly_stats: &[HourlyStats]) -> Vec<u8> {
        const MAX_HOURS: usize = 6;
        const RANGE_IDEAL: f64 = 0.0025; // 25 pips = référence 100%
        const ATR_IDEAL: f64 = 0.0020;
        const BODYRANGE_IDEAL: f64 = 40.0; // 40% = référence 100%

        let mut scored_hours: Vec<(u8, f64)> = hourly_stats
            .iter()
            .filter(|h| h.candle_count > 0)
            .map(|h| {
                // Score composite : Range (60%) + ATR (25%) + BodyRange (15%)
                let range_score = (h.range_mean / RANGE_IDEAL).min(1.0) * 60.0;
                let atr_score = (h.atr_mean / ATR_IDEAL).min(1.0) * 25.0;
                let body_score = (h.body_range_mean / BODYRANGE_IDEAL).min(1.0) * 15.0;

                let mut total_score = range_score + atr_score + body_score;

                // BONUS: Signal propre (Noise faible = scalping propre)
                if h.noise_ratio_mean < 2.0 {
                    total_score += 10.0;
                } else if h.noise_ratio_mean < 2.5 {
                    total_score += 5.0;
                }

                // PÉNALITÉ: Volatilité chaotique (Breakout % élevé + BodyRange faible)
                // = fausses cassures = danger pour scalping
                if h.breakout_percentage > 20.0 && h.body_range_mean < 25.0 {
                    total_score -= 15.0;
                }

                // PÉNALITÉ: BodyRange fort MAIS peu de Breakouts (indécision)
                if h.body_range_mean > 40.0 && h.breakout_percentage < 8.0 {
                    total_score -= 8.0;
                }

                (h.hour, total_score)
            })
            .collect();

        // Trier par score composite décroissant
        scored_hours.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Retourner les meilleures heures (max 6)
        scored_hours
            .iter()
            .take(MAX_HOURS)
            .map(|(hour, _)| *hour)
            .collect()
    }
}
