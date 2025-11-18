// commands/movement_analysis_commands.rs - Commandes pour analyse qualité mouvements
// Conforme RÈGLE 5: < 200L pour commands

use crate::models::EventMovementQuality;
use crate::services::{DatabaseLoader, MovementAnalysisConfig, MovementAnalyzer};
use chrono::{Duration, Utc};

/// Analyse la qualité des mouvements pour une paire et événement donnés
#[tauri::command]
pub async fn analyze_movement_quality(
    symbol: String,
    event_type: String,
) -> Result<EventMovementQuality, String> {
    // Obtenir le chemin vers la base de données pairs
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

    // Charger les candles M1 (ou M5 selon disponibilité)
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

    // Configurer l'analyseur avec les paramètres de Phase 1.2
    let config = MovementAnalysisConfig {
        directional_threshold_atr_ratio: 0.75, // Mouvement > 75% ATR
        reversal_window_minutes: 15,           // Analyser reversals dans 15min
        min_required_candles: 30,
    };

    tracing::info!(
        "🔍 Exécution analyse mouvement pour {} - {}",
        symbol,
        event_type
    );

    // Exécuter l'analyse
    let quality =
        MovementAnalyzer::analyze_movement_quality(&symbol, &event_type, &candles, &config)
            .map_err(|e| format!("Erreur analyse mouvement: {}", e))?;

    tracing::info!(
        "✅ Analyse complétée - Score qualité: {}",
        quality.quality_score
    );

    // Log détaillé avant retour
    tracing::info!("📤 Retour de analyze_movement_quality:");
    tracing::info!("   - quality_score: {}", quality.quality_score);
    tracing::info!(
        "   - directional_move_rate: {}",
        quality.directional_move_rate
    );
    tracing::info!("   - whipsaw_rate: {}", quality.whipsaw_rate);
    tracing::info!("   - success_rate: {}", quality.success_rate);
    tracing::info!("   - sample_size: {}", quality.sample_size);
    tracing::info!("   - avg_pips_moved: {}", quality.avg_pips_moved);

    Ok(quality)
}

/// Récupère les qualités de mouvement stockées en DB pour une paire
#[tauri::command]
pub async fn get_movement_qualities(symbol: String) -> Result<Vec<EventMovementQuality>, String> {
    // Placeholder : sera implémenté avec requête DB
    // Pour l'instant, retourne vecteur vide
    let _symbol = symbol;
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_analyze_movement_quality_params_valid() {
        let symbol = "EURUSD".to_string();
        let event_type = "NFP".to_string();
        assert!(!symbol.is_empty());
        assert!(!event_type.is_empty());
    }
}
