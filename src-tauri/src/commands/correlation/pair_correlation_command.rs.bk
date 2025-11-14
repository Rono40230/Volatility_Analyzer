// pair_correlation_command.rs - Commande Tauri pour corrélation par paire
// Conforme .clinerules: <200L

use super::pair_correlation::{calculate_pair_event_correlation, PairCorrelationResult};
use crate::commands::candle_index_commands::CandleIndexState;
use tauri::State;
use tracing::info;

#[tauri::command]
pub async fn get_pair_event_correlation(
    symbol: String,
    calendar_id: Option<i32>,
    state: State<'_, CandleIndexState>,
) -> Result<PairCorrelationResult, String> {
    info!(
        "📊 get_pair_event_correlation: symbol={}, calendar_id={:?}",
        symbol, calendar_id
    );

    // Charger les candles pour cette paire
    let mut index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock candle index: {}", e))?;

    let candle_index = index_state
        .as_mut()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;

    candle_index.load_pair_candles(&symbol)?;

    let result = calculate_pair_event_correlation(&symbol, calendar_id, candle_index);

    drop(index_state); // Libérer le lock

    result
}
