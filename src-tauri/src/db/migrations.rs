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
            source_file TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

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
