// models/asset_class.rs - Détection et normalisation des actifs
// Conforme .clinerules : < 150L

use serde::{Deserialize, Serialize};
use crate::models::symbol_conversion::SymbolConversion;

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
                display_digits: 2,
            }
        } else if s.contains("XAU") || s.contains("GOLD") {
            AssetProperties {
                asset_type: AssetType::Gold,
                pip_value: 0.01, // ✓ Correction: was 0.1, should be 0.01
                unit: "$".to_string(),
                display_digits: 2,
            }
        } else if s.contains("XAG") || s.contains("SILVER") {
            AssetProperties {
                asset_type: AssetType::Silver,
                pip_value: 0.01, // Convention Silver
                unit: "$".to_string(),
                display_digits: 2,
            }
        } else if s.contains("BTC") || s.contains("ETH") || s.contains("CRYPTO") || s.contains("SOL") || s.contains("BNB") || s.contains("XRP") || s.contains("ADA") || s.contains("DOT")
            || s.contains("LTC") || s.contains("BCH") || s.contains("DOGE") || s.contains("SHIB") || s.contains("LINK") || s.contains("MATIC") || s.contains("AVAX") || s.contains("UNI") || s.contains("XLM") || s.contains("TRX") || s.contains("ATOM") || s.contains("NEAR") || s.contains("PEPE")
        {
            AssetProperties {
                asset_type: AssetType::Crypto,
                pip_value: 1.0, // Crypto: 1$ = 1 pip = 1 point
                unit: "$".to_string(),
                display_digits: 0,
            }
        } else if s.contains("OIL") || s.contains("WTI") || s.contains("BRENT") || s.contains("CRUDE") || s.contains("XPT") || s.contains("XPD") {
            AssetProperties {
                asset_type: AssetType::Commodity,
                pip_value: 0.01,
                unit: "$".to_string(),
                display_digits: 2,
            }
        } else if s.contains("NGAS") {
            AssetProperties {
                asset_type: AssetType::Commodity,
                pip_value: 0.001,
                unit: "$".to_string(),
                display_digits: 3,
            }
        } else if s.contains("VIX") || s.contains("IDX") || (s.starts_with("US") && (s.contains("30") || s.contains("39") || s.contains("ATEC") || s.contains("TECH")))
            || s.contains("DAX") || s.contains("SPX500") || s.contains("GER") || s.contains("FRA") || s.contains("UK")
            || s.contains("NAS") || s.contains("EUSTX") || s.contains("JPN") || s.contains("DEUR") || s.contains("USTEC")
            || s.contains("HK") || s.contains("GR") || s.contains("DE40") || s.contains("WS")
            || s.contains("NDX") || s.contains("DXY") || s.contains("STOXX") || s.contains("CAC") || s.contains("FTSE") || s.contains("ASX") || s.contains("HSI")
        {
            AssetProperties {
                asset_type: AssetType::Index,
                pip_value: 1.0,
                unit: "points".to_string(),
                display_digits: 0,
            }
        } else if s.contains("HUF") || s.contains("CZK") {
            AssetProperties {
                asset_type: AssetType::ForexJpy,
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
                display_digits: 4,
            }
        }
    }

    /// Crée les propriétés depuis un override DB (si fourni), sinon fallback hardcodé
    /// Ignore l'override si pip_value est invalide (<= 0.0) ou trop éloigné du hardcodé (ratio > 100x)
    pub fn from_symbol_with_override(symbol: &str, db_override: Option<SymbolConversion>) -> Self {
        let base = Self::from_symbol(symbol);
        match db_override {
            Some(conv) if conv.pip_value > 0.0 => {
                // Protection: rejeter les overrides aberrants (ratio > 100x du hardcodé)
                let ratio = if base.pip_value > 0.0 {
                    (conv.pip_value / base.pip_value).max(base.pip_value / conv.pip_value)
                } else {
                    1.0
                };
                if ratio > 100.0 {
                    tracing::warn!(
                        "⚠️ Override DB ignoré pour {} : pip_value={} vs hardcodé={} (ratio {:.0}x)",
                        symbol, conv.pip_value, base.pip_value, ratio
                    );
                    return base;
                }
                AssetProperties {
                    asset_type: base.asset_type,
                    pip_value: conv.pip_value,
                    unit: conv.unit,
                    display_digits: conv.display_digits as usize,
                }
            }
            _ => base,
        }
    }

    /// Convertit une valeur brute (prix) en pips/points standardisés
    /// Retourne 0.0 si pip_value est invalide (protection division par zéro)
    pub fn normalize(&self, raw_value: f64) -> f64 {
        if raw_value == 0.0 || self.pip_value <= 0.0 {
            return 0.0;
        }
        raw_value / self.pip_value
    }

    /// Convertit des pips/points en valeur brute (prix)
    #[allow(dead_code)]
    pub fn denormalize(&self, pips: f64) -> f64 {
        pips * self.pip_value
    }

    /// Facteur de mise à l'échelle des seuils ATR/Range par classe d'actif.
    /// Référence = ForexMajor (1.0). Gold/Indices ont des ATR bruts bien plus grands.
    /// Usage : `seuil_atr * props.atr_scaling_factor()` dans les fonctions de scoring.
    #[allow(dead_code)] // Utilisé par scaled_atr_* quand l'appelant a AssetProperties
    pub fn atr_scaling_factor(&self) -> f64 {
        match self.asset_type {
            AssetType::ForexMajor => 1.0,   // ATR ~1-3 pips, référence
            AssetType::ForexJpy => 1.0,     // ATR ~1-3 pips (après normalize)
            AssetType::Gold => 5.0,         // ATR ~5-15 pips gold
            AssetType::Silver => 3.0,       // ATR ~3-10 pips silver
            AssetType::Crypto => 50.0,      // ATR ~50-200 pips BTC
            AssetType::Index => 20.0,       // ATR ~20-80 pips indices
            AssetType::Commodity => 3.0,    // ATR ~3-10 pips WTI
            AssetType::Unknown => 1.0,
        }
    }
}

#[cfg(test)]
#[path = "asset_class_tests.rs"]
mod tests;

