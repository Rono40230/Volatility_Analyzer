// services/straddle_adjustments.rs
// Calculs des valeurs pondérées par le whipsaw

/// Calcule les 4 valeurs ajustées selon la fréquence whipsaw
pub struct AdjustedMetrics {
    pub win_rate_adjusted: f64,        // Win Rate pondéré
    pub sl_adjusted_pips: f64,         // SL ajusté
    pub trailing_stop_adjusted: f64,   // Trailing Stop ajusté
    pub timeout_adjusted_minutes: i32, // Timeout ajusté
}

impl AdjustedMetrics {
    /// Calcule un timeout ajusté basé sur l’ATR (volatilité)
    /// Range harmonisé [10, 30] min (cohérent avec straddle_parameter_service)
    /// ATR élevé → timeout court (volatilité décline vite)
    /// ATR faible → timeout long (besoin de plus de temps)
    pub fn calculer_timeout_from_atr(atr_mean: f64) -> i32 {
        // 8.0 pips = seuil de haute volatilité M15 (percentile ~90 sur Forex majeurs)
        // Au-delà, le timeout est au minimum (10 min) car la vol décline vite
        let atr_normalized = (atr_mean / 8.0).min(1.0);
        let timeout_base = 30.0;
        let timeout_min = 10.0;
        let timeout = timeout_base - (atr_normalized * (timeout_base - timeout_min));
        (timeout as i32).clamp(10, 30)
    }

    /// Calcule tous les ajustements basés sur la fréquence whipsaw et la volatilité (ATR)
    ///
    /// Formules:
    /// 1. Win Rate ajusté = WR brut × (1 - whipsaw_frequency)
    /// 2. SL ajusté = SL brut × Ratio(whipsaw_frequency)
    ///    - Whipsaw 30%+ → ratio 1.5× (peu d'espace, beaucoup de faux mouvements)
    ///    - Whipsaw 20-30% → ratio 1.8× (équilibre)
    ///    - Whipsaw 10-20% → ratio 2.2× (plus d'espace)
    ///    - Whipsaw 5-10% → ratio 2.5× (SL large)
    ///    - Whipsaw <5% → ratio 2.8× (SL très large, peu de whipsaws)
    /// 3. Trailing Stop ajusté = 1.59 × (1 - whipsaw_frequency / 2)
    /// 4. Timeout ajusté = Basé sur ATR (volatilité)
    ///    - ATR élevé = timeout court (18 min) - volatilité décline vite
    ///    - ATR faible = timeout long (32 min) - volatilité décline lentement
    #[allow(dead_code)]
    pub fn new(
        win_rate_percentage: f64,
        offset_optimal_pips: f64,
        whipsaw_frequency_percentage: f64,
        atr_mean: f64,
    ) -> Self {
        let whipsaw_factor = whipsaw_frequency_percentage / 100.0;

        let win_rate_adjusted = win_rate_percentage * (1.0 - whipsaw_factor);

        // === FORMULE SL CORRIGÉE ===
        // Logique: Plus whipsaw est élevé, plus le SL doit être AUGMENTÉ (plus d'espace contre le bruit)
        // Plus whipsaw est bas, plus le SL peut être RÉDUIT (conditions propres, peu de faux mouvements)
        let whipsaw_adjusted_ratio = match whipsaw_factor {
            w if w > 0.50 => 3.5, // Whipsaw 50%+ → ratio 3.5× (énorme SL)
            w if w > 0.30 => 3.0, // Whipsaw 30-50% → ratio 3.0×
            w if w > 0.20 => 2.5, // Whipsaw 20-30% → ratio 2.5×
            w if w > 0.10 => 2.0, // Whipsaw 10-20% → ratio 2.0×
            w if w > 0.05 => 1.5, // Whipsaw 5-10% → ratio 1.5×
            _ => 1.2,             // Whipsaw <5% → ratio 1.2× (petit SL)
        };
        // Arrondir à l'unité supérieure (pas de décimales pour les pips)
        let sl_adjusted_pips = (offset_optimal_pips * whipsaw_adjusted_ratio).ceil();

        // Calibré empiriquement : 1.59 × ATR capture ~95% des retracements
        // sans couper trop tôt. Origine : backtests sur EURUSD/GBPUSD M1 (2022-2024)
        let trailing_stop_brut = 1.59;
        let trailing_stop_adjusted = trailing_stop_brut * (1.0 - whipsaw_factor / 2.0);

        // === TIMEOUT BASÉ SUR LA VOLATILITÉ (ATR) ===
        // Normaliser l'ATR sur une échelle 0.0 - 1.0 (basé sur percentiles typiques)
        // ATR faible typique: 1.0-3.0 pips (Forex)
        // ATR élevée typique: 5.0-10.0 pips (Forex)
        let timeout_adjusted_minutes = Self::calculer_timeout_from_atr(atr_mean);

        AdjustedMetrics {
            win_rate_adjusted,
            sl_adjusted_pips,
            trailing_stop_adjusted,
            timeout_adjusted_minutes,
        }
    }

    /// Calcule les valeurs BRUTES sans pondération whipsaw
    /// Whipsaw est désactivé - retourne les valeurs non modifiées
    pub fn new_with_pair(
        win_rate_percentage: f64,
        offset_optimal_pips: f64,
        _whipsaw_frequency_percentage: f64,
        atr_mean: f64,
        _symbol: &str,
    ) -> Self {
        // Whipsaw désactivé = pas de pondération
        // Retourner le win rate brut (sans ajustement whipsaw)

        let win_rate_adjusted = win_rate_percentage; // Win rate brut non ajusté

        // SL brut = offset sans pondération
        let sl_adjusted_pips = offset_optimal_pips;

        // TS brut = coefficient fixe (sans whipsaw)
        // Retourner coefficient 1.0 pour que TS = SL × 1.0
        let trailing_stop_adjusted = 1.0;

        // Timeout basé sur ATR (indépendant du whipsaw)
        let timeout_adjusted_minutes = Self::calculer_timeout_from_atr(atr_mean);

        AdjustedMetrics {
            win_rate_adjusted,
            sl_adjusted_pips,
            trailing_stop_adjusted,
            timeout_adjusted_minutes,
        }
    }
}
