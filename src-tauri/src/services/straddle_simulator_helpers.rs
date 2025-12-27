// services/straddle_simulator_helpers.rs - Helpers pour simulation Straddle
// Contient les fonctions utilitaires pour éviter de dépasser 300 lignes

use crate::models::Candle;

/// Calcule l'ATR moyen (Average True Range) pour une liste de candles
/// Utilise une EMA(14) des True Ranges pour être conforme au standard MT5
/// et donner plus de poids aux mouvements récents
pub fn calculer_atr_moyen(candles: &[Candle]) -> f64 {
    let mut tr_values: Vec<f64> = Vec::new();

    // Calcul du True Range pour chaque candle
    for i in 0..candles.len() {
        let high = candles[i].high;
        let low = candles[i].low;
        let close = if i > 0 {
            candles[i - 1].close
        } else {
            candles[i].close
        };

        let tr = (high - low)
            .max((high - close).abs())
            .max((low - close).abs());
        tr_values.push(tr);
    }

    if tr_values.is_empty() {
        return 0.0;
    }

    // Calcul de l'EMA(14) des True Ranges
    calculate_ema(&tr_values, 14)
}

/// Calcule l'EMA (Exponential Moving Average) avec une période donnée
pub fn calculate_ema(values: &[f64], period: usize) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let period = period.min(values.len()); // Limiter la période au nombre de valeurs disponibles

    // Coefficient de lissage EMA = 2 / (period + 1)
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Initialiser avec la SMA des premières valeurs
    let sma_init: f64 = values[0..period].iter().sum::<f64>() / period as f64;
    let mut ema = sma_init;

    // Appliquer l'EMA sur les valeurs restantes
    for i in period..values.len() {
        ema = values[i] * multiplier + ema * (1.0 - multiplier);
    }

    ema
}

// --- STRUCTURES DE COÛT ---

#[derive(Debug, Clone)]
pub struct AssetCost {
    pub spread_pips: f64,
    pub slippage_pips: f64,
}

/// Récupère les coûts estimés (Spread + Slippage) pour le News Trading selon l'actif
pub fn get_asset_cost(symbol: &str) -> AssetCost {
    let s = symbol.to_uppercase();
    if s.contains("JPY") && (s.contains("GBP") || s.contains("EUR")) {
        // Crosses volatils (GBPJPY, EURJPY)
        AssetCost { spread_pips: 6.0, slippage_pips: 3.0 }
    } else if s.contains("GBP") {
        // Majors volatiles (GBPUSD)
        AssetCost { spread_pips: 4.0, slippage_pips: 2.0 }
    } else if s.contains("XAU") || s.contains("GOLD") {
        // Or (Gold)
        AssetCost { spread_pips: 5.0, slippage_pips: 2.0 }
    } else if s.contains("BTC") {
        // Crypto (BTC) - Valeurs élevées en points
        AssetCost { spread_pips: 50.0, slippage_pips: 20.0 }
    } else if s.contains("DAX") || s.contains("GER40") || s.contains("DE40") {
        // DAX
        AssetCost { spread_pips: 6.0, slippage_pips: 3.0 }
    } else if s.contains("US30") || s.contains("DJI") {
        // Dow Jones
        AssetCost { spread_pips: 8.0, slippage_pips: 5.0 }
    } else {
        // Majors liquides (EURUSD, USDJPY) par défaut
        AssetCost { spread_pips: 2.5, slippage_pips: 1.0 }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WhipsawDetail {
    pub entry_index: usize,
    pub entry_price: f64,
    pub buy_stop: f64,
    pub sell_stop: f64,
    pub buy_trigger_index: usize,
    pub sell_trigger_index: usize,
    pub net_loss_pips: f64, // Perte réelle incluant spread/slippage
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StraddleSimulationResult {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub whipsaws: usize,
    pub win_rate_percentage: f64,
    pub whipsaw_frequency_percentage: f64,
    pub offset_optimal_pips: f64,
    pub percentile_95_wicks: f64,
    pub risk_level: String,
    pub risk_color: String,
    // Valeurs pondérées par le whipsaw (Option B - affichage direct)
    pub win_rate_adjusted: f64,        // Win Rate pondéré par whipsaw
    pub sl_adjusted_pips: f64,         // SL ajusté par whipsaw
    pub trailing_stop_adjusted: f64,   // Trailing Stop réduit
    pub timeout_adjusted_minutes: i32, // Timeout réduit
    pub whipsaw_details: Vec<WhipsawDetail>, // Détails de chaque whipsaw
    
    // Nouvelles métriques financières (Réalité News Trading)
    pub total_pnl_net_pips: f64,       // P&L Net cumulé
    pub avg_trade_cost_pips: f64,      // Coût moyen par trade (Spread + Slippage)
    pub is_profitable_net: bool,       // Si P&L Net > 0
}

/// Calcule le risque et la couleur basé sur la fréquence whipsaw
pub fn calculate_risk_level(whipsaw_freq_pct: f64) -> (String, String) {
    if whipsaw_freq_pct < 10.0 {
        ("Faible".to_string(), "#22c55e".to_string())
    } else if whipsaw_freq_pct < 20.0 {
        ("Moyen".to_string(), "#eab308".to_string())
    } else if whipsaw_freq_pct < 30.0 {
        ("Élevé".to_string(), "#f97316".to_string())
    } else {
        ("Critique".to_string(), "#ef4444".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level() {
        let (level, _) = calculate_risk_level(5.0);
        assert_eq!(level, "Faible");
        let (level, _) = calculate_risk_level(25.0);
        assert_eq!(level, "Élevé");
    }
}
