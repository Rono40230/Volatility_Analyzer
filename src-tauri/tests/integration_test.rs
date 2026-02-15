// tests/integration_test.rs - Tests d'intégration et non-régression
// Phase 6.2 : pipeline tick → M1 enrichie → analyse entry point
// Phase 6.3 : non-régression M1 classiques (sans spread)

use tauri_app_lib::models::entry_analysis::EntryAnalysisConfig;
use tauri_app_lib::models::Candle;
use tauri_app_lib::services::entry_point_analyzer::analyze_entry_points;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    // ─── Helpers ──────────────────────────────────────────────────────────

    /// Crée une candle M1 enrichie (avec spread) pour les tests d'intégration
    fn enriched_candle(
        day: u32,
        hour: u32,
        min: u32,
        close: f64,
        high: f64,
        low: f64,
        spread: f64,
    ) -> Candle {
        Candle {
            symbol: "EURUSD".to_string(),
            datetime: Utc.with_ymd_and_hms(2025, 1, day, hour, min, 0).unwrap(),
            open: close,
            high,
            low,
            close,
            volume: 100.0,
            spread_mean: Some(spread),
            spread_open: Some(spread),
            spread_high: Some(spread * 1.2),
            spread_low: Some(spread * 0.8),
            spread_close: Some(spread),
            tick_count: Some(50),
            ..Default::default()
        }
    }

    /// Crée une candle M1 classique (SANS spread) pour tests de non-régression
    fn classic_candle(day: u32, hour: u32, min: u32, close: f64, high: f64, low: f64) -> Candle {
        Candle {
            symbol: "EURUSD".to_string(),
            datetime: Utc.with_ymd_and_hms(2025, 1, day, hour, min, 0).unwrap(),
            open: close,
            high,
            low,
            close,
            volume: 100.0,
            // Pas de spread → mode dégradé
            ..Default::default()
        }
    }

    // ─── 6.2 : Tests d'intégration tick → entry ────────────────────────

    #[test]
    fn test_modules_compile() {
        // Vérifie que l'ensemble des modules se compilent (import croisés)
    }

    #[test]
    fn test_integration_enriched_candles_to_entry_analysis() {
        // Simule le résultat d'un import tick : 5 jours de M1 enrichies
        // pour le quarter 14:30-14:44 avec spread réel
        let config = EntryAnalysisConfig {
            forward_minutes: 10,
            spread_threshold_pips: 8.0,
            min_samples: 2,
        };

        let mut candles = Vec::new();
        // 5 jours (lun-ven), candles pour 14:30-14:44 + 10 min forward
        for day in [6, 7, 8, 9, 10] {
            // Quarter 14:30 à 14:44
            for min in 30..45 {
                let vol = if min >= 30 && min <= 35 { 0.0020 } else { 0.0010 };
                candles.push(enriched_candle(
                    day,
                    14,
                    min,
                    1.1000,
                    1.1000 + vol,
                    1.1000 - vol,
                    0.0002, // spread 2 pips
                ));
            }
            // Forward 14:45-14:54
            for min in 45..55 {
                candles.push(enriched_candle(
                    day,
                    14,
                    min,
                    1.1005,
                    1.1040,
                    1.0970,
                    0.0003,
                ));
            }
        }

        let result = analyze_entry_points(&candles, "EURUSD", "NFP", 14, 2, &config);
        assert!(result.is_ok(), "Should succeed with enriched candles: {:?}", result.err());

        let r = result.unwrap();
        assert_eq!(r.symbol, "EURUSD");
        assert_eq!(r.sample_size, 5);
        // Le spread moyen à l'entrée devrait être ~2 pips (0.0002 / 0.0001)
        assert!(
            r.avg_spread_at_entry_pips > 0.0,
            "Spread should be computed from tick data"
        );
        // Win rate doit être basé sur le comptage réel
        assert!(
            r.real_win_rate >= 0.0 && r.real_win_rate <= 1.0,
            "Win rate must be in [0,1]"
        );
        // Minute details doit couvrir les 15 offsets
        assert!(
            r.minute_details.len() <= 15,
            "Max 15 offsets in a quarter"
        );
        // Le profit net doit être mouvement - spread (pas le mouvement brut)
        assert!(
            r.avg_net_profit_pips < r.avg_movement_pips || r.avg_movement_pips == 0.0,
            "Net profit should be less than gross movement (spread deducted)"
        );
    }

    #[test]
    fn test_integration_spread_affects_profit() {
        // Même données mais spread élevé → profit net plus faible
        let config = EntryAnalysisConfig {
            forward_minutes: 5,
            spread_threshold_pips: 100.0, // pas de filtre
            min_samples: 1,
        };

        let mut candles_low_spread = Vec::new();
        let mut candles_high_spread = Vec::new();

        for day in [6, 7, 8] {
            for min in 30..36 {
                candles_low_spread.push(enriched_candle(
                    day, 14, min, 1.1000, 1.1020, 1.0980, 0.0001,
                ));
                candles_high_spread.push(enriched_candle(
                    day, 14, min, 1.1000, 1.1020, 1.0980, 0.0010,
                ));
            }
            for min in 36..42 {
                candles_low_spread.push(enriched_candle(
                    day, 14, min, 1.1010, 1.1040, 1.0970, 0.0001,
                ));
                candles_high_spread.push(enriched_candle(
                    day, 14, min, 1.1010, 1.1040, 1.0970, 0.0010,
                ));
            }
        }

        let r_low =
            analyze_entry_points(&candles_low_spread, "EURUSD", "Test", 14, 2, &config).unwrap();
        let r_high =
            analyze_entry_points(&candles_high_spread, "EURUSD", "Test", 14, 2, &config).unwrap();

        // Avec un spread plus élevé, le profit net doit être inférieur
        assert!(
            r_high.avg_net_profit_pips < r_low.avg_net_profit_pips,
            "Higher spread should reduce net profit: low={:.2}, high={:.2}",
            r_low.avg_net_profit_pips,
            r_high.avg_net_profit_pips
        );
    }

    #[test]
    fn test_integration_non_tradable_zone_detection() {
        // Candles avec spread très élevé pour certaines minutes
        let config = EntryAnalysisConfig {
            forward_minutes: 5,
            spread_threshold_pips: 5.0,
            min_samples: 1,
        };

        let mut candles = Vec::new();
        for day in [6, 7, 8] {
            // Offset 0 (min 30) : spread 10 pips → non-tradable
            candles.push(enriched_candle(
                day, 14, 30, 1.1000, 1.1010, 1.0990, 0.0010,
            ));
            // Offset 1 (min 31) : spread 2 pips → tradable
            candles.push(enriched_candle(
                day, 14, 31, 1.1000, 1.1010, 1.0990, 0.0002,
            ));
            // Forward
            for min in 32..38 {
                candles.push(enriched_candle(
                    day, 14, min, 1.1005, 1.1030, 1.0980, 0.0002,
                ));
            }
        }

        let result =
            analyze_entry_points(&candles, "EURUSD", "Test", 14, 2, &config).unwrap();
        // Offset 0 doit être marqué non-tradable
        assert!(
            result.non_tradable_minutes.contains(&0),
            "Offset 0 (spread 10 pips > seuil 5) should be non-tradable"
        );
    }

    // ─── 6.3 : Tests de non-régression (M1 classiques sans spread) ────

    #[test]
    fn test_regression_classic_candles_still_work() {
        // Les candles M1 classiques (spread=None) doivent continuer à fonctionner
        // C'est le mode dégradé : spread traité comme 0
        let config = EntryAnalysisConfig {
            forward_minutes: 5,
            spread_threshold_pips: 10.0,
            min_samples: 1,
        };

        let mut candles = Vec::new();
        for day in [6, 7, 8, 9, 10] {
            for min in 30..45 {
                candles.push(classic_candle(day, 14, min, 1.1000, 1.1015, 1.0985));
            }
            for min in 45..51 {
                candles.push(classic_candle(day, 14, min, 1.1010, 1.1040, 1.0970));
            }
        }

        let result = analyze_entry_points(&candles, "EURUSD", "No-spread", 14, 2, &config);
        assert!(
            result.is_ok(),
            "Classic M1 without spread should still work: {:?}",
            result.err()
        );

        let r = result.unwrap();
        assert_eq!(r.sample_size, 5);
        // Spread at entry devrait être 0 (aucun spread dans les données)
        assert!(
            r.avg_spread_at_entry_pips < 0.1,
            "No spread data → spread should be ~0, got {:.2}",
            r.avg_spread_at_entry_pips
        );
        // Le profit brut = profit net quand spread=0
        assert!(
            (r.avg_net_profit_pips - r.avg_movement_pips).abs() < 1.0,
            "Without spread, net ≈ gross: net={:.2}, gross={:.2}",
            r.avg_net_profit_pips,
            r.avg_movement_pips
        );
    }

    #[test]
    fn test_regression_spread_none_fields_no_panic() {
        // Vérifie qu'aucun unwrap() ne panique sur les champs spread=None
        let candle = classic_candle(6, 14, 30, 1.1000, 1.1020, 1.0980);
        assert!(candle.spread_mean.is_none());
        assert!(candle.spread_open.is_none());
        assert!(candle.tick_count.is_none());
        // shadow_ratio et true_range ne dépendent pas du spread
        let tr = candle.true_range(Some(1.0990));
        assert!(tr > 0.0);
        let sr = candle.shadow_ratio();
        assert!(sr >= 0.0 || sr.is_infinite());
    }

    #[test]
    fn test_regression_atr_works_without_spread() {
        // ATR ne dépend pas du spread — doit toujours fonctionner
        let candles: Vec<Candle> = (0..10)
            .map(|i| classic_candle(6, 14, 30 + i, 1.1000, 1.1010 + 0.0001 * i as f64, 1.0990))
            .collect();
        let atr = tauri_app_lib::services::atr::calculate_atr_sma(&candles, 5);
        assert!(atr > 0.0, "ATR should work without spread data");
    }
}
