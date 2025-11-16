// services/volatility/best_hours_finder.rs - Détection des meilleures heures
// Module séparé pour respecter la limite de taille (metrics.rs < 300L)

use crate::models::HourlyStats;

/// Détecteur des meilleures heures pour stratégie STRADDLE
pub(super) struct BestHoursFinder;

impl BestHoursFinder {
    /// Trouve les meilleures heures pour stratégie STRADDLE scalping
    ///
    /// FILTRAGE :
    /// - Retourne TOUTES les heures avec range > 25 pips (seuil straddle)
    /// - Triées par mouvement_potential_score_straddle() décroissant
    /// - Maximum 6 heures (pour couvrir la journée sans surcharger)
    ///
    /// Logique :
    /// 1. Calculer score straddle pour chaque heure
    /// 2. Filtrer : range_mean > 0.0025 (25 pips minimum)
    /// 3. Trier par score décroissant
    /// 4. Retourner top 6 heures qualifiées
    pub(super) fn find_best_hours(hourly_stats: &[HourlyStats]) -> Vec<u8> {
        const STRADDLE_RANGE_THRESHOLD: f64 = 0.0025; // 25 pips minimum
        const MAX_HOURS: usize = 6; // Max heures par jour

        let mut scored_hours: Vec<(u8, f64)> = hourly_stats
            .iter()
            .filter(|h| h.candle_count > 0 && h.range_mean > STRADDLE_RANGE_THRESHOLD)
            .map(|h| (h.hour, h.movement_potential_score_straddle()))
            .collect();

        // Trie par score straddle décroissant
        scored_hours.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Retourne les meilleures heures qualifiées (max 6)
        scored_hours
            .iter()
            .take(MAX_HOURS)
            .map(|(hour, _)| *hour)
            .collect()
    }
}
