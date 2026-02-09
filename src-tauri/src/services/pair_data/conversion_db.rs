// services/pair_data/conversion_db.rs - CRUD pour symbol_conversions (DB layer)
// Conforme .clinerules : < 300 lines, retourne Result<T, VolatilityError>

use crate::db::DbPool;
use crate::models::symbol_conversion::SymbolConversion;
use crate::models::VolatilityError;
use diesel::prelude::*;

/// Récupère toutes les conversions personnalisées
pub fn get_all_conversions(pool: &DbPool) -> Result<Vec<SymbolConversion>, VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let rows: Vec<SymbolConversionRow> = diesel::sql_query(
        "SELECT symbol, pip_value, unit, display_digits, mt5_digits FROM symbol_conversions ORDER BY symbol",
    )
    .load::<SymbolConversionRow>(&mut conn)
    .map_err(|e| VolatilityError::DatabaseError(format!("Query failed: {}", e)))?;

    Ok(rows
        .into_iter()
        .map(|r| SymbolConversion {
            symbol: r.symbol,
            pip_value: r.pip_value,
            unit: r.unit,
            display_digits: r.display_digits,
            mt5_digits: r.mt5_digits,
        })
        .collect())
}

/// Récupère la conversion pour un symbole (si override existe)
pub fn get_conversion_for_symbol(
    pool: &DbPool,
    symbol: &str,
) -> Result<Option<SymbolConversion>, VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let upper = symbol.to_uppercase();
    let results: Vec<SymbolConversionRow> = diesel::sql_query(
        "SELECT symbol, pip_value, unit, display_digits, mt5_digits FROM symbol_conversions WHERE symbol = ?",
    )
    .bind::<diesel::sql_types::Text, _>(&upper)
    .load(&mut conn)
    .map_err(|e| VolatilityError::DatabaseError(format!("Query failed: {}", e)))?;

    Ok(results.into_iter().next().map(|r| SymbolConversion {
        symbol: r.symbol,
        pip_value: r.pip_value,
        unit: r.unit,
        display_digits: r.display_digits,
        mt5_digits: r.mt5_digits,
    }))
}

/// Insère ou met à jour une conversion (UPSERT)
pub fn upsert_conversion(
    pool: &DbPool,
    conversion: &SymbolConversion,
) -> Result<(), VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let upper = conversion.symbol.to_uppercase();
    diesel::sql_query(
        "INSERT INTO symbol_conversions (symbol, pip_value, unit, display_digits, mt5_digits, updated_at) 
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(symbol) DO UPDATE SET 
           pip_value = excluded.pip_value,
           unit = excluded.unit,
           display_digits = excluded.display_digits,
           mt5_digits = excluded.mt5_digits,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind::<diesel::sql_types::Text, _>(&upper)
    .bind::<diesel::sql_types::Double, _>(conversion.pip_value)
    .bind::<diesel::sql_types::Text, _>(&conversion.unit)
    .bind::<diesel::sql_types::Integer, _>(conversion.display_digits)
    .bind::<diesel::sql_types::Integer, _>(conversion.mt5_digits)
    .execute(&mut conn)
    .map_err(|e| VolatilityError::DatabaseError(format!("Upsert failed: {}", e)))?;

    tracing::info!(
        "✅ Conversion sauvegardée: {} → pip_value={}, unit={}, digits={}, mt5_digits={}",
        upper,
        conversion.pip_value,
        conversion.unit,
        conversion.display_digits,
        conversion.mt5_digits,
    );

    Ok(())
}

/// Supprime une conversion personnalisée (retourne au hardcodé)
pub fn delete_conversion(pool: &DbPool, symbol: &str) -> Result<(), VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let upper = symbol.to_uppercase();
    diesel::sql_query("DELETE FROM symbol_conversions WHERE symbol = ?")
        .bind::<diesel::sql_types::Text, _>(&upper)
        .execute(&mut conn)
        .map_err(|e| VolatilityError::DatabaseError(format!("Delete failed: {}", e)))?;

    tracing::info!("🗑️ Conversion supprimée: {}", upper);
    Ok(())
}

// Helper struct pour QueryableByName avec Diesel
use diesel::sql_types::{Double, Integer, Text};

#[derive(QueryableByName, Debug)]
struct SymbolConversionRow {
    #[diesel(sql_type = Text)]
    symbol: String,
    #[diesel(sql_type = Double)]
    pip_value: f64,
    #[diesel(sql_type = Text)]
    unit: String,
    #[diesel(sql_type = Integer)]
    display_digits: i32,
    #[diesel(sql_type = Integer)]
    mt5_digits: i32,
}
