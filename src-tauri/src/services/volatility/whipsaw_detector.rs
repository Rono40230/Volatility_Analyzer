// services/volatility/whipsaw_detector.rs - Détection de fausses cassures (whipsaw)
// CORRECTION PHASE 8: Simulation réaliste d'un Straddle
use super::whipsaw_simulator::{simulate_straddle_trade, TradeResult};
use crate::models::Candle;
use tracing::{debug, info};

/// Calcule la fréquence des whipsaws de manière réaliste
///
/// Logique corrigée :
/// 1. Test UN SEUL trade par jour (à l'heure optimale)
/// 2. Fenêtre : quarter optimal + 1 heure (60 candles M1)
/// 3. Whipsaw = Buy + Sell triggers + au moins 1 SL hit
/// 4. Compte win/loss réels avec TP/SL dynamiques
pub fn calculer_frequence_whipsaw(candles: &[Candle], offset_pips: f64, symbol: &str) -> WhipsawAnalysis {
    if candles.len() < 61 {
        // Besoin au moins 61 candles (1 min entry + 60 min test window)
        return WhipsawAnalysis::default();
    }

    let asset_props = crate::models::AssetProperties::from_symbol(symbol);

    info!("🔄 Calculant whipsaw frequency (mode réaliste) pour {}", symbol);
    info!("   - Offset: {} pips", offset_pips);
    info!("   - Fenêtre: 60 candles (quarter + 1h)");

    let mut daily_groups: std::collections::HashMap<String, Vec<&Candle>> =
        std::collections::HashMap::new();

    // Grouper par jour
    for candle in candles.iter() {
        let day_key = candle.datetime.format("%Y-%m-%d").to_string();
        daily_groups
            .entry(day_key)
            .or_default()
            .push(candle);
    }

    let mut win_count = 0;
    let mut loss_count = 0;
    let mut whipsaws = Vec::new();
    let mut simulated_trades = Vec::new();

    // Pour chaque jour, simuler UN trade au quarter optimal
    for (day_key, day_candles) in daily_groups.iter() {
        if day_candles.len() < 61 {
            continue; // Pas assez de candles dans la journée
        }

        // Tester le trade à l'index 0 de la journée (quarter optimal)
        // Dans une vraie implémentation, on choisirait l'ATR peak de la journée
        let entry_index = 0;
        let entry_candle = day_candles[entry_index];
        let entry_price = entry_candle.close;

        // Fenêtre de test : 60 candles suivantes (quarter + 1h)
        let window_end = (entry_index + 61).min(day_candles.len());
        if window_end - entry_index < 61 {
            continue; // Pas assez de candles après l'entrée
        }

        let test_window = &day_candles[entry_index + 1..window_end];

        // Paramètres dynamiques basés sur ATR (simplifié pour cette version)
        // ATR moyen = offset_pips / 1.5 (estimation)
        let atr_estimate = offset_pips / 1.5;
        let sl_pips = atr_estimate * 0.5; // SL = ATR * 0.5
        let tp_pips = atr_estimate * 1.0; // TP = ATR * 1.0

        // Simuler le Straddle
        let result =
            simulate_straddle_trade(entry_price, offset_pips, sl_pips, tp_pips, test_window, asset_props.pip_value);

        debug!(
            "📊 {}: Trade simulation - entry={:.4}, offset={}, SL={}, TP={}, result={:?}",
            day_key, entry_price, offset_pips, sl_pips, tp_pips, result
        );

        match result {
            TradeResult::Win => win_count += 1,
            TradeResult::Loss => {
                loss_count += 1;
                whipsaws.push(WhipsawDetail {
                    entry_index,
                    entry_price,
                    buy_stop: entry_price + asset_props.denormalize(offset_pips),
                    sell_stop: entry_price - asset_props.denormalize(offset_pips),
                    buy_trigger_index: 0,
                    sell_trigger_index: 0,
                });
            }
            TradeResult::Timeout => {
                // Timeout = pas de décision, ne compte pas
            }
        }

        simulated_trades.push((day_key.clone(), result));
    }

    let total_trades = win_count + loss_count;
    let whipsaw_frequency = if total_trades > 0 {
        loss_count as f64 / total_trades as f64
    } else {
        0.0
    };

    info!(
        "✅ Résultats: {} wins, {} losses ({:.1}% whipsaw), {} timeouts",
        win_count,
        loss_count,
        whipsaw_frequency * 100.0,
        daily_groups.len() - total_trades
    );

    WhipsawAnalysis {
        total_trades,
        whipsaw_count: loss_count,
        whipsaw_frequency,
        offset_pips,
        candles_analyzed: candles.len(),
        whipsaws,
        risk_level: calculer_niveau_risque(whipsaw_frequency),
    }
}

