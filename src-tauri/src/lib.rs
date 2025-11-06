// lib.rs - Point d'entrée principal Tauri
// Conforme .clinerules : < 120 lignes pour main/lib

mod commands;
mod models;
mod services;
mod db;

use commands::*;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialise le logger pour tracing avec niveau DEBUG
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("🚀 Démarrage de l'application Tauri...");
    tracing::info!("Initialisation du pool DB calendrier");

    // Initialise le pool DB pour le calendrier économique
    // Base de données dans ~/.local/share pour éviter le hot-reload
    let data_dir = match dirs::data_local_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("❌ ERREUR: Impossible de déterminer le répertoire de données local");
            eprintln!("   Votre système ne semble pas avoir de répertoire de données standard.");
            std::process::exit(1);
        }
    };
    
    let db_path = data_dir
        .join("volatility-analyzer")
        .join("volatility.db");
    
    // Créer le dossier si nécessaire
    if let Some(parent) = db_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("❌ ERREUR: Impossible de créer le répertoire de données: {}", e);
            eprintln!("   Chemin: {:?}", parent);
            std::process::exit(1);
        }
    }
    
    let db_url = format!("sqlite://{}", db_path.display());
    let calendar_pool = match db::create_pool(&db_url) {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("❌ ERREUR: Impossible de créer le pool de base de données calendrier: {}", e);
            eprintln!("   URL: {}", db_url);
            std::process::exit(1);
        }
    };
    
    println!("✅ Pool de base de données calendrier initialisé");
    
    // Crée la table calendar_events si elle n'existe pas
    if let Err(e) = db::ensure_calendar_table(&calendar_pool) {
        eprintln!("❌ ERREUR: Impossible de créer la table calendar_events: {}", e);
        eprintln!("   La base de données pourrait être corrompue.");
        std::process::exit(1);
    }
    
    println!("✅ Table calendar_events vérifiée/créée");

    let calendar_state = calendar_commands::CalendarState {
        pool: Mutex::new(Some(calendar_pool)),
    };

    println!("✅ CalendarState créé avec pool actif");

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(calendar_state)
        .invoke_handler(tauri::generate_handler![
            // Volatility commands (Phase 1)
            ping,
            load_symbols,
            analyze_symbol,
            get_hourly_stats,
            get_best_hours,
            // Calendar commands (Phase 2 MVP)
            get_upcoming_events,
            load_economic_events_from_csv,
            import_and_convert_calendar, // Nouveau: import automatisé
            get_calendar_import_info, // Info sur le dernier import
            // Event correlation commands (Phase 3 ML)
            get_events_for_period,
            analyze_event_correlation,
            get_correlation_heatmap, // Heatmap événements × paires
            get_event_impact_by_pair, // Impact d'un événement sur toutes les paires
            get_past_events, // Liste des événements passés pour dropdown
            get_pair_event_history, // Historique des événements pour une paire
            // Pair data import commands (Phase 4)
            import_pair_data,
            clean_csv_files, // Nouveau: nettoyage CSV européens
            import_and_clean_files, // Nouveau: import unifié (clean + import)
            // Session analysis commands (Phase 5)
            analyze_sessions,
            // File management commands (Phase 6)
            list_calendar_files,
            list_pair_csv_files,
            delete_calendar_file,
            delete_pair_files,
            // Config commands (Phase 7)
            get_selected_calendar_file,
            set_selected_calendar_file,
        ]);

    println!("✅ Tauri Builder configuré");
    println!("📋 Commandes enregistrées: ping, load_symbols, analyze_symbol, get_hourly_stats, get_best_hours, get_upcoming_events, load_economic_events_from_csv, import_pair_data, analyze_sessions");
    println!("🔧 Lancement de l'application...");

    if let Err(e) = builder.run(tauri::generate_context!()) {
        eprintln!("❌ ERREUR FATALE lors du lancement de l'application Tauri: {}", e);
        std::process::exit(1);
    }
}
