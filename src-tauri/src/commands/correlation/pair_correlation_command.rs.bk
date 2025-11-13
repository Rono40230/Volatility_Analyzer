// pair_correlation_command.rs - Commande Tauri pour corrélation par paire
// Conforme .clinerules: <200L

use super::pair_correlation::{calculate_pair_event_correlation, PairCorrelationResult};
use tracing::info;

#[tauri::command]
pub async fn get_pair_event_correlation(
    symbol: String,
    calendar_id: Option<i32>,
) -> Result<PairCorrelationResult, String> {
    info!(
        "📊 get_pair_event_correlation: symbol={}, calendar_id={:?}",
        symbol, calendar_id
    );

    calculate_pair_event_correlation(&symbol, calendar_id)
}
