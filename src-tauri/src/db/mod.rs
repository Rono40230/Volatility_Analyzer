// db/mod.rs - Configuration Diesel SQLite

pub mod migrations;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::sync::Arc;

pub type DbPool = Arc<r2d2::Pool<ConnectionManager<SqliteConnection>>>;

pub use migrations::{ensure_calendar_imports_table, ensure_calendar_table, ensure_pair_tables, ensure_archives_table, ensure_symbol_conversions_table, ensure_volatility_profiles_table};

/// Initialise un pool de connexions SQLite optimisé
///
/// Configuration:
/// - max_size(5): 5 connexions simultanées
/// - min_idle(1): garder 1 connexion ready
/// - timeout(5s): attendre max 5s une connexion
/// - idle_timeout(60s): recycler après 60s d'inactivité
pub fn create_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(Some(std::time::Duration::from_secs(60)))
        .connection_customizer(Box::new(ConnectionOptions)) // Enable WAL
        .build(manager)?;

    tracing::debug!("✅ Pool créé: max_size=5, min_idle=1, timeout=5s, idle=60s, WAL=on");

    Ok(Arc::new(pool))
}

#[derive(Debug)]
struct ConnectionOptions;

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::connection::SimpleConnection;
        // Note: PRAGMA journal_mode = WAL must be set once globally, not on every connection acquire if others are open.
        // We assume WAL is set during pool initialization or app startup.
        conn.batch_execute("
            PRAGMA busy_timeout = 5000;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
        ").map_err(|e| {
            tracing::error!("❌ [Pool] Failed to set PRAGMAs: {}", e);
            diesel::r2d2::Error::QueryError(e)
        })?;
        Ok(())
    }
}

pub fn init_wal_mode(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::connection::SimpleConnection;
    let mut conn = SqliteConnection::establish(database_url)?;
    conn.batch_execute("
        PRAGMA journal_mode = WAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA synchronous = NORMAL;
    ")?;
    tracing::info!("✅ WAL mode enabled for {}", database_url);
    Ok(())
}
