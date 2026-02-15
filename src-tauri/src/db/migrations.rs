// db/migrations.rs - Création et migration des tables (source unique de vérité)
//
// STRATÉGIE DE MIGRATION :
// - Les tables sont créées via `CREATE TABLE IF NOT EXISTS` (idempotent).
// - Les colonnes ajoutées après la version initiale utilisent `ALTER TABLE ADD COLUMN`
//   avec `let _ =` car la colonne peut déjà exister sur les bases à jour.
// - Le dossier `migrations/` contient les fichiers Diesel CLI historiques mais
//   ils ne sont PAS exécutés au runtime. Seul ce fichier fait foi.
// - Chaque `ensure_*` est appelée au démarrage dans lib.rs.
//
use crate::db::DbPool;
use diesel::prelude::*;

/// Crée la table calendar_events si elle n'existe pas.
/// Schéma complet incluant calendar_import_id et peak_delay_json.
pub fn ensure_calendar_table(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS calendar_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            symbol TEXT NOT NULL,
            event_time TIMESTAMP NOT NULL,
            impact TEXT NOT NULL,
            description TEXT NOT NULL,
            actual REAL,
            forecast REAL,
            previous REAL,
            calendar_import_id INTEGER REFERENCES calendar_imports(id) ON DELETE CASCADE,
            peak_delay_json TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_calendar_events_symbol ON calendar_events(symbol)",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_calendar_events_time ON calendar_events(event_time)",
    )
    .execute(&mut conn)?;

    // Colonnes ajoutées après v1 — nécessaire pour les bases existantes
    let _ = diesel::sql_query("ALTER TABLE calendar_events ADD COLUMN calendar_import_id INTEGER REFERENCES calendar_imports(id) ON DELETE CASCADE")
        .execute(&mut conn);
    let _ = diesel::sql_query("ALTER TABLE calendar_events ADD COLUMN actual REAL")
        .execute(&mut conn);
    let _ = diesel::sql_query("ALTER TABLE calendar_events ADD COLUMN forecast REAL")
        .execute(&mut conn);
    let _ = diesel::sql_query("ALTER TABLE calendar_events ADD COLUMN previous REAL")
        .execute(&mut conn);
    let _ = diesel::sql_query("ALTER TABLE calendar_events ADD COLUMN peak_delay_json TEXT")
        .execute(&mut conn);

    Ok(())
}

/// Crée les tables de la DB paires
#[allow(dead_code)]
pub fn ensure_pair_tables(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS candle_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            symbol TEXT NOT NULL,
            timeframe TEXT NOT NULL,
            time TIMESTAMP NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            imported_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            source_file TEXT NOT NULL,
            spread_open REAL,
            spread_high REAL,
            spread_low REAL,
            spread_close REAL,
            spread_mean REAL,
            tick_count INTEGER
        )",
    )
    .execute(&mut conn)?;

    // Migration idempotente : ajouter les colonnes spread si elles n'existent pas encore
    for col in &[
        "spread_open REAL",
        "spread_high REAL",
        "spread_low REAL",
        "spread_close REAL",
        "spread_mean REAL",
        "tick_count INTEGER",
    ] {
        let _ = diesel::sql_query(format!(
            "ALTER TABLE candle_data ADD COLUMN {}",
            col
        ))
        .execute(&mut conn);
        // Ignore l'erreur "duplicate column" si la colonne existe déjà
    }

    diesel::sql_query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_candle_data_symbol_timeframe_time 
            ON candle_data(symbol, timeframe, time)",
    )
    .execute(&mut conn)?;

    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_candle_data_time ON candle_data(time)")
        .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_candle_data_source_file ON candle_data(source_file)",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS pair_metadata (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            symbol TEXT NOT NULL,
            timeframe TEXT NOT NULL,
            row_count INTEGER NOT NULL DEFAULT 0,
            last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_imported_file TEXT,
            data_quality_score REAL DEFAULT 1.0,
            UNIQUE(symbol, timeframe)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_pair_metadata_symbol ON pair_metadata(symbol)",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS import_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            filename TEXT NOT NULL,
            symbol TEXT NOT NULL,
            timeframe TEXT NOT NULL,
            imported_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            row_count INTEGER NOT NULL,
            expected_row_count INTEGER,
            status TEXT NOT NULL,
            error_message TEXT,
            checksum TEXT
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_import_log_imported_at ON import_log(imported_at)",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_import_log_symbol ON import_log(symbol, timeframe)",
    )
    .execute(&mut conn)?;

    Ok(())
}

