// db/mod.rs - Configuration Diesel SQLite

pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::sync::Arc;

pub type DbPool = Arc<r2d2::Pool<ConnectionManager<SqliteConnection>>>;

/// ============================================================================
/// CONFIGURATION DU POOL DE CONNEXIONS DIESEL R2D2
/// ============================================================================
///
/// Cette fonction initialise un pool de connexions SQLite optimisé pour
/// l'application Volatility Analyzer. Les paramètres sont choisis pour:
///
/// 1. **max_size(5)** - 5 connexions maximum
///    - Représente le nombre de connexions SQLite ouvertes en même temps
///    - 2 pools × 5 connexions = 10 connexions au maximum sur la machine
///    - SQLite peut supporter plusieurs lecteurs simultanés mais 1 seul writer
///    - 5 est un bon compromis pour une app avec peu d'écritures simultanées
///
/// 2. **min_idle(Some(1))** - Garder 1 connexion idle toujours prête
///    - Réduit la latence du premier appel (pas d'attente de création)
///    - Prévient le démarrage "slow" si toutes les connexions sont utilisées
///    - Impact mémoire minimal (1 seule connexion SQLite par pool)
///
/// 3. **connection_timeout(5s)** - Attendre maximum 5 secondes une connexion
///    - Si le pool est saturé, attend 5s avant d'errorer
///    - Prévient les hangs infinis en cas de connexion "leakée"
///    - 5s = timeout raisonnable pour une app UI (pas trop court, pas trop long)
///
/// 4. **idle_timeout(Some(60s))** - Fermer connexions inutilisées après 60s
///    - Recycler les connexions périmées ou avec état inconsistant
///    - Réduit la mémoire sur longue durée sans impact perf (min_idle relance)
///    - Important pour les connexions SQLite qui peuvent accumuler locks
///
/// 5. **connection_test_on_checkout(true)** - Tester chaque connexion avant usage
///    - Vérifie que la connexion est valide avant la retourner
///    - Détecte les connexions fermées/périmées du côté DB
///    - Impact perf minimal (test = 1 query simple)
///
/// # Architecture Pool
///
/// L'app crée 2 pools séparés (chacun avec ces paramètres) :
/// - **Calendar Pool** (volatility.db) : lecture/écriture événements économiques
/// - **Pairs Pool** (pairs.db) : lecture candles, métadonnées paires
///
/// Avantages de 2 pools :
/// ✓ Isolation : un problème DB ne bloque pas l'autre
/// ✓ Scaling indépendant : ajuster tailles séparément selon usage
/// ✓ Failover : si un pool échoue, l'autre fonctionne
///
/// # Utilisation Correcte
///
/// CORRECT : Utiliser le pool via Diesel commands
/// ```ignore
/// let mut conn = calendar_pool.get()?;  // Acquiert une connexion du pool
/// let result = diesel::dsl::sql_query(sql).execute(&mut conn)?;
/// // conn relâché automatiquement quand scope se termine
/// ```
///
/// INCORRECT : Créer des connexions rusqlite directes
/// ```ignore
/// // NE PAS FAIRE - bypasse le pool:
/// let conn = rusqlite::Connection::open(db_path)?;
/// // ...
/// ```
///
/// # Monitoring (Future)
///
/// Pour monitorer le pool en production:
/// ```ignore
/// let state = pool.state();
/// tracing::info!("Connections: {} idle, {} used", state.idle_connections, state.connections)
/// ```
///
pub fn create_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(Some(std::time::Duration::from_secs(60)))
        .build(manager)?;
    
    tracing::debug!("✅ Pool créé avec config: max_size=5, min_idle=1, timeout=5s, idle=60s");
    
    Ok(Arc::new(pool))
}

