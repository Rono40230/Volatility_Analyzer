// commands/volatility/analyze_volatility_duration_command.rs
// Commande pour analyser la durée de volatilité d'un créneau

use crate::commands::pair_data::PairDataState;
use serde::{Deserialize, Serialize};
use tauri::{command, State};

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
    pair_state: State<'_, PairDataState>,
    volatility_profile_service: State<'_, crate::services::VolatilityProfileService>,
) -> Result<VolatilityDurationResponse, String> {
    use crate::services::candle_index::CandleIndex;
    use crate::services::database_loader::DatabaseLoader;
    use crate::services::slice_metrics_analyzer;
    use crate::services::volatility_duration_calculator::calculer_duree_volatilite;
    use crate::models::asset_class::AssetProperties;

    let pool = pair_state
        .pool
        .lock()
        .map_err(|e| format!("Lock pair pool failed: {e}"))?
        .clone()
        .ok_or("Pair database pool not initialized")?;

    let db_loader = DatabaseLoader::new(pool);
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

    // FIX 2.3: Get asset type and retrieve profile from DB
    let asset_props = AssetProperties::from_symbol(&symbol);
    let profile = volatility_profile_service
        .get_profile_by_type(asset_props.asset_type)
        .map_err(|e| format!("Failed to get volatility profile: {}", e))?;

    let default_peak = Some(5); // fallback
    let default_half_life = Some(profile.half_life_minutes.ceil() as i64);

    // Calculer la durée de volatilité avec profil d'asset_class
    let duration = calculer_duree_volatilite(&candles, &symbol, default_peak, default_half_life);

    Ok(VolatilityDurationResponse {
        peak_duration_minutes: duration.peak_duration_minutes,
        volatility_half_life_minutes: duration.volatility_half_life_minutes,
        recommended_trade_expiration_minutes: duration.recommended_trade_expiration_minutes,
        confidence_score: duration.confidence_score,
        sample_size: duration.sample_size,
    })
}
