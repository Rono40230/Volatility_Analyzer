// services/volatility/offset_calculator.rs - Calcul de l'offset optimal pour Straddle
use crate::models::Candle;
use crate::services::straddle_simulator_helpers::calculate_atr_mean;
use tracing::info;

/// Calcule l'offset optimal pour un Straddle basé sur ATR
///
/// Algorithme (CORRECTED - PHASE 2):
/// 1. Calculer l'ATR moyen (Average True Range)
/// 2. Appliquer le multiplicateur Straddle: ATR × 1.75
/// 3. Retourner en points (pips × 10000)
///
/// Rationale:
/// - Offset = ATR × 1.75 protège contre la vraie volatilité (pas juste les wicks)
/// - Formule Straddle standard: 1.5 (conservateur) à 2.0 (agressif)
/// - 1.75 = équilibre optimal
pub fn calculate_optimal_offset(candles: &[Candle]) -> f64 {
    if candles.is_empty() {
        return 10.0; // Fallback : 10 pips minimum
    }

    // 1. Calculer l'ATR moyen (volatilité réelle)
    let atr_mean = calculate_atr_mean(candles);

    if atr_mean <= 0.0 {
        return 10.0; // Fallback si ATR invalide
    }

    // 2. Appliquer la formule Straddle: Offset = ATR × 1.75
    // Cela signifie:
    // - Buy Stop = Open + (ATR × 1.75)
    // - Sell Stop = Open - (ATR × 1.75)
    let optimal_offset = atr_mean * 1.75;

    // 3. Convertir en pips (multiplier par 10000)
    let result = optimal_offset * 10000.0;

    info!(
        "✅ Offset final (ATR-based): {:.0} pips | ATR_mean: {:.8}",
        result, atr_mean
    );

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_offset_calculation_atr_based() {
        // Créer 3 candles avec ATR connu
        let base_time = DateTime::parse_from_rfc3339("2024-12-01T14:00:00Z")
            .expect("valid RFC3339 datetime")
            .with_timezone(&Utc);

        let candles = vec![
            Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: base_time,
                open: 1.0850,
                high: 1.0860,
                low: 1.0840,
                close: 1.0855,
                volume: 1000.0,
            },
            Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: base_time + chrono::Duration::minutes(1),
                open: 1.0855,
                high: 1.0870,
                low: 1.0845,
                close: 1.0860,
                volume: 1000.0,
            },
            Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: base_time + chrono::Duration::minutes(2),
                open: 1.0860,
                high: 1.0875,
                low: 1.0850,
                close: 1.0865,
                volume: 1000.0,
            },
        ];

        let offset = calculate_optimal_offset(&candles);
        assert!(offset > 0.0, "Offset doit être > 0");

        // Calcul ATR (TR1=0.0020, TR2=0.0025, TR3=0.0025) → ATR_mean ≈ 0.00233
        // Offset = ATR_mean × 1.75 ≈ 0.00233 × 1.75 ≈ 0.00408
        // En pips = 0.00408 × 10000 ≈ 40.8 pips
        // Plage acceptable: 35-50 pips
        assert!(
            offset > 35.0 && offset < 50.0,
            "Offset {:.1} doit être entre 35 et 50 pips",
            offset
        );
    }

    #[test]
    fn test_empty_candles() {
        let candles = vec![];
        let offset = calculate_optimal_offset(&candles);
        assert_eq!(offset, 10.0); // Fallback
    }
}
