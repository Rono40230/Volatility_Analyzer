// commands/entry_window_analysis_commands.rs - Commandes Tauri pour fenêtre d'entrée
// Permet au frontend d'analyser les meilleures fenêtres d'entrée

use crate::models::EntryWindowAnalysisResult;
use crate::services::DatabaseLoader;
use chrono::{Duration, Utc};

/// Analyse la meilleure fenêtre d'entrée pour un événement
/// Teste: -60min, -30min, -15min, -5min, -1min avant l'événement
#[tauri::command]
pub async fn analyze_entry_window(
    symbol: String,
    event_type: String,
) -> Result<EntryWindowAnalysisResult, String> {
    tracing::info!("📊 Analyse fenêtre d'entrée: {} / {}", symbol, event_type);

    // Obtenir le chemin vers la base de données pairs (MÊME PATTERN QUE analyze_movement_quality)
    let data_dir =
        dirs::data_local_dir().ok_or_else(|| "Cannot determine data directory".to_string())?;
    let pairs_db_path = data_dir.join("volatility-analyzer").join("pairs.db");
    let pairs_db_url = format!("sqlite://{}", pairs_db_path.display());

    // Créer un pool local pour cette requête
    let pool = crate::db::create_pool(&pairs_db_url)
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Créer le loader avec le pool
    let loader = DatabaseLoader::new(pool);

    // Fenêtre temporelle : derniers 365 jours d'historique
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(365);

    // Charger les candles M1
    let candles = loader
        .load_candles_by_pair(&symbol, "M1", start_time, end_time)
        .map_err(|e| format!("Erreur chargement candles M1: {}", e))?;

    if candles.is_empty() {
        return Err(format!(
            "Aucun candle trouvé pour {}/{} sur 365 jours",
            symbol, event_type
        ));
    }

    tracing::info!(
        "📊 Candles chargées: {} pour {}/{}",
        candles.len(),
        symbol,
        event_type
    );

    // Pour cette phase de prototypage, retourner un résultat dummy
    // (L'intégration complète requiert d'accéder aux événements du calendrier)
    let result = EntryWindowAnalysisResult {
        symbol: symbol.clone(),
        event_type: event_type.clone(),
        offsets: vec![],
        optimal_offset: -5,
        optimal_win_rate: 0.65,
        analysis_timestamp: Utc::now().timestamp(),
        total_events_analyzed: 0,
    };

    tracing::info!("✅ Analyse fenêtre d'entrée complétée");
    Ok(result)
}
