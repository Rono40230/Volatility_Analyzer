use crate::services::backtest::{BacktestConfig, BacktestEngine, BacktestResult, StrategyMode};
use crate::commands::retrospective_analysis::helpers::{setup_databases, load_events_by_type};
use chrono::{NaiveDate, NaiveTime, Duration, Datelike, Utc};
use crate::models::calendar_event::CalendarEvent;

#[tauri::command]
pub async fn run_backtest(
    pair: String,
    event_type: String,
    config: BacktestConfig,
    mode: StrategyMode,
    state: tauri::State<'_, crate::commands::calendar_commands::CalendarState>,
) -> Result<BacktestResult, String> {
    let (conn, loader) = setup_databases(&state).await?;
    let events = load_events_by_type(conn, &event_type).await?;
    
    if events.is_empty() {
        return Err(format!("No events found for type: {}", event_type));
    }

    BacktestEngine::run(&pair, &events, config, mode, &loader)
}

#[tauri::command]
pub async fn run_backtest_time(
    pair: String,
    time: String,
    start_date: String,
    end_date: String,
    config: BacktestConfig,
    mode: StrategyMode,
    state: tauri::State<'_, crate::commands::calendar_commands::CalendarState>,
) -> Result<BacktestResult, String> {
    let (_, loader) = setup_databases(&state).await?;

    let time = NaiveTime::parse_from_str(&time, "%H:%M")
        .map_err(|e| format!("Invalid time format: {}", e))?;
    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date: {}", e))?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date: {}", e))?;

    let mut events = Vec::new();
    let mut current_date = start;

    while current_date <= end {
        // Skip weekends (Saturday=6, Sunday=7)
        let weekday = current_date.weekday().number_from_monday();
        if weekday <= 5 {
            let event_time = current_date.and_time(time);
            
            events.push(CalendarEvent {
                id: 0, // Dummy ID
                symbol: "USD".to_string(), // Dummy currency
                event_time,
                description: format!("Time Backtest {}", time),
                impact: "Medium".to_string(),
                actual: None,
                forecast: None,
                previous: None,
                created_at: Utc::now().naive_utc(),
                calendar_import_id: 0, // Dummy import ID
            });
        }

        current_date += Duration::days(1);
    }

    if events.is_empty() {
        return Err("No valid weekdays found in range".to_string());
    }

    BacktestEngine::run(&pair, &events, config, mode, &loader)
}
