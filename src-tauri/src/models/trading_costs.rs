use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingCostProfile {
    pub spread_min: f64,
    pub spread_max: f64,
    pub spread_avg: f64,
    pub slippage: f64,
}

impl TradingCostProfile {
    pub fn get_profile(symbol: &str) -> Self {
        let s = symbol.to_uppercase();
        
        if s.contains("JPY") && (s.contains("GBP") || s.contains("EUR")) {
            // Crosses volatils (GBPJPY, EURJPY)
            Self { spread_min: 4.0, spread_max: 8.0, spread_avg: 6.5, slippage: 3.0 }
        } else if s.contains("GBP") || s.contains("AUD") {
            // Majors volatiles (GBPUSD, AUDUSD)
            Self { spread_min: 2.0, spread_max: 6.0, spread_avg: 4.0, slippage: 2.0 }
        } else if s.contains("XAU") || s.contains("GOLD") {
            // Or (Gold)
            Self { spread_min: 3.0, spread_max: 6.0, spread_avg: 4.0, slippage: 2.0 }
        } else if s.contains("BTC") {
            // Crypto (BTC)
            Self { spread_min: 30.0, spread_max: 60.0, spread_avg: 40.0, slippage: 20.0 }
        } else if s.contains("DAX") || s.contains("GER40") || s.contains("DE40") {
            // DAX
            Self { spread_min: 4.0, spread_max: 8.0, spread_avg: 6.0, slippage: 3.0 }
        } else if s.contains("US30") || s.contains("DJI") || s.contains("NAS") || s.contains("USTEC") {
            // Indices US
            Self { spread_min: 5.0, spread_max: 10.0, spread_avg: 7.5, slippage: 5.0 }
        } else {
            // Majors liquides (EURUSD, USDJPY) par défaut
            Self { spread_min: 1.0, spread_max: 4.0, spread_avg: 2.5, slippage: 1.0 }
        }
    }
}
