// commands/volatility/analyze_slice_metrics_command.rs
// Commande pour analyser les métriques d'un créneau de 15 minutes

use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct SliceMetricsResponse {
    pub symbol: String,
    pub hour: u8,
    pub quarter: u8,
    pub candle_count: usize,
    pub atr_mean: f64,
    pub atr_max: f64,
    pub volatility_mean: f64,
    pub range_mean: f64,
    pub body_range_mean: f64,
    pub noise_ratio_mean: f64,
    pub breakout_percentage: f64,
    pub volume_imbalance_mean: f64,
    pub shadow_ratio_mean: f64,
    pub optimal_entry_offset: i64,
    pub optimal_entry_win_rate: f64,
    pub movement_quality_score: f64,
    pub movement_quality_label: String,
}

#[command]
pub async fn analyze_slice_metrics(
    symbol: String,
    hour: u8,
    quarter: u8,
) -> Result<SliceMetricsResponse, String> {
    use crate::db;
    use crate::services::candle_index::CandleIndex;
    use crate::services::database_loader::DatabaseLoader;
    use crate::services::slice_metrics_analyzer;

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

    // Analyser les métriques du créneau
    let (metrics, candles) = slice_metrics_analyzer::analyze_slice_metrics(
        &candle_index,
        &symbol,
        hour as u32,
        quarter as u32,
    )?;

    // Optimiser la fenêtre d'entrée
    use crate::services::entry_window_optimizer::optimize_entry_window;
    let entry_window = optimize_entry_window(&candles, &symbol, quarter as u32);

    // Analyser la qualité du mouvement
    use crate::services::movement_quality_analyzer::analyze_movement_quality;
    let movement_quality = analyze_movement_quality(&candles);

    Ok(SliceMetricsResponse {
        symbol,
        hour,
        quarter,
        candle_count: metrics.candle_count,
        atr_mean: metrics.atr_mean,
        atr_max: metrics.atr_max,
        volatility_mean: metrics.volatility_mean,
        range_mean: metrics.range_mean,
        body_range_mean: metrics.body_range_mean,
        noise_ratio_mean: metrics.noise_ratio_mean,
        breakout_percentage: metrics.breakout_percentage,
        volume_imbalance_mean: metrics.volume_imbalance_mean,
        shadow_ratio_mean: metrics.shadow_ratio_mean,
        // New fields
        optimal_entry_offset: entry_window.optimal_offset_minutes,
        optimal_entry_win_rate: entry_window.optimal_win_rate,
        movement_quality_score: movement_quality.overall_quality,
        movement_quality_label: movement_quality.quality_label,
    })
}
