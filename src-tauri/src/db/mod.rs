// db/mod.rs - Configuration Diesel SQLite

pub mod migrations;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::sync::Arc;

pub type DbPool = Arc<r2d2::Pool<ConnectionManager<SqliteConnection>>>;

pub use migrations::{ensure_calendar_imports_table, ensure_calendar_table, ensure_pair_tables};

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
        .build(manager)?;

    tracing::debug!("✅ Pool créé: max_size=5, min_idle=1, timeout=5s, idle=60s");

    Ok(Arc::new(pool))
}
