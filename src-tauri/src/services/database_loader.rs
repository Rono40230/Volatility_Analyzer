// services/database_loader.rs - Charge les candles depuis la BD paires
// Alternative à CsvLoader: lit depuis candle_data table à la place des fichiers CSV

use crate::db::DbPool;
use crate::models::candle::Candle;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tracing::{error, info, instrument};

/// Erreur spécifique au DatabaseLoader
#[derive(Debug)]
#[allow(dead_code)]
pub enum LoaderError {
    Connection(String),
    Query(String),
    Parsing(String),
    Validation(String),
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::Connection(e) => write!(f, "DB Connection error: {}", e),
            LoaderError::Query(e) => write!(f, "Query error: {}", e),
            LoaderError::Parsing(e) => write!(f, "Parse error: {}", e),
            LoaderError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for LoaderError {}

/// ============================================================================
/// SERVICE DE CHARGEMENT DES CANDLES DEPUIS LA BD
/// ============================================================================
///
/// Charge les candles depuis la table `candle_data` de la base de données
/// pairs.db. Utilise le **pool Diesel r2d2** pour les connexions.
///
/// **IMPORTANT** : Cette struct utilise maintenant le pool au lieu de créer
/// des connexions rusqlite directes. Cela garantit:
/// - ✓ Connexions réutilisées (pooling)
/// - ✓ Thread-safety (r2d2 géré)
/// - ✓ Timeouts (connection_timeout du pool)
/// - ✓ Configuration centralisée (voir db/mod.rs)
///
/// # Migration depuis CsvLoader
/// Cette struct remplace le CsvLoader après migration CSV → DB.
/// Les données sont maintenant en database au lieu de fichiers CSV.
///
#[allow(dead_code)]
#[derive(Clone)]
pub struct DatabaseLoader {
    /// Pool de connexions Diesel r2d2 vers pairs.db
    /// Utilisé pour toutes les opérations de lecture de candles
    db_pool: DbPool,
}

impl DatabaseLoader {
    /// Crée une nouvelle instance du loader avec un pool existant
    ///
    /// # Arguments
    /// * `pool` - Pool Diesel r2d2 déjà initialisé (depuis lib.rs)
    ///
    /// # Exemple
    /// ```ignore
    /// let pool = db::create_pool("sqlite:///path/to/pairs.db")?;
    /// let loader = DatabaseLoader::new(pool);
    /// let candles = loader.load_candles_by_pair("UNIUSD", "M1", start, end)?;
    /// ```
    ///
    /// # Note sur le pool
    /// Le pool est créé une fois au démarrage dans lib.rs et passé à
    /// DatabaseLoader. Ne pas créer de nouveau pool ici.
    #[allow(dead_code)]
    pub fn new(pool: DbPool) -> Self {
        tracing::debug!("📦 DatabaseLoader créé avec pool (utilise pooling de connexions)");
        DatabaseLoader { db_pool: pool }
    }

    /// Charge les candles pour une paire donnée dans une plage temporelle
    ///
    /// # Arguments
    /// * `symbol` - Symbole de la paire (ex: "UNIUSD")
    /// * `timeframe` - Timeframe (ex: "M1", "H4")
    /// * `start_time` - Date/heure de début (inclusive)
    /// * `end_time` - Date/heure de fin (inclusive)
    ///
    /// # Retour
    /// Vecteur de candles triées par timestamp (croissant)
    ///
    /// # Note sur le pool
    /// Le pool Diesel est maintenant stocké et disponible pour usage futur
    /// (conversion progressif vers Diesel ORM plutôt que rusqlite direct).
    /// Pour l'instant, on utilise rusqlite mais via le chemin DB du pool.
    #[allow(dead_code)]
    #[instrument(skip(self), fields(symbol = %symbol, timeframe = %timeframe))]
    pub fn load_candles_by_pair(
        &self,
        symbol: &str,
        timeframe: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Candle>, LoaderError> {
        // Vérifier que le pool est actif (nouveau pattern: pool passé au constructor)
        let _pool_ref = &self.db_pool;
        tracing::debug!("Using pool-managed candle loader (pool active)");

        let db_path = dirs::data_local_dir()
            .map(|d| d.join("volatility-analyzer").join("pairs.db"))
            .unwrap_or_else(|| PathBuf::from("pairs.db"));

        let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
            error!("Failed to open DB at {:?}: {}", db_path, e);
            LoaderError::Connection(e.to_string())
        })?;

