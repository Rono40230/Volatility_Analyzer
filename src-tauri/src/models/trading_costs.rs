// models/trading_costs.rs - Profils de coûts de trading par classe d'actif
//
// DEPRECATED (Phase 3): Fallback temporaire pour M1 sans données tick.
// Remplacé progressivement par le spread réel mesuré dans les candles enrichies
// (champs spread_mean dans Candle). Sera supprimé quand toutes les paires
// auront des données tick importées.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingCostProfile {
    pub spread_min: f64,
    pub spread_max: f64,
    pub spread_avg: f64,
    pub slippage: f64,
    /// Multiplicateur du spread pendant une publication de news.
    /// Ex: 3.0 = le spread triple pendant la fenêtre événementielle.
    pub spread_multiplier_event: f64,
    /// Multiplicateur du slippage à l'entrée pendant un événement.
    /// L'entrée subit 2-3× plus de slippage que la sortie car le carnet
    /// d'ordres est déséquilibré au moment de la publication.
    pub entry_slippage_multiplier: f64,
}

impl TradingCostProfile {
    #[allow(dead_code)] // DEPRECATED: fallback pour M1 sans tick data
    pub fn get_profile(symbol: &str) -> Self {
        let s = symbol.to_uppercase();
        
        if s.contains("JPY") && (s.contains("GBP") || s.contains("EUR")) {
            // Crosses volatils (GBPJPY, EURJPY) — spread ×4, slippage entrée ×3
            Self { spread_min: 4.0, spread_max: 8.0, spread_avg: 6.5, slippage: 3.0, spread_multiplier_event: 4.0, entry_slippage_multiplier: 3.0 }
        } else if s.contains("GBP") || s.contains("AUD") {
            // Majors volatiles (GBPUSD, AUDUSD) — spread ×3, slippage entrée ×2.5
            Self { spread_min: 2.0, spread_max: 6.0, spread_avg: 4.0, slippage: 2.0, spread_multiplier_event: 3.0, entry_slippage_multiplier: 2.5 }
        } else if s.contains("NZD") {
            // NZD pairs (NZDUSD, NZDJPY) — spread plus large que les majors
            Self { spread_min: 2.5, spread_max: 6.0, spread_avg: 4.5, slippage: 2.0, spread_multiplier_event: 3.0, entry_slippage_multiplier: 2.5 }
        } else if s.contains("CAD") {
            // CAD pairs (USDCAD, CADJPY) — liquidité correcte
            Self { spread_min: 2.0, spread_max: 5.0, spread_avg: 3.5, slippage: 1.5, spread_multiplier_event: 3.0, entry_slippage_multiplier: 2.0 }
        } else if s.contains("CHF") {
            // CHF pairs (USDCHF, EURCHF) — liquidité moyenne, risque SNB
            Self { spread_min: 1.5, spread_max: 5.0, spread_avg: 3.0, slippage: 1.5, spread_multiplier_event: 3.5, entry_slippage_multiplier: 2.5 }
        } else if s.contains("XAU") || s.contains("GOLD") {
            // Or (Gold) — spread ×4, slippage entrée ×3
            Self { spread_min: 3.0, spread_max: 6.0, spread_avg: 4.0, slippage: 2.0, spread_multiplier_event: 4.0, entry_slippage_multiplier: 3.0 }
        } else if s.contains("BTC") {
            // Crypto (BTC) — spread ×5, slippage entrée ×3
            Self { spread_min: 30.0, spread_max: 60.0, spread_avg: 40.0, slippage: 20.0, spread_multiplier_event: 5.0, entry_slippage_multiplier: 3.0 }
        } else if s.contains("DAX") || s.contains("GER40") || s.contains("DE40") {
            // DAX — spread ×3, slippage entrée ×2.5
            Self { spread_min: 4.0, spread_max: 8.0, spread_avg: 6.0, slippage: 3.0, spread_multiplier_event: 3.0, entry_slippage_multiplier: 2.5 }
        } else if s.contains("US30") || s.contains("DJI") || s.contains("NAS") || s.contains("USTEC") {
            // Indices US — spread ×3, slippage entrée ×2.5
            Self { spread_min: 5.0, spread_max: 10.0, spread_avg: 7.5, slippage: 5.0, spread_multiplier_event: 3.0, entry_slippage_multiplier: 2.5 }
        } else {
            // Majors liquides (EURUSD, USDJPY) — spread ×3, slippage entrée ×2
            Self { spread_min: 1.0, spread_max: 4.0, spread_avg: 2.5, slippage: 1.0, spread_multiplier_event: 3.0, entry_slippage_multiplier: 2.0 }
        }
    }

    /// Retourne le spread effectif pendant une fenêtre événementielle.
    /// Applique le multiplicateur au spread moyen.
    #[allow(dead_code)]
    pub fn spread_during_event(&self) -> f64 {
        self.spread_avg * self.spread_multiplier_event
    }

    /// Retourne le coût total par trade pendant un événement.
    /// Spread événementiel + slippage asymétrique (entrée majorée + sortie normale).
    #[allow(dead_code)]
    pub fn cost_per_trade_event(&self) -> f64 {
        let entry_slippage = self.slippage * self.entry_slippage_multiplier;
        let exit_slippage = self.slippage; // Sortie = slippage normal
        self.spread_during_event() + entry_slippage + exit_slippage
    }

    /// Retourne le coût total par trade en conditions normales.
    #[allow(dead_code)]
    pub fn cost_per_trade_normal(&self) -> f64 {
        self.spread_avg + (self.slippage * 2.0)
    }
}