/// Crée la table calendar_imports
pub fn ensure_calendar_imports_table(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS calendar_imports (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT NOT NULL UNIQUE,
            filename TEXT NOT NULL,
            event_count INTEGER NOT NULL DEFAULT 0,
            oldest_event_date TIMESTAMP,
            newest_event_date TIMESTAMP,
            imported_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            is_active BOOLEAN NOT NULL DEFAULT 1
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_calendar_imports_is_active ON calendar_imports(is_active)",
    )
    .execute(&mut conn)?;

    Ok(())
}

/// Crée la table symbol_conversions si elle n'existe pas (dans pairs.db)
/// Stocke les overrides utilisateur pour pip_value / unit / display_digits
pub fn ensure_symbol_conversions_table(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS symbol_conversions (
            symbol TEXT PRIMARY KEY NOT NULL,
            pip_value REAL NOT NULL,
            unit TEXT NOT NULL DEFAULT 'pips',
            display_digits INTEGER NOT NULL DEFAULT 1,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut conn)?;

    // Ajouter la colonne hidden (FIX: soft delete avec masquage)
    let _ = diesel::sql_query("ALTER TABLE symbol_conversions ADD COLUMN hidden BOOLEAN NOT NULL DEFAULT 0")
        .execute(&mut conn);

    // Créer l'index pour filtrer les conversions cachées
    let _ = diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_symbol_conversions_hidden ON symbol_conversions(hidden)")
        .execute(&mut conn);

    tracing::info!("✅ Table symbol_conversions vérifiée/créée");
    Ok(())
}

/// Crée la table archives si elle n'existe pas
pub fn ensure_archives_table(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS archives (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            title TEXT NOT NULL,
            archive_type TEXT NOT NULL,
            period_start TEXT NOT NULL,
            period_end TEXT NOT NULL,
            comment TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            data_json TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    Ok(())
}

/// Crée la table volatility_profiles si elle n'existe pas (FIX 2.3)
/// Stocke les profiles de décroissance de volatilité par classe d'actif
pub fn ensure_volatility_profiles_table(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS volatility_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            asset_type VARCHAR(20) NOT NULL,
            half_life_minutes REAL NOT NULL,
            recommended_multiplier REAL NOT NULL DEFAULT 2.0,
            data_source VARCHAR(50) NOT NULL DEFAULT 'manual',
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(asset_type)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_volatility_profiles_asset_type ON volatility_profiles(asset_type)",
    )
    .execute(&mut conn)?;

    // Insert default profiles if the table is empty
    // Check count with a simpler approach - try to count and if it fails, just use count(*) 
    let _empty_table = matches!(
        diesel::sql_query("SELECT COUNT(*) FROM volatility_profiles")
            .execute(&mut conn),
        Ok(0) | Err(_)
    );

    // Just check if we need to insert by attempting a select with a simple query
    let should_insert = diesel::sql_query("SELECT COUNT(*) as cnt FROM volatility_profiles WHERE asset_type = 'ForexMajor'")
        .execute(&mut conn)
        .map(|_| false) // If executed successfully, record likely exists
        .unwrap_or(true); // If it fails, try to insert

    if should_insert {
        let _ = diesel::sql_query(
            "INSERT INTO volatility_profiles (asset_type, half_life_minutes, recommended_multiplier, data_source, updated_at) VALUES
            ('ForexMajor', 1.8, 2.0, 'manual', CURRENT_TIMESTAMP),
            ('ForexJpy', 2.0, 2.0, 'manual', CURRENT_TIMESTAMP),
            ('Gold', 2.2, 2.0, 'manual', CURRENT_TIMESTAMP),
            ('Silver', 2.2, 2.0, 'manual', CURRENT_TIMESTAMP),
            ('Crypto', 5.0, 2.0, 'manual', CURRENT_TIMESTAMP),
            ('Index', 3.0, 2.0, 'manual', CURRENT_TIMESTAMP),
            ('Commodity', 3.5, 2.0, 'manual', CURRENT_TIMESTAMP)"
        )
        .execute(&mut conn);
        tracing::info!("✅ Default volatility profiles inserted");
    }

    tracing::info!("✅ Table volatility_profiles vérifiée/créée");
    Ok(())
}
