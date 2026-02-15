// services/pair_data/conversion_db.rs - CRUD pour symbol_conversions (DB layer)
// Conforme .clinerules : < 300 lines, retourne Result<T, VolatilityError>

use crate::db::DbPool;
use crate::models::symbol_conversion::SymbolConversion;
use crate::models::VolatilityError;
use diesel::prelude::*;
use diesel::sql_types::{Double, Integer, Text, Bool};

/// Récupère TOUTES les conversions (y compris cachées) pour traitement en amont
pub fn get_all_conversions(pool: &DbPool) -> Result<Vec<SymbolConversion>, VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let rows: Vec<SymbolConversionRow> = diesel::sql_query(
        "SELECT symbol, pip_value, unit, display_digits, COALESCE(hidden, 0) as hidden FROM symbol_conversions ORDER BY symbol",
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
            hidden: r.hidden,
        })
        .collect())
}

/// Récupère la conversion pour un symbole (si override existe et non caché)
pub fn get_conversion_for_symbol(
    pool: &DbPool,
    symbol: &str,
) -> Result<Option<SymbolConversion>, VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let upper = symbol.to_uppercase();
    let results: Vec<SymbolConversionRow> = diesel::sql_query(
        "SELECT symbol, pip_value, unit, display_digits, COALESCE(hidden, 0) as hidden FROM symbol_conversions WHERE symbol = ? AND COALESCE(hidden, 0) = 0",
    )
    .bind::<Text, _>(&upper)
    .load(&mut conn)
    .map_err(|e| VolatilityError::DatabaseError(format!("Query failed: {}", e)))?;

    Ok(results.into_iter().next().map(|r| SymbolConversion {
        symbol: r.symbol,
        pip_value: r.pip_value,
        unit: r.unit,
        display_digits: r.display_digits,
        hidden: r.hidden,
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
        "INSERT INTO symbol_conversions (symbol, pip_value, unit, display_digits, hidden, updated_at) 
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(symbol) DO UPDATE SET 
           pip_value = excluded.pip_value,
           unit = excluded.unit,
           display_digits = excluded.display_digits,
           hidden = excluded.hidden,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind::<Text, _>(&upper)
    .bind::<Double, _>(conversion.pip_value)
    .bind::<Text, _>(&conversion.unit)
    .bind::<Integer, _>(conversion.display_digits)
    .bind::<Bool, _>(conversion.hidden)
    .execute(&mut conn)
    .map_err(|e| VolatilityError::DatabaseError(format!("Upsert failed: {}", e)))?;

    tracing::info!(
        "✅ Conversion sauvegardée: {} → pip_value={}, unit={}, digits={}, hidden={}",
        upper,
        conversion.pip_value,
        conversion.unit,
        conversion.display_digits,
        conversion.hidden,
    );

    Ok(())
}

/// Cache une conversion (marque hidden=true au lieu de DELETE)
pub fn hide_conversion(pool: &DbPool, symbol: &str) -> Result<(), VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let upper = symbol.to_uppercase();
    diesel::sql_query("UPDATE symbol_conversions SET hidden = 1, updated_at = CURRENT_TIMESTAMP WHERE symbol = ?")
        .bind::<Text, _>(&upper)
        .execute(&mut conn)
        .map_err(|e| VolatilityError::DatabaseError(format!("Hide failed: {}", e)))?;

    tracing::info!("👁️ Conversion masquée: {}", upper);
    Ok(())
}

/// Restaure une conversion (remet hidden=false)
pub fn unhide_conversion(pool: &DbPool, symbol: &str) -> Result<(), VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let upper = symbol.to_uppercase();
    diesel::sql_query("UPDATE symbol_conversions SET hidden = 0, updated_at = CURRENT_TIMESTAMP WHERE symbol = ?")
        .bind::<Text, _>(&upper)
        .execute(&mut conn)
        .map_err(|e| VolatilityError::DatabaseError(format!("Unhide failed: {}", e)))?;

    tracing::info!("👁️ Conversion restaurée: {}", upper);
    Ok(())
}

/// Supprime définitivement une conversion personnalisée
pub fn delete_conversion(pool: &DbPool, symbol: &str) -> Result<(), VolatilityError> {
    let mut conn = pool.get().map_err(|e| {
        VolatilityError::DatabaseError(format!("Pool connection failed: {}", e))
    })?;

    let upper = symbol.to_uppercase();
    diesel::sql_query("DELETE FROM symbol_conversions WHERE symbol = ?")
        .bind::<Text, _>(&upper)
        .execute(&mut conn)
        .map_err(|e| VolatilityError::DatabaseError(format!("Delete failed: {}", e)))?;

    tracing::info!("🗑️ Conversion supprimée définitivement: {}", upper);
    Ok(())
}

// Helper struct pour QueryableByName avec Diesel
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
    #[diesel(sql_type = Bool)]
    hidden: bool,
}
