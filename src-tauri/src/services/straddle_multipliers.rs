//! Straddle SL Multiplicators Service
//!
//! Defines pair-specific Stop Loss multiplicators based on volatility profiles
//! and time-of-day adjustments.

use std::collections::HashMap;

/// Time-of-day volatility zones
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeZone {
    /// Critical trading hours: Europe open, US open, Asia close
    /// Volatility +50%
    Critical,
    /// Calm hours: Night NY, European lull, US lunch
    /// Volatility -30%
    Calm,
    /// Normal trading hours
    /// Volatility baseline (1.0)
    Normal,
}

/// Calculate base SL from ATR using pair-specific multiplicator
///
/// This is the PRIMARY calculation that determines the width of the SL.
/// The result will then be adjusted by whipsaw frequency and time zones.
///
/// # Arguments
/// * `atr` - ATR value in points/pips
/// * `pair` - Trading pair (e.g., "EURUSD", "BTCUSD")
/// * `is_event_time` - True if economic event is happening
/// * `volatility_percentile` - Optional ATR percentile (0-100)
///
/// # Returns
/// Base SL value in points/pips before time-of-day adjustment
#[allow(dead_code)]
pub fn calculate_sl_base(
    atr: f64,
    pair: &str,
    is_event_time: bool,
    volatility_percentile: Option<f64>,
) -> f64 {
    let mul = get_sl_multiplier(pair, is_event_time, volatility_percentile);
    let sl_base = atr * mul;

    tracing::debug!(
        "SL base calculation: {} ATR {:.0} × MUL {:.2} = {:.0}",
        pair,
        atr,
        mul,
        sl_base
    );

    sl_base
}

/// Get time zone based on UTC hour
#[allow(dead_code)]
pub fn get_time_zone(hour_utc: u32) -> TimeZone {
    match hour_utc {
        // CRITICAL HOURS (Vol +50%)
        8..=9 => TimeZone::Critical,   // Europe open
        12..=14 => TimeZone::Critical, // US open
        16..=17 => TimeZone::Critical, // Asia close/BoJ

        // CALM HOURS (Vol -30%)
        2..=7 => TimeZone::Calm, // Night NY
        10 => TimeZone::Calm,    // European lull (10:00-10:30)
        15 => TimeZone::Calm,    // US lunch

        // NORMAL
        _ => TimeZone::Normal,
    }
}

/// Get volatility multiplier for time zone
#[allow(dead_code)]
pub fn get_time_multiplier(zone: TimeZone) -> f64 {
    match zone {
        TimeZone::Critical => 1.5,
        TimeZone::Calm => 0.7,
        TimeZone::Normal => 1.0,
    }
}

/// SL Multiplier tuple: (default, high_volatility, event_time)
#[allow(dead_code)]
type MulTuple = (f64, f64, f64);

/// Initialize SL multiplicators for all supported pairs
#[allow(dead_code)]
fn init_multipliers() -> HashMap<String, MulTuple> {
    let mut m = HashMap::new();

    // FOREX MAJEURS
    m.insert("EURUSD".to_string(), (1.40, 1.50, 1.50)); // Stable majeur
    m.insert("USDJPY".to_string(), (1.45, 1.60, 1.60)); // Ultra-stable BoJ
    m.insert("USDCAD".to_string(), (1.55, 1.70, 1.70)); // Oil volatility
    m.insert("GBPUSD".to_string(), (1.70, 1.90, 1.90)); // BoE volatile

    // FOREX CROISÉES
    m.insert("EURJPY".to_string(), (1.80, 2.00, 2.00)); // Cross illiquid
    m.insert("CADJPY".to_string(), (1.80, 2.00, 2.00)); // Oil + JPY combo

    // COMMODITÉS
    m.insert("XAUUSD".to_string(), (3.50, 4.00, 4.00)); // Or safe-haven
    m.insert("XAGUSD".to_string(), (4.00, 4.50, 4.50)); // Argent extrême
    m.insert("XAUJPY".to_string(), (4.20, 4.80, 4.80)); // Or×JPY double safe

    // INDICES
    m.insert("USA500IDXUSD".to_string(), (3.00, 3.50, 3.50)); // S&P 500

    // CRYPTO
    m.insert("BTCUSD".to_string(), (4.50, 5.00, 5.00)); // Bitcoin extrême

    m
}

/// Get SL multiplicator for a pair
///
/// # Arguments
/// * `pair` - Trading pair (e.g., "EURUSD")
/// * `is_event_time` - True if economic event happening
/// * `volatility_level` - Optional ATR percentile (0-100). If > 70, uses high_vol multiplier
///
/// # Returns
/// Appropriate multiplicator for the pair, or 2.0 (safe default) if pair unknown
#[allow(dead_code)]
pub fn get_sl_multiplier(
    pair: &str,
    is_event_time: bool,
    volatility_percentile: Option<f64>,
) -> f64 {
    let multipliers = init_multipliers();

    let (default, high_vol, event) = match multipliers.get(pair) {
        Some(&mul) => mul,
        None => {
            // Unknown pair: use conservative safe default
            tracing::warn!(
                "Unknown pair for SL multiplier: {}, using default 2.0",
                pair
            );
            return 2.0;
        }
    };

    // Priority: event_time > high_volatility > default
    if is_event_time {
        return event;
    }

    if let Some(vol_pct) = volatility_percentile {
        if vol_pct > 70.0 {
            return high_vol;
        }
    }

    default
}