/// Évalue le niveau de risque basé sur la fréquence de whipsaw (réaliste)
/// CORRECTION: Seuils ajustés pour simulation réaliste (pas 1425 tests/jour)
fn calculer_niveau_risque(whipsaw_frequency: f64) -> WhipsawRiskLevel {
    if whipsaw_frequency < 0.05 {
        WhipsawRiskLevel::VeryLow // < 5%
    } else if whipsaw_frequency < 0.15 {
        WhipsawRiskLevel::Low // 5-15%
    } else if whipsaw_frequency < 0.30 {
        WhipsawRiskLevel::Moderate // 15-30%
    } else if whipsaw_frequency < 0.50 {
        WhipsawRiskLevel::High // 30-50%
    } else {
        WhipsawRiskLevel::VeryHigh // > 50%
    }
}

/// Résultat de l'analyse des whipsaws
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WhipsawAnalysis {
    pub total_trades: usize,
    pub whipsaw_count: usize,
    pub whipsaw_frequency: f64,
    pub offset_pips: f64,
    pub candles_analyzed: usize,
    pub whipsaws: Vec<WhipsawDetail>,
    pub risk_level: WhipsawRiskLevel,
}

impl Default for WhipsawAnalysis {
    fn default() -> Self {
        Self {
            total_trades: 0,
            whipsaw_count: 0,
            whipsaw_frequency: 0.0,
            offset_pips: 0.0,
            candles_analyzed: 0,
            whipsaws: Vec::new(),
            risk_level: WhipsawRiskLevel::Unknown,
        }
    }
}

/// Détail d'un whipsaw survenu
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WhipsawDetail {
    pub entry_index: usize,
    pub entry_price: f64,
    pub buy_stop: f64,
    pub sell_stop: f64,
    pub buy_trigger_index: usize,
    pub sell_trigger_index: usize,
}

/// Niveau de risque basé sur la fréquence de whipsaw
/// CORRECTION PHASE 8: Seuils réalistes (simulation 1 trade/jour, 60 min window)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhipsawRiskLevel {
    VeryLow,  // < 5%   - Excellent, très peu de fausses sorties
    Low,      // 5-15%  - Bon, fausses sorties rares
    Moderate, // 15-30% - Acceptable, quelques fausses sorties
    High,     // 30-50% - Risqué, beaucoup de fausses sorties
    VeryHigh, // > 50%  - Très risqué, la majorité sont des whipsaws
    Unknown,  // Données insuffisantes
}

impl WhipsawRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WhipsawRiskLevel::VeryLow => "Très Bas",
            WhipsawRiskLevel::Low => "Bas",
            WhipsawRiskLevel::Moderate => "Modéré",
            WhipsawRiskLevel::High => "Élevé",
            WhipsawRiskLevel::VeryHigh => "Très Élevé",
            WhipsawRiskLevel::Unknown => "Inconnu",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            WhipsawRiskLevel::VeryLow => "#22c55e",  // Green
            WhipsawRiskLevel::Low => "#84cc16",      // Lime
            WhipsawRiskLevel::Moderate => "#eab308", // Yellow
            WhipsawRiskLevel::High => "#f97316",     // Orange
            WhipsawRiskLevel::VeryHigh => "#ef4444", // Red
            WhipsawRiskLevel::Unknown => "#6b7280",  // Gray
        }
    }
}

/// PHASE 7a-2: Whipsaw Root Cause Analysis
/// Classifier whipsaws en "early" (avant peak) et "late" (après peak)

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_candle(i: usize, high: f64, low: f64) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now() + chrono::Duration::minutes(i as i64),
            open: 1.0855,
            high,
            low,
            close: 1.0855 + (high - 1.0855) / 2.0,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_whipsaw_detection_basic() {
        let mut candles = Vec::new();
        for i in 0..100 {
            // Range large pour déclencher des trades avec un petit offset
            candles.push(create_test_candle(i, 1.0895, 1.0815));
        }
        let result = calculer_frequence_whipsaw(&candles, 10.0, "EURUSD");
        assert!(result.total_trades > 0);
    }

    #[test]
    fn test_whipsaw_empty_data() {
        let candles = vec![];
        let result = calculer_frequence_whipsaw(&candles, 50.0, "EURUSD");
        assert_eq!(result.total_trades, 0);
    }
}
