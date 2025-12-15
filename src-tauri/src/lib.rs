// lib.rs - Point d'entrée principal Tauri
// Conforme .clinerules : < 120 lignes pour main/lib

mod commands;
mod db;
mod models;
mod schema;
mod services;

use commands::retrospective_analysis::analyze_volatility_profile;
use commands::*;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialise le logger pour tracing avec niveau DEBUG
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("🚀 Démarrage de l'application Tauri...");
    tracing::info!("Initialisation du pool DB calendrier");

    // Initialise le pool DB pour le calendrier économique
    // Base de données dans ~/.local/share pour éviter le hot-reload
    let data_dir = match dirs::data_local_dir() {
        Some(dir) => dir,
        None => {
            tracing::error!("❌ ERREUR: Impossible de déterminer le répertoire de données local");
            tracing::error!(
                "   Votre système ne semble pas avoir de répertoire de données standard."
            );
            std::process::exit(1);
        }
    };

    let db_path = data_dir.join("volatility-analyzer").join("volatility.db");

    // Créer le dossier si nécessaire
    if let Some(parent) = db_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::error!(
                "❌ ERREUR: Impossible de créer le répertoire de données: {}",
                e
            );
            tracing::error!("   Chemin: {:?}", parent);
            std::process::exit(1);
        }
    }

    let db_url = format!("sqlite://{}", db_path.display());
    let calendar_pool = match db::create_pool(&db_url) {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!(
                "❌ ERREUR: Impossible de créer le pool de base de données calendrier: {}",
                e
            );
            tracing::error!("   URL: {}", db_url);
            std::process::exit(1);
        }
    };

    tracing::info!("✅ Pool de base de données calendrier initialisé");

    // Crée la table calendar_events si elle n'existe pas
    if let Err(e) = db::ensure_calendar_table(&calendar_pool) {
        tracing::error!(
            "❌ ERREUR: Impossible de créer la table calendar_events: {}",
            e
        );
        tracing::error!("   La base de données pourrait être corrompue.");
        std::process::exit(1);
    }

    tracing::info!("✅ Table calendar_events vérifiée/créée");

    // Crée la table calendar_imports si elle n'existe pas
    if let Err(e) = db::ensure_calendar_imports_table(&calendar_pool) {
        tracing::error!(
            "❌ ERREUR: Impossible de créer la table calendar_imports: {}",
            e
        );
        std::process::exit(1);
    }

    tracing::info!("✅ Table calendar_imports vérifiée/créée");

    let calendar_state = calendar_commands::CalendarState {
        pool: Mutex::new(Some(calendar_pool.clone())),
    };

    tracing::info!("✅ CalendarState créé avec pool actif");

    // Initialise le pool DB pour les paires (données de trading)
    let pairs_db_path = data_dir.join("volatility-analyzer").join("pairs.db");

    let pairs_db_url = format!("sqlite://{}", pairs_db_path.display());
    let pairs_pool = match db::create_pool(&pairs_db_url) {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!(
                "❌ ERREUR: Impossible de créer le pool de base de données paires: {}",
                e
            );
            std::process::exit(1);
        }
    };

    tracing::info!("✅ Pool de base de données paires initialisé");

    if let Err(e) = db::ensure_pair_tables(&pairs_pool) {
        tracing::error!("❌ ERREUR: Impossible de créer les tables paires: {}", e);
        std::process::exit(1);
    }

    tracing::info!("✅ Tables paires vérifiées/créées");

    let pair_state = pair_data::PairDataState {
        pool: Mutex::new(Some(pairs_pool)),
    };

    tracing::info!("✅ PairDataState créé avec pool actif");

    // Initialise le state pour les métriques d'événements
    let candles_state = commands::event_metrics::CandlesState::default();

    tracing::info!("✅ CandlesState créé pour event metrics");

    // Initialise l'index des candles (vide au démarrage, rempli par init_candle_index)
    let candle_index_state = candle_index_commands::CandleIndexState {
        index: Mutex::new(None),
    };

    tracing::info!("✅ CandleIndexState créé (vide, en attente d'initialisation)");

    // Initialise le service d'archivage (utilise le pool calendrier)
    let archive_service = services::ArchiveService::new(calendar_pool.clone());
    tracing::info!("✅ ArchiveService créé");

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(calendar_state)
        .manage(pair_state)
        .manage(candles_state)
        .manage(candle_index_state)
        .manage(archive_service)
        .invoke_handler(tauri::generate_handler![
            // Volatility commands (Phase 1)
            ping,
            load_symbols,
            analyze_symbol,
            get_hourly_stats,
            get_best_hours,
            calculer_offset_optimal,
            calculer_taux_reussite,
            calculer_frequence_whipsaw,
            analyze_slice_metrics, // NEW: Analyse métriques d'un créneau 15min
            analyze_straddle_metrics, // NEW: Analyse complète avec VRAIES données
            analyze_volatility_duration_for_slice, // NEW: Durée de volatilité réelle
            analyze_quarter_entry_timing, // NEW: Meilleur moment d'entrée par minute
            load_candles_for_hour, // NEW: Charger 60 candles pour une heure
            get_cached_candles_for_hour, // NEW: Récupérer candles en cache
            // Calendar commands (Phase 2 MVP)
            get_upcoming_events,
            load_economic_events_from_csv,
            import_and_convert_calendar, // Nouveau: import automatisé
            get_calendar_import_info,    // Info sur le dernier import
            // Event correlation commands (Phase 3 ML)
            get_events_for_period,
            analyze_event_correlation,
            get_correlation_heatmap, // Heatmap événements × paires
            get_past_events,         // Liste des événements passés pour dropdown
            get_pair_event_history,  // Historique des événements pour une paire
            // Pair data import commands (Phase 4)
            import_pair_data,
            clean_csv_files,        // Nouveau: nettoyage CSV européens
            import_and_clean_files, // Nouveau: import unifié (clean + import)
            // Session analysis commands (Phase 5)
            analyze_sessions,
            // File management commands (Phase 6)
            list_calendar_files,
            list_pair_csv_files,
            get_pair_data_summary,
            get_pair_metadata_from_db,   // NEW: métadonnées depuis BD
            get_calendars_metadata,      // NEW: métadonnées calendriers (format UI)
            get_calendar_id_by_filename, // NEW: récupérer ID calendrier depuis nom fichier
            get_calendar_period_by_id,   // NEW: récupérer dates période du calendrier par ID
            get_pairs_metadata,          // NEW: métadonnées paires (format UI)
            import_calendar_files,       // NEW: importer calendriers
            delete_pair_from_db,         // NEW: supprimer paire de la BD
            delete_calendar_from_db,     // NEW: supprimer calendrier de la BD
            delete_calendar_file,
            delete_pair_files,
            // Config commands (Phase 7)
            get_selected_calendar_file,
            set_selected_calendar_file,
            // Event metrics commands (Phase 1 Roadmap)
            calculer_metriques_evenement,
            load_candles_for_metrics,
            get_available_symbols,
            clear_candles,
            // Movement analysis commands (Phase 1.2)
            analyze_movement_quality,
            get_movement_qualities,
            // Entry window analysis commands (Phase 1.3)
            analyze_entry_window,
            // Volatility duration commands (Phase 1.1)
            analyze_volatility_duration,
            // Candle index commands (Performance optimization)
            init_candle_index,
            load_pair_candles,
            get_pair_candles,
            get_candle_index_stats,
            get_candles_for_hour,
            get_candles_for_quarter, // NEW: Charger candles filtrées par quarter (TÂCHE 5)
            // Archive commands
            save_archive,
            list_archives,
            list_all_archives,
            get_archive,
            delete_archive,
            // Global Analysis (Phase IA)
            analyze_all_archives,
            get_available_pairs,
            // Retrospective analysis commands (Phase 7)
            analyze_peak_delay,
            analyze_decay_profile,
            analyze_volatility_profile,
            get_event_types,
            // PDF export commands
            exporter_formules_pdf,
        ]);

    tracing::info!("✅ Tauri Builder configuré");
    tracing::info!("📋 Commandes enregistrées: analyze_symbol, import_pair_data, import_and_clean_files, delete_pair_from_db, delete_calendar_from_db, et autres");
    tracing::info!("🔧 Lancement de l'application...");

    if let Err(e) = builder.run(tauri::generate_context!()) {
        tracing::error!(
            "❌ ERREUR FATALE lors du lancement de l'application Tauri: {}",
            e
        );
        std::process::exit(1);
    }
}
