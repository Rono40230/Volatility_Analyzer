//! Seuils de scoring pour HourlyStats.
//! Les seuils ATR/Range sont des références Forex Major.
//! Utiliser `scaled_atr_*` pour adapter à d'autres classes d'actifs.

/// ATR adapté Forex (Pips) (30 pts) — référence ForexMajor
pub const ATR_EXCELLENT: f64 = 10.0;
pub const ATR_GOOD: f64 = 7.0;
pub const ATR_FAIR: f64 = 4.0;
pub const ATR_POOR: f64 = 2.0;

/// Body Range réaliste (25 pts) — universel (en %)
pub const BODY_RANGE_EXCELLENT: f64 = 45.0;
pub const BODY_RANGE_GOOD: f64 = 35.0;
pub const BODY_RANGE_FAIR: f64 = 25.0;
pub const BODY_RANGE_POOR: f64 = 15.0;

/// Volatilité (20 pts) — universel (en %)
pub const VOL_EXCELLENT: f64 = 0.30;
pub const VOL_GOOD: f64 = 0.20;
pub const VOL_FAIR: f64 = 0.10;
pub const VOL_POOR: f64 = 0.05;

/// Noise Ratio (15 pts) — universel (sans unité)
pub const NOISE_EXCELLENT: f64 = 2.0;
pub const NOISE_GOOD: f64 = 3.0;
pub const NOISE_FAIR: f64 = 4.0;

/// Breakout % (10 pts) — universel (en %)
pub const BREAKOUT_EXCELLENT: f64 = 15.0;
pub const BREAKOUT_GOOD: f64 = 10.0;
pub const BREAKOUT_FAIR: f64 = 5.0;

/// Seuils ATR adaptés par classe d'actif via `atr_scaling_factor`.
pub fn scaled_atr_excellent(factor: f64) -> f64 { ATR_EXCELLENT * factor }
pub fn scaled_atr_good(factor: f64) -> f64 { ATR_GOOD * factor }
pub fn scaled_atr_fair(factor: f64) -> f64 { ATR_FAIR * factor }
pub fn scaled_atr_poor(factor: f64) -> f64 { ATR_POOR * factor }
