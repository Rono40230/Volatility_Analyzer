// commands/pair_data/conversion_commands.rs - Commandes Tauri pour les conversions
// Conforme .clinerules : < 200 lines

use super::PairDataState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionEntry {
    pub symbol: String,
    pub pip_value: f64,
    pub unit: String,
    pub display_digits: i32,
    pub mt5_digits: i32,
    pub is_custom: bool,
}

#[tauri::command]
pub fn get_all_conversions(
    state: tauri::State<'_, PairDataState>,
) -> Result<Vec<ConversionEntry>, String> {
    use crate::models::AssetProperties;
    use crate::services::pair_data::conversion_db;

    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")?;
        pool_opt.clone().ok_or("DB pool not initialized")?
    };

    let db_conversions = conversion_db::get_all_conversions(&pool)
        .map_err(|e| format!("DB error: {}", e))?;

    let db_map: std::collections::HashMap<String, _> = db_conversions
        .into_iter()
        .map(|c| (c.symbol.clone(), c))
        .collect();

    let known_symbols = [
        "EURUSD", "GBPUSD", "USDCAD", "USDCHF", "AUDUSD", "NZDUSD",
        "USDJPY", "GBPJPY", "CADJPY", "EURJPY", "AUDJPY",
        "XAUUSD", "XAGUSD",
        "BTCUSD", "ETHUSD",
        "US30", "USATEC", "DEUIDX", "SPX500",
        "WTI", "BRENT", "NGAS",
    ];

    let mut entries: Vec<ConversionEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for sym in &known_symbols {
        let symbol = sym.to_string();
        if let Some(custom) = db_map.get(&symbol) {
            entries.push(ConversionEntry {
                symbol: symbol.clone(),
                pip_value: custom.pip_value,
                unit: custom.unit.clone(),
                display_digits: custom.display_digits,
                mt5_digits: custom.mt5_digits,
                is_custom: true,
            });
        } else {
            let props = AssetProperties::from_symbol(sym);
            entries.push(ConversionEntry {
                symbol: symbol.clone(),
                pip_value: props.pip_value,
                unit: props.unit,
                display_digits: props.display_digits as i32,
                mt5_digits: props.mt5_digits as i32,
                is_custom: false,
            });
        }
        seen.insert(symbol);
    }

    for (sym, custom) in &db_map {
        if !seen.contains(sym) {
            entries.push(ConversionEntry {
                symbol: sym.clone(),
                pip_value: custom.pip_value,
                unit: custom.unit.clone(),
                display_digits: custom.display_digits,
                mt5_digits: custom.mt5_digits,
                is_custom: true,
            });
        }
    }

    Ok(entries)
}

#[tauri::command]
pub fn save_conversion(
    state: tauri::State<'_, PairDataState>,
    symbol: String,
    pip_value: f64,
    unit: String,
    display_digits: i32,
    mt5_digits: i32,
) -> Result<(), String> {
    use crate::models::symbol_conversion::SymbolConversion;
    use crate::services::pair_data::conversion_db;

    if symbol.is_empty() {
        return Err("Symbol cannot be empty".to_string());
    }
    if pip_value <= 0.0 {
        return Err("pip_value must be positive".to_string());
    }

    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")?;
        pool_opt.clone().ok_or("DB pool not initialized")?
    };

    let conversion = SymbolConversion {
        symbol: symbol.to_uppercase(),
        pip_value,
        unit,
        display_digits,
        mt5_digits,
    };

    conversion_db::upsert_conversion(&pool, &conversion)
        .map_err(|e| format!("Save failed: {}", e))
}

#[tauri::command]
pub fn delete_conversion(
    state: tauri::State<'_, PairDataState>,
    symbol: String,
) -> Result<(), String> {
    use crate::services::pair_data::conversion_db;

    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")?;
        pool_opt.clone().ok_or("DB pool not initialized")?
    };

    conversion_db::delete_conversion(&pool, &symbol)
        .map_err(|e| format!("Delete failed: {}", e))
}