/// Calculate final SL with time-of-day adjustment
///
/// # Arguments

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time_zone_critical() {
        assert_eq!(get_time_zone(8), TimeZone::Critical); // Europe open
        assert_eq!(get_time_zone(13), TimeZone::Critical); // US open
        assert_eq!(get_time_zone(16), TimeZone::Critical); // Asia close
    }

    #[test]
    fn test_get_time_zone_calm() {
        assert_eq!(get_time_zone(3), TimeZone::Calm); // Night NY
        assert_eq!(get_time_zone(10), TimeZone::Calm); // European lull
        assert_eq!(get_time_zone(15), TimeZone::Calm); // US lunch
    }

    #[test]
    fn test_get_time_zone_normal() {
        assert_eq!(get_time_zone(11), TimeZone::Normal);
        assert_eq!(get_time_zone(20), TimeZone::Normal);
        assert_eq!(get_time_zone(23), TimeZone::Normal);
    }

    #[test]
    fn test_get_time_multiplier() {
        assert_eq!(get_time_multiplier(TimeZone::Critical), 1.5);
        assert_eq!(get_time_multiplier(TimeZone::Calm), 0.7);
        assert_eq!(get_time_multiplier(TimeZone::Normal), 1.0);
    }

    #[test]
    fn test_get_sl_multiplier_eurusd() {
        let mul = get_sl_multiplier("EURUSD", false, None);
        assert_eq!(mul, 1.40);

        let mul_event = get_sl_multiplier("EURUSD", true, None);
        assert_eq!(mul_event, 1.50);
    }

    #[test]
    fn test_get_sl_multiplier_btcusd() {
        let mul = get_sl_multiplier("BTCUSD", false, None);
        assert_eq!(mul, 4.50);

        let mul_event = get_sl_multiplier("BTCUSD", true, None);
        assert_eq!(mul_event, 5.00);
    }

    #[test]
    fn test_get_sl_multiplier_high_volatility() {
        // When volatility is high (>70 percentile), use high_vol multiplier
        let mul = get_sl_multiplier("EURUSD", false, Some(75.0));
        assert_eq!(mul, 1.50); // Uses high_vol instead of default
    }

    #[test]
    fn test_get_sl_multiplier_unknown_pair() {
        let mul = get_sl_multiplier("UNKNOWN", false, None);
        assert_eq!(mul, 2.0); // Safe default
    }

    #[test]
    #[ignore] // apply_time_adjustment not yet implemented
    fn test_apply_time_adjustment_critical() {
        // let adjusted = apply_time_adjustment(100.0, 13); // US open
        // assert_eq!(adjusted, 150.0);
    }

    #[test]
    #[ignore] // apply_time_adjustment not yet implemented
    fn test_apply_time_adjustment_calm() {
        // let adjusted = apply_time_adjustment(100.0, 3); // Night NY
        // assert_eq!(adjusted, 70.0);
    }

    #[test]
    #[ignore] // apply_time_adjustment not yet implemented
    fn test_apply_time_adjustment_normal() {
        // let adjusted = apply_time_adjustment(100.0, 11); // Normal hour
        // assert_eq!(adjusted, 100.0);
    }

    #[test]
    #[ignore] // apply_time_adjustment not yet implemented
    fn test_btc_realistic_scenario() {
        // BTC at critical time with event
        let atr = 1264.0; // From capture: 2.97% of ~$42,500
        let mul = get_sl_multiplier("BTCUSD", true, None);
        let sl_base = atr * mul;
        assert_eq!(sl_base, 6320.0); // 1264 × 5.0

        // let sl_adjusted = apply_time_adjustment(sl_base, 13); // US open (critical)
        // assert_eq!(sl_adjusted, 9480.0); // 6320 × 1.5
    }

    #[test]
    #[ignore] // apply_time_adjustment not yet implemented
    fn test_btc_calm_scenario() {
        // BTC at calm time, normal
        let atr = 1264.0;
        let mul = get_sl_multiplier("BTCUSD", false, None);
        let sl_base = atr * mul;
        assert_eq!(sl_base, 5688.0); // 1264 × 4.5

        // let sl_adjusted = apply_time_adjustment(sl_base, 3); // Night NY (calm)
        // assert_eq!(sl_adjusted, 3981.6); // 5688 × 0.7
    }

    #[test]
    fn test_sl_base_calculation() {
        // Test the primary SL base calculation function
        let atr = 100.0;
        let sl = calculate_sl_base(atr, "EURUSD", false, None);
        assert_eq!(sl, 140.0); // 100 × 1.4

        let sl_event = calculate_sl_base(atr, "EURUSD", true, None);
        assert_eq!(sl_event, 150.0); // 100 × 1.5

        let sl_btc = calculate_sl_base(atr, "BTCUSD", false, None);
        assert_eq!(sl_btc, 450.0); // 100 × 4.5
    }
}
