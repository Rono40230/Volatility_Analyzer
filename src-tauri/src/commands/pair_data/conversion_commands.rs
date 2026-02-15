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
    pub is_custom: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
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
        "BTCUSD", "ETHUSD", "ETHUS0",
        "US30", "US39", "USATEC", "DEUIDX", "DEUIDS", "SPX500", "VIX",
        "WTI", "BRENT", "NGAS",
    ];

    let mut entries: Vec<ConversionEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for sym in &known_symbols {
        let symbol = sym.to_string();
        let props = AssetProperties::from_symbol(sym);
        
        if let Some(custom) = db_map.get(&symbol) {
            // ✅ FILTRER les conversions cachées
            if custom.hidden {
                // Paire cachée = ne pas la montrer, skip
                seen.insert(symbol);
                continue;
            }
            
            // Si l'unité est vide en DB, utiliser la valeur par défaut
            let unit = if custom.unit.trim().is_empty() {
                props.unit
            } else {
                custom.unit.clone()
            };

            entries.push(ConversionEntry {
                symbol: symbol.clone(),
                pip_value: custom.pip_value,
                unit,
                display_digits: custom.display_digits,
                is_custom: true,
                hidden: None,
            });
        } else {
            entries.push(ConversionEntry {
                symbol: symbol.clone(),
                pip_value: props.pip_value,
                unit: props.unit,
                display_digits: props.display_digits as i32,
                is_custom: false,
                hidden: None,
            });
        }
        seen.insert(symbol);
    }

    for (sym, custom) in &db_map {
        // ✅ Ne pas afficher les conversions cachées
        if custom.hidden {
            continue;
        }
        if !seen.contains(sym) {
            entries.push(ConversionEntry {
                symbol: sym.clone(),
                pip_value: custom.pip_value,
                unit: custom.unit.clone(),
                display_digits: custom.display_digits,
                is_custom: true,
                hidden: None,
            });
        }
    }

    // Log all conversions being returned
    for entry in &entries {
        eprintln!("[GET_ALL_CONVERSIONS] {} = {} (pip_value={})", entry.symbol, entry.unit, entry.pip_value);
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
) -> Result<(), String> {
    use crate::models::symbol_conversion::SymbolConversion;
    use crate::services::pair_data::conversion_db;

    eprintln!("[SAVE_CONVERSION] {} -> unit={}, pip_value={}", symbol, unit, pip_value);

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
        hidden: false,
    };

    conversion_db::upsert_conversion(&pool, &conversion)
        .map_err(|e| format!("Save failed: {}", e))
}

#[tauri::command]
pub fn delete_conversion(
    state: tauri::State<'_, PairDataState>,
    symbol: String,
) -> Result<(), String> {
    use crate::models::symbol_conversion::SymbolConversion;
    use crate::services::pair_data::conversion_db;

    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")?;
        pool_opt.clone().ok_or("DB pool not initialized")?
    };

    // 1️⃣ S'assurer que l'entrée existe en BD (sinon la créer avec defaults)
    match conversion_db::get_conversion_for_symbol(&pool, &symbol) {
        Ok(Some(_)) => {
            // Existe déjà en BD: marquer hidden=true
            conversion_db::hide_conversion(&pool, &symbol)
                .map_err(|e| format!("Hide failed: {}", e))
        }
        Ok(None) => {
            // N'existe pas en BD (paire hardcodée): créer l'entrée + marquer hidden
            use crate::models::AssetProperties;
            let props = AssetProperties::from_symbol(&symbol);
            let conversion = SymbolConversion {
                symbol: symbol.clone(),
                pip_value: props.pip_value,
                unit: props.unit,
                display_digits: props.display_digits as i32,
                hidden: true,  // ✅ Marquer HIDDEN immédiatement
            };
            conversion_db::upsert_conversion(&pool, &conversion)
                .map_err(|e| format!("Create hidden conversion failed: {}", e))
        }
        Err(e) => Err(format!("Failed to check existing conversion: {}", e)),
    }
}

#[tauri::command]
pub fn restore_conversion(
    state: tauri::State<'_, PairDataState>,
    symbol: String,
) -> Result<(), String> {
    use crate::services::pair_data::conversion_db;

    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")?;
        pool_opt.clone().ok_or("DB pool not initialized")?
    };

    conversion_db::unhide_conversion(&pool, &symbol)
        .map_err(|e| format!("Restore failed: {}", e))
}

#[tauri::command]
pub async fn invalidate_analysis_cache() -> Result<(), String> {
    tracing::info!("🗑️ Invalidating analysis cache after conversion changes");
    // Signal au frontend de purger son localStorage
    // (Frontend reçoit OK, puis clear les caches lui-même)
    // Pas d'action backend ici - juste un signal de synchronisation
    Ok(())
}