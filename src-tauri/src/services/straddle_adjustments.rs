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
    pub fn new(
        win_rate_percentage: f64,
        offset_optimal_pips: f64,
        whipsaw_frequency_percentage: f64,
        atr_mean: f64,
    ) -> Self {
        let whipsaw_factor = whipsaw_frequency_percentage / 100.0;

        let win_rate_adjusted = win_rate_percentage * (1.0 - whipsaw_factor);

        // === FORMULE SL CORRIGÉE ===
        // Logique: Plus whipsaw est élevé, plus le SL doit être RÉDUIT (moins d'espace)
        // Plus whipsaw est bas, plus le SL peut être LARGE (plus d'espace pour capturer mouvements)
        let whipsaw_adjusted_ratio = match whipsaw_factor {
            w if w > 0.30 => 1.5, // Whipsaw 30%+ → ratio 1.5× (trop de faux mouvements)
            w if w > 0.20 => 1.8, // Whipsaw 20-30% → ratio 1.8× (équilibre)
            w if w > 0.10 => 2.2, // Whipsaw 10-20% → ratio 2.2× (augmente SL)
            w if w > 0.05 => 2.5, // Whipsaw 5-10% → ratio 2.5× (SL large)
            _ => 2.8,             // Whipsaw <5% → ratio 2.8× (SL très large, peu de whipsaws)
        };
        // Arrondir à l'unité supérieure (pas de décimales pour les pips)
        let sl_adjusted_pips = (offset_optimal_pips * whipsaw_adjusted_ratio).ceil();

        let trailing_stop_brut = 1.59;
        let trailing_stop_adjusted = trailing_stop_brut * (1.0 - whipsaw_factor / 2.0);

        // === TIMEOUT BASÉ SUR LA VOLATILITÉ (ATR) ===
        // Normaliser l'ATR sur une échelle 0.0 - 1.0 (basé sur percentiles typiques)
        // ATR faible typique: 0.0001-0.0003 (Forex)
        // ATR élevée typique: 0.0005-0.0010 (Forex)
        let atr_normalized = (atr_mean / 0.0008).min(1.0); // Normaliser avec 0.0008 comme référence

        // Timeout inversement proportionnel à la volatilité:
        // - Volatilité basse (ATR faible) → timeout long (32 min)
        // - Volatilité haute (ATR élevée) → timeout court (18 min)
        let timeout_base = 32.0;
        let timeout_min = 18.0;
        let timeout_adjusted_minutes =
            (timeout_base - (atr_normalized * (timeout_base - timeout_min))) as i32;

        AdjustedMetrics {
            win_rate_adjusted,
            sl_adjusted_pips,
            trailing_stop_adjusted,
            timeout_adjusted_minutes,
        }
    }
}
