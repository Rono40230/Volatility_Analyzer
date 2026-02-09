// services/pair_data/symbol_properties.rs
// Gestion des propriétés des symboles (valeur du point, pip, etc.)
// Supporte les overrides DB via set_pairs_pool()

use crate::db::DbPool;
use crate::models::AssetProperties;
use crate::services::pair_data::conversion_db;
use std::sync::OnceLock;

/// Pool global pour accéder aux conversions DB depuis n'importe quel service
static PAIRS_POOL: OnceLock<DbPool> = OnceLock::new();

/// Initialise le pool global (appelé une fois au démarrage dans lib.rs)
pub fn set_pairs_pool(pool: DbPool) {
    let _ = PAIRS_POOL.set(pool);
    tracing::info!("✅ Pool pairs.db enregistré pour symbol_properties");
}

/// Retourne les propriétés complètes d'un symbole (DB override > hardcodé)
pub fn get_asset_properties(symbol: &str) -> AssetProperties {
    let db_override = PAIRS_POOL
        .get()
        .and_then(|pool| conversion_db::get_conversion_for_symbol(pool, symbol).ok())
        .flatten();
    AssetProperties::from_symbol_with_override(symbol, db_override)
}

/// Retourne la valeur d'un point (Tick Size) pour un symbole donné
/// Pour le backtest, on veut le pip_value directement (pas le point MT5)
/// car l'utilisateur saisit ses paramètres en pips.
pub fn get_point_value(symbol: &str) -> f64 {
    get_asset_properties(symbol).pip_value
}

/// Retourne la valeur d'un Pip standard
#[allow(dead_code)]
pub fn get_pip_value(symbol: &str) -> f64 {
    get_asset_properties(symbol).pip_value
}

/// Normalise une valeur en pips selon le symbole
pub fn normalize_to_pips(value: f64, symbol: &str) -> f64 {
    get_asset_properties(symbol).normalize(value)
}

/// Détermine si c'est une paire Forex
#[allow(dead_code)]
fn is_forex_pair(symbol: &str) -> bool {
    let s = symbol.to_uppercase();
    let currencies = ["EUR", "USD", "GBP", "JPY", "CAD", "AUD", "NZD", "CHF"];
    let count = currencies.iter().filter(|c| s.contains(*c)).count();
    count >= 2
}
