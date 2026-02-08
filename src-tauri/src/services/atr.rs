// services/atr.rs
// Module centralisé pour le calcul du True Range et de l'ATR
// Toutes les variantes (SMA, EMA) sont ici.
// Les autres modules DOIVENT utiliser ces fonctions au lieu de recalculer.

use crate::models::Candle;

/// Calcule le True Range d'une bougie.
/// Si `prev_close` est fourni, utilise la formule standard :
///   TR = max(H - L, |H - prevC|, |L - prevC|)
/// Sinon (premier candle, gap), fallback sur le range simple H - L.
pub fn calculate_true_range(high: f64, low: f64, prev_close: Option<f64>) -> f64 {
    let range = high - low;
    match prev_close {
        Some(pc) => range
            .max((high - pc).abs())
            .max((low - pc).abs()),
        None => range,
    }
}

/// Calcule une série de True Ranges à partir de candles.
/// Le premier candle utilise H-L (pas de prev_close disponible).
pub fn calculate_true_range_series(candles: &[Candle]) -> Vec<f64> {
    if candles.is_empty() {
        return Vec::new();
    }

    let mut tr_values = Vec::with_capacity(candles.len());
    for (i, candle) in candles.iter().enumerate() {
        let prev_close = if i > 0 { Some(candles[i - 1].close) } else { None };
        tr_values.push(calculate_true_range(candle.high, candle.low, prev_close));
    }
    tr_values
}

/// ATR par moyenne arithmétique simple (SMA) sur `period` candles.
/// Retourne 0.0 si pas assez de données.
pub fn calculate_atr_sma(candles: &[Candle], period: usize) -> f64 {
    let tr_values = calculate_true_range_series(candles);
    if tr_values.is_empty() {
        return 0.0;
    }
    let period = period.max(1).min(tr_values.len());
    // Utiliser les `period` dernières valeurs
    let start = tr_values.len().saturating_sub(period);
    let slice = &tr_values[start..];
    slice.iter().sum::<f64>() / slice.len() as f64
}

/// ATR par moyenne mobile exponentielle (EMA) sur `period` candles.
/// Initialise le EMA avec la SMA des `period` premières valeurs,
/// puis applique le lissage exponentiel.
pub fn calculate_atr_ema(candles: &[Candle], period: usize) -> f64 {
    let tr_values = calculate_true_range_series(candles);
    if tr_values.is_empty() {
        return 0.0;
    }
    let period = period.max(1).min(tr_values.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    // SMA initiale sur les `period` premières valeurs
    let sma_init: f64 = tr_values[..period].iter().sum::<f64>() / period as f64;
    let mut ema = sma_init;

    // Lissage EMA sur le reste
    for value in tr_values.iter().skip(period) {
        ema = *value * multiplier + ema * (1.0 - multiplier);
    }
    ema
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn make_candle(open: f64, high: f64, low: f64, close: f64, offset_min: i64) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: DateTime::<Utc>::from_timestamp_millis(
                1609459200000 + (offset_min * 60000),
            )
            .expect("timestamp must be valid"),
            open,
            high,
            low,
            close,
            volume: 100.0,
        }
    }

    #[test]
    fn test_true_range_without_prev_close() {
        let tr = calculate_true_range(1.1050, 1.1000, None);
        assert!((tr - 0.005).abs() < 1e-10);
    }

    #[test]
    fn test_true_range_with_gap_up() {
        // Gap up: prev_close=1.1000, current H=1.1080, L=1.1050
        // TR = max(0.003, |1.1080-1.1000|, |1.1050-1.1000|) = max(0.003, 0.008, 0.005) = 0.008
        let tr = calculate_true_range(1.1080, 1.1050, Some(1.1000));
        assert!((tr - 0.008).abs() < 1e-10);
    }

    #[test]
    fn test_atr_sma_basic() {
        let candles = vec![
            make_candle(1.1000, 1.1020, 1.0990, 1.1010, 0),
            make_candle(1.1010, 1.1030, 1.0995, 1.1025, 1),
            make_candle(1.1025, 1.1040, 1.1005, 1.1035, 2),
        ];
        let atr = calculate_atr_sma(&candles, 3);
        assert!(atr > 0.0);
    }

    #[test]
    fn test_atr_ema_basic() {
        let candles = vec![
            make_candle(1.1000, 1.1020, 1.0990, 1.1010, 0),
            make_candle(1.1010, 1.1030, 1.0995, 1.1025, 1),
            make_candle(1.1025, 1.1040, 1.1005, 1.1035, 2),
            make_candle(1.1035, 1.1060, 1.1015, 1.1050, 3),
        ];
        let atr = calculate_atr_ema(&candles, 3);
        assert!(atr > 0.0);
    }

    #[test]
    fn test_empty_candles() {
        assert_eq!(calculate_atr_sma(&[], 14), 0.0);
        assert_eq!(calculate_atr_ema(&[], 14), 0.0);
    }
}
