use super::super::pair_data::PairDataState;
use super::analysis::{analyze_symbol, AnalysisCacheState, CommandError};
use crate::commands::calendar_commands::CalendarState;
use crate::models::HourlyStats;
use tauri::State;
use tracing::info;

#[tauri::command]
pub async fn get_hourly_stats(
    symbol: String,
    hour: u8,
    calendar_id: i32,
    calendar_state: State<'_, CalendarState>,
    pair_state: State<'_, PairDataState>,
    analysis_cache: State<'_, AnalysisCacheState>,
) -> Result<HourlyStats, CommandError> {
    info!(
        "Command: get_hourly_stats({}, hour={}, calendar_id={})",
        symbol, hour, calendar_id
    );

    if hour > 23 {
        return Err(CommandError {
            message: format!("Invalid hour: {}. Must be 0-23", hour),
            error_type: "ValidationError".to_string(),
        });
    }

    let result = analyze_symbol(symbol, calendar_id, None, None, calendar_state, pair_state, analysis_cache).await?;

    let stats = result
        .hourly_stats
        .into_iter()
        .find(|s| s.hour == hour)
        .ok_or_else(|| CommandError {
            message: format!("No stats found for hour {}", hour),
            error_type: "NotFound".to_string(),
        })?;

    Ok(stats)
}

#[tauri::command]
pub async fn get_best_hours(
    symbol: String,
    calendar_id: i32,
    calendar_state: State<'_, CalendarState>,
    pair_state: State<'_, PairDataState>,
    analysis_cache: State<'_, AnalysisCacheState>,
) -> Result<(u8, u8), CommandError> {
    info!(
        "Command: get_best_hours({}, calendar_id={})",
        symbol, calendar_id
    );

    let result = analyze_symbol(symbol, calendar_id, None, None, calendar_state, pair_state, analysis_cache).await?;
    Ok(result.best_quarter)
}
