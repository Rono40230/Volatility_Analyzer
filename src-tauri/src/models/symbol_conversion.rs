// models/symbol_conversion.rs - Modèle pour les conversions personnalisées
// Conforme .clinerules : < 150L

use serde::{Deserialize, Serialize};

/// Conversion personnalisée stockée en DB (overrides les valeurs hardcodées)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolConversion {
    pub symbol: String,
    pub pip_value: f64,
    pub unit: String,
    pub display_digits: i32,
    #[serde(default)]
    pub hidden: bool,
}
