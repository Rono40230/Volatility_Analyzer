// models/asset_class.rs - Détection et normalisation des actifs
// Conforme .clinerules : < 150L

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    ForexMajor, // 5 digits (EURUSD)
    ForexJpy,   // 3 digits (USDJPY)
    Gold,       // 2 digits (XAUUSD)
    Silver,     // 3 digits (XAGUSD)
    Crypto,     // Variable (BTCUSD)
    Index,      // Variable (DAX, US30)
    Commodity,  // Variable (OIL, NGAS)
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetProperties {
    pub asset_type: AssetType,
    pub pip_value: f64, // Valeur d'un "pip" ou "point" standardisé
    pub unit: String,   // "pips", "points", "$"
    pub display_digits: usize,
}

impl AssetProperties {
    pub fn from_symbol(symbol: &str) -> Self {
        let s = symbol.to_uppercase();

        if s.contains("JPY") {
            AssetProperties {
                asset_type: AssetType::ForexJpy,
                pip_value: 0.01,
                unit: "pips".to_string(),
                display_digits: 1,
            }
        } else if s.contains("XAU") || s.contains("GOLD") {
            AssetProperties {
                asset_type: AssetType::Gold,
                pip_value: 0.1, // Convention standard Gold: 0.1$ = 1 pip
                unit: "pips".to_string(),
                display_digits: 1,
            }
        } else if s.contains("XAG") || s.contains("SILVER") {
            AssetProperties {
                asset_type: AssetType::Silver,
                pip_value: 0.01, // Convention Silver
                unit: "pips".to_string(),
                display_digits: 2,
            }
        } else if s.contains("BTC") || s.contains("ETH") || s.contains("CRYPTO") || s.contains("SOL") || s.contains("BNB") || s.contains("XRP") || s.contains("ADA") || s.contains("DOT")
            || s.contains("LTC") || s.contains("BCH") || s.contains("DOGE") || s.contains("SHIB") || s.contains("LINK") || s.contains("MATIC") || s.contains("AVAX") || s.contains("UNI") || s.contains("XLM") || s.contains("TRX") || s.contains("ATOM") || s.contains("NEAR") || s.contains("PEPE")
        {
            AssetProperties {
                asset_type: AssetType::Crypto,
                pip_value: 1.0, // Crypto: 1$ = 1 point
                unit: "pts".to_string(),
                display_digits: 0,
            }
        } else if s.contains("OIL") || s.contains("WTI") || s.contains("BRENT") || s.contains("CRUDE") || s.contains("XPT") || s.contains("XPD") {
            AssetProperties {
                asset_type: AssetType::Commodity,
                pip_value: 0.01, // Pétrole/Platine: souvent 0.01
                unit: "pts".to_string(),
                display_digits: 2,
            }
        } else if s.contains("NGAS") {
            AssetProperties {
                asset_type: AssetType::Commodity,
                pip_value: 0.001, // Gaz Naturel: souvent 0.001
                unit: "pts".to_string(),
                display_digits: 3,
            }
        } else if s.contains("IDX") || s.contains("US30") || s.contains("DAX") || s.contains("NAS")
            || s.contains("GER") || s.contains("SPX") || s.contains("US100") || s.contains("US500")
            || s.contains("FRA40") || s.contains("UK100") || s.contains("EUSTX") || s.contains("JPN225")
            || s.contains("USATEC") || s.contains("USAIDX") || s.contains("DEUIDX") || s.contains("USTEC")
            || s.contains("US500") || s.contains("US30") || s.contains("HK50") || s.contains("FR40")
            || s.contains("GR30") || s.contains("DE40") || s.contains("WS30") || s.contains("NDX")
            || s.contains("VIX") || s.contains("DXY") || s.contains("STOXX") || s.contains("CAC")
            || s.contains("FTSE") || s.contains("NI225") || s.contains("ASX") || s.contains("HSI")
        {
            AssetProperties {
                asset_type: AssetType::Index,
                pip_value: 1.0, // Indices: on parle en points
                unit: "pts".to_string(),
                display_digits: 1,
            }
        } else if s.contains("HUF") || s.contains("CZK") {
            // Forex Exotique (souvent 2 décimales comme JPY)
            AssetProperties {
                asset_type: AssetType::ForexJpy, // On utilise ForexJpy pour la logique de pip (0.01)
                pip_value: 0.01,
                unit: "pips".to_string(),
                display_digits: 2,
            }
        } else {
            // Par défaut: Forex Major (5 digits)
            AssetProperties {
                asset_type: AssetType::ForexMajor,
                pip_value: 0.0001,
                unit: "pips".to_string(),
                display_digits: 1,
            }
        }
    }

    /// Convertit une valeur brute (prix) en pips/points standardisés
    pub fn normalize(&self, raw_value: f64) -> f64 {
        if raw_value == 0.0 {
            return 0.0;
        }
        raw_value / self.pip_value
    }

    /// Convertit des pips/points en valeur brute (prix)
    #[allow(dead_code)]
    pub fn denormalize(&self, pips: f64) -> f64 {
        pips * self.pip_value
    }
}

#[cfg(test)]
#[path = "asset_class_tests.rs"]
mod tests;

