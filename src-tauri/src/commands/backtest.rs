use crate::services::backtest::{BacktestConfig, BacktestEngine, BacktestResult};
use crate::commands::retrospective_analysis::helpers::{setup_databases, load_events_by_type};
use crate::models::trading_costs::TradingCostProfile;
use crate::models::AssetProperties;
use chrono::{NaiveDate, NaiveTime, Duration, Datelike, Utc};
use crate::models::calendar_event::CalendarEvent;

#[tauri::command]
pub async fn run_backtest(
    pair: String,
    event_type: String,
    config: BacktestConfig,
    state: tauri::State<'_, crate::commands::calendar_commands::CalendarState>,
) -> Result<BacktestResult, String> {
    let (conn, loader) = setup_databases(&state).await?;
    let events = load_events_by_type(conn, &event_type).await?;
    
    if events.is_empty() {
        return Err(format!("No events found for type: {}", event_type));
    }

    // BacktestEngine::run est CPU-intensif → spawn_blocking
    tokio::task::spawn_blocking(move || {
        BacktestEngine::run(&pair, &events, config, &loader)
    })
    .await
    .map_err(|e| format!("Backtest task failed: {}", e))?
}

#[tauri::command]
pub async fn run_backtest_time(
    pair: String,
    time: String,
    start_date: String,
    end_date: String,
    config: BacktestConfig,
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

    // BacktestEngine::run est CPU-intensif → spawn_blocking
    tokio::task::spawn_blocking(move || {
        BacktestEngine::run(&pair, &events, config, &loader)
    })
    .await
    .map_err(|e| format!("Backtest time task failed: {}", e))?
}

/// Retourne une configuration de backtest avec les paramètres recommandés
/// basés sur le profil de coûts du symbole et les ratios standards du Straddle.
#[tauri::command]
pub async fn get_recommended_backtest_config(symbol: String) -> Result<BacktestConfig, String> {
    if symbol.is_empty() {
        return Err("Symbole requis".to_string());
    }

    let costs = TradingCostProfile::get_profile(&symbol);
    let asset_props = AssetProperties::from_symbol(&symbol);

    // SL basé sur le profil de coûts : 5× le spread moyen (conservateur)
    // Cohérent avec StraddleParameterService (2.0-6.0× ATR → ~5× spread)
    let stop_loss_pips = (costs.spread_avg * 5.0).ceil();

    Ok(BacktestConfig {
        stop_loss_pips,
        tp_rr: 2.0,                    // R/R standard (cohérent avec hard_tp = 2× SL)
        trailing_atr_coef: 1.5,        // Trailing = 1.5× ATR (standard)
        atr_period: 14,                // Période ATR classique
        trailing_refresh_seconds: 60,  // Rafraîchissement standard
        timeout_minutes: 20,           // Fenêtre événementielle standard [10-30]
        sl_recovery_pips: Some((stop_loss_pips * 1.2).ceil()), // 1.2× SL (cohérent)
        spread_pips: costs.spread_avg,
        slippage_pips: costs.slippage,
        point_value: asset_props.pip_value,
    })
}
