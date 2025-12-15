// commands/volatility/analyze_volatility_duration_command.rs
// Commande pour analyser la durée de volatilité d'un créneau

use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct VolatilityDurationResponse {
    pub peak_duration_minutes: i64,
    pub volatility_half_life_minutes: i64,
    pub recommended_trade_expiration_minutes: i64,
    pub confidence_score: f64,
    pub sample_size: usize,
}

#[command]
pub async fn analyze_volatility_duration_for_slice(
    symbol: String,
    hour: u8,
    quarter: u8,
) -> Result<VolatilityDurationResponse, String> {
    use crate::db;
    use crate::services::candle_index::CandleIndex;
    use crate::services::database_loader::DatabaseLoader;
    use crate::services::slice_metrics_analyzer;
    use crate::services::volatility_duration_calculator::calculer_duree_volatilite;

    // Créer le pool de connexions pour la BD paires
    let data_dir =
        dirs::data_local_dir().ok_or_else(|| "Failed to get data directory".to_string())?;
    let pairs_db_path = data_dir.join("volatility-analyzer").join("pairs.db");
    let pairs_db_url = format!("sqlite://{}", pairs_db_path.display());

    let pairs_pool = db::create_pool(&pairs_db_url)
        .map_err(|e| format!("Failed to create pairs DB pool: {}", e))?;

    // Créer un CandleIndex avec DatabaseLoader
    let db_loader = DatabaseLoader::new(pairs_pool);
    let mut candle_index = CandleIndex::with_db_loader(db_loader);

    // Charger les bougies pour ce symbole
    candle_index
        .load_pair_candles(&symbol)
        .map_err(|e| format!("Failed to load candles for {}: {}", symbol, e))?;

    // Analyser les métriques du créneau et récupérer les bougies
    let (_metrics, candles) = slice_metrics_analyzer::analyze_slice_metrics(
        &candle_index,
        &symbol,
        hour as u32,
        quarter as u32,
    )?;

    // Calculer la durée de volatilité
    let duration = calculer_duree_volatilite(&candles, &symbol);

    Ok(VolatilityDurationResponse {
        peak_duration_minutes: duration.peak_duration_minutes,
        volatility_half_life_minutes: duration.volatility_half_life_minutes,
        recommended_trade_expiration_minutes: duration.recommended_trade_expiration_minutes,
        confidence_score: duration.confidence_score,
        sample_size: duration.sample_size,
    })
}