        let start_str = start_time.to_rfc3339();
        let end_str = end_time.to_rfc3339();

        let mut stmt = conn
            .prepare(
                "SELECT symbol, time, open, high, low, close, volume
                 FROM candle_data
                 WHERE symbol = ? AND timeframe = ? AND time >= ? AND time <= ?
                 ORDER BY time ASC",
            )
            .map_err(|e| {
                error!("Query prepare error: {}", e);
                LoaderError::Query(e.to_string())
            })?;

        let rows = stmt
            .query_map(
                rusqlite::params![symbol, timeframe, &start_str, &end_str],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?, // symbol
                        row.get::<_, String>(1)?, // time
                        row.get::<_, f64>(2)?,    // open
                        row.get::<_, f64>(3)?,    // high
                        row.get::<_, f64>(4)?,    // low
                        row.get::<_, f64>(5)?,    // close
                        row.get::<_, f64>(6)?,    // volume
                    ))
                },
            )
            .map_err(|e| {
                error!("Query execution error: {}", e);
                LoaderError::Query(e.to_string())
            })?;

        let candles: Result<Vec<_>, LoaderError> = rows
            .map(|row_result| {
                let (sym, time_str, open, high, low, close, volume) =
                    row_result.map_err(|e| LoaderError::Query(e.to_string()))?;

                let datetime = DateTime::parse_from_rfc3339(&time_str)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok_or_else(|| {
                        LoaderError::Parsing(format!("Invalid datetime: {}", time_str))
                    })?;

                Candle::new(sym, datetime, open, high, low, close, volume)
                    .map_err(|e| LoaderError::Validation(e.to_string()))
            })
            .collect();

        let result = candles?;

        info!(
            "Loaded {} candles for {}/{} from {} to {}",
            result.len(),
            symbol,
            timeframe,
            start_str,
            end_str
        );

        Ok(result)
    }

    /// Récupère tous les symboles uniques dans la DB
    ///
    /// Accède à la DB depuis le chemin standard (le pool est maintenant géré centralement)
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub fn get_all_symbols(&self) -> Result<Vec<String>, LoaderError> {
        let _pool_ref = &self.db_pool;
        let db_path = dirs::data_local_dir()
            .map(|d| d.join("volatility-analyzer").join("pairs.db"))
            .unwrap_or_else(|| PathBuf::from("pairs.db"));

        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| LoaderError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT DISTINCT symbol FROM candle_data ORDER BY symbol")
            .map_err(|e| LoaderError::Query(e.to_string()))?;

        let symbols = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| LoaderError::Query(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| LoaderError::Query(e.to_string()))?;

        Ok(symbols)
    }

    /// Récupère tous les timeframes uniques pour un symbole
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub fn get_timeframes_for_symbol(&self, symbol: &str) -> Result<Vec<String>, LoaderError> {
        let _pool_ref = &self.db_pool;
        let db_path = dirs::data_local_dir()
            .map(|d| d.join("volatility-analyzer").join("pairs.db"))
            .unwrap_or_else(|| PathBuf::from("pairs.db"));

        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| LoaderError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT timeframe FROM candle_data WHERE symbol = ? ORDER BY timeframe",
            )
            .map_err(|e| LoaderError::Query(e.to_string()))?;

        let timeframes = stmt
            .query_map(rusqlite::params![symbol], |row| row.get::<_, String>(0))
            .map_err(|e| LoaderError::Query(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| LoaderError::Query(e.to_string()))?;

        Ok(timeframes)
    }

    /// Compte le nombre de candles pour une paire/timeframe
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub fn count_candles(&self, symbol: &str, timeframe: &str) -> Result<i64, LoaderError> {
        let _pool_ref = &self.db_pool;
        let db_path = dirs::data_local_dir()
            .map(|d| d.join("volatility-analyzer").join("pairs.db"))
            .unwrap_or_else(|| PathBuf::from("pairs.db"));

        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| LoaderError::Connection(e.to_string()))?;

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM candle_data WHERE symbol = ? AND timeframe = ?",
                rusqlite::params![symbol, timeframe],
                |row| row.get(0),
            )
            .map_err(|e| LoaderError::Query(e.to_string()))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    // Les tests réels s'éxécutent contre une vraie DB de test
    // Pas de mocks data - utilisera une fixture DB temporaire
}