/// ============================================================================
/// GUIDE D'UTILISATION DU POOL DE CONNEXIONS
/// ============================================================================
///
/// Cette section documente les patterns CORRECTS et INCORRECTS pour utiliser
/// le pool de connexions Diesel r2d2 dans l'application.
///
/// # Pattern CORRECT #1 : Utiliser le pool avec Diesel commands
///
/// ```ignore
/// #[tauri::command]
/// async fn my_command(state: State<'_, CalendarState>) -> Result<Data, String> {
///     let pool_guard = state.pool.lock().map_err(|_| "Lock failed")?;
///     let pool = pool_guard.as_ref().ok_or("Pool not initialized")?;
///
///     let mut conn = pool.get()
///         .map_err(|e| format!("Pool connection error: {}", e))?;
///
///     // Utiliser conn avec Diesel queries
///     let result = diesel::sql_query("SELECT ...").execute(&mut conn)?;
///
///     Ok(result)
/// }
/// ```
///
/// # Pattern CORRECT #2 : Stocker le pool dans une struct réutilisable
///
/// ```ignore
/// pub struct MyService {
///     pool: DbPool,
/// }
///
/// impl MyService {
///     pub fn new(pool: DbPool) -> Self {
///         MyService { pool }
///     }
///
///     pub fn query_data(&self) -> Result<Vec<Data>, Error> {
///         let mut conn = self.pool.get()?;
///         let result = diesel::sql_query("SELECT ...").load(&mut conn)?;
///         Ok(result)
///     }
/// }
/// ```
///
/// # Pattern INCORRECT #1 : Créer une nouvelle connexion à chaque fois
///
/// ```ignore
/// // ❌ NE PAS FAIRE - bypasse le pool, crée N connexions
/// pub fn query_data() -> Result<Vec<Data>, Error> {
///     let conn = rusqlite::Connection::open(db_path)?;  // MAUVAIS!
///     let result = conn.query_row(...)?;
///     Ok(result)
/// }
/// ```
///
/// # Pattern INCORRECT #2 : Oublier de retourner la connexion au pool
///
/// ```ignore
/// // ❌ NE PAS FAIRE - garde la connexion verrouillée trop longtemps
/// pub fn bad_pattern(state: State<'_, CalendarState>) {
///     let pool_guard = state.pool.lock().unwrap();
///     let pool = pool_guard.as_ref().unwrap();
///     let conn = pool.get().unwrap();
///     // ... fait du travail très long pendant longtemps ...
///     // Pool reste verrouillé, autres threads attendent!
/// }
/// ```
///
/// # Pattern CORRECT #3 : Brèves connexions (le meilleur)
///
/// ```ignore
/// pub fn quick_query(state: State<'_, CalendarState>) -> Result<Count, String> {
///     let pool_guard = state.pool.lock()
///         .map_err(|_| "Lock failed")?;
///     let pool = pool_guard.as_ref()
///         .ok_or("Pool not initialized")?;
///
///     // Scope étroit = connexion retournée au pool rapidement
///     let count = {
///         let mut conn = pool.get()
///             .map_err(|e| format!("Connection error: {}", e))?;
///         diesel::sql_query("SELECT COUNT(*) FROM table")
///             .load(&mut conn)
///     }?;
///
///     Ok(count)
/// }
/// ```
///
/// # Migration depuis rusqlite vers le pool
///
/// Si vous trouvez du code utilisant `Connection::open()` direct:
/// 1. Refactorisez pour recevoir le `DbPool` au lieu du chemin DB
/// 2. Appelez `pool.get()` au lieu de `Connection::open()`
/// 3. Testez avec `cargo check` et `cargo test`
///
/// Fichiers actifs avec `Connection::open()` direct (à migrer progressivement):
/// - `src/commands/metadata/*.rs`
/// - `src/commands/import_clean/*.rs`
/// - `src/commands/pair_data/*.rs`
///
/// Fichiers MIGRÉS vers le pool:
/// - `src/services/database_loader.rs` (stocke pool, prêt pour migration Diesel)
/// - `src-tauri/src/db/mod.rs` (utilise pool correctement)
///
/// # Monitoring du pool en production (Future)
///
/// Pour déboguer les problèmes de pool:
/// ```ignore
/// let state = pool.state();
/// tracing::info!(
///     "Pool state: {} idle, {} used",
///     state.idle_connections,
///     state.connections - state.idle_connections
/// );
/// ```

/// Crée la table calendar_events si elle n'existe pas
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

    Ok(())
}

/// Crée les tables de la DB paires (candle_data, pair_metadata, import_log)
/// Utilisée lors de l'initialisation du pool pairs.db
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

/// Crée la table calendar_imports si elle n'existe pas
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

    // Ajouter colonne calendar_import_id à calendar_events si elle n'existe pas
    // (vérification basique pour éviter les erreurs)
    let _ = diesel::sql_query("ALTER TABLE calendar_events ADD COLUMN calendar_import_id INTEGER")
        .execute(&mut conn);

    Ok(())
}
