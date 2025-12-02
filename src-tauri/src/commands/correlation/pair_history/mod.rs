mod types;
pub use types::{PairEventHistory, PairEventHistoryItem, TopEvent};

use rusqlite::{Connection, Result as SqliteResult};
use tauri::State;

use super::volatility_helpers::{calculate_volatilities_optimized, parse_sqlite_datetime};
use crate::commands::candle_index_commands::CandleIndexState;

#[tauri::command]
pub async fn get_pair_event_history(
    pair_symbol: String,
    months_back: Option<i32>,
    state: State<'_, CandleIndexState>,
) -> Result<PairEventHistory, String> {
    let months = months_back.unwrap_or(6);

    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");

    let db_path = data_dir.join("volatility.db");

    if !db_path.exists() {
        return Err("Database not found".to_string());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let cutoff_date = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(months as i64 * 30))
        .ok_or("Date calculation error")?
        .format("%Y-%m-%d")
        .to_string();

    // Récupérer tous les événements HIGH/MEDIUM dans la période
    let mut event_stmt = conn
        .prepare(
            "SELECT id, description, datetime(event_time), impact
             FROM calendar_events
             WHERE date(event_time) >= ?1 AND impact IN ('H', 'M')
             ORDER BY event_time DESC",
        )
        .map_err(|e| format!("Failed to prepare events: {}", e))?;

    let events: Vec<(i32, String, String, String)> = event_stmt
        .query_map([&cutoff_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| format!("Failed to query events: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect events: {}", e))?;

    let mut index_state = state
        .index
        .lock()
        .map_err(|e| format!("Failed to lock candle index state: {}", e))?;

    let candle_index = index_state
        .as_mut()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;

    candle_index.load_pair_candles(&pair_symbol)?;

    let mut event_history = Vec::new();
    let mut total_volatility = 0.0;
    let mut total_multiplier = 0.0;
    let mut max_vol: f64 = 0.0;
    let mut valid_count = 0;

    for (event_id, event_name, datetime_str, impact) in &events {
        let event_datetime = match parse_sqlite_datetime(datetime_str) {
            Ok(dt) => dt,
            Err(e) => {
                eprintln!("⚠️ Skipping event {}: {}", event_id, e);
                continue;
            }
        };

        let metrics = calculate_volatilities_optimized(
            candle_index,
            &pair_symbol,
            event_datetime,
            30,
            7,
            super::volatility_helpers::get_pip_value(&pair_symbol),
        )
        .unwrap_or(super::volatility_helpers::VolatilityMetrics {
            event_volatility: 0.0,
            baseline_volatility: 0.0,
        });

        let event_volatility = metrics.event_volatility;
        let baseline_volatility = metrics.baseline_volatility;

        if event_volatility > 0.0 && baseline_volatility > 0.0 {
            let multiplier = event_volatility / baseline_volatility;
            let change_percent =
                ((event_volatility - baseline_volatility) / baseline_volatility) * 100.0;

            let direction = if change_percent > 100.0 {
                "HAUSSIER"
            } else if change_percent > 50.0 {
                "BAISSIER"
            } else {
                "NEUTRE"
            }
            .to_string();

            event_history.push(PairEventHistoryItem {
                event_id: *event_id,
                datetime: datetime_str.clone(),
                event_name: event_name.clone(),
                impact: impact.clone(),
                volatility: event_volatility,
                volatility_formatted: Some(format!("{:.1}", event_volatility)),
                change_percent,
                direction,
            });

            total_volatility += event_volatility;
            total_multiplier += multiplier;
            max_vol = max_vol.max(event_volatility);
            valid_count += 1;
        }
    }

    let mut sorted_events = event_history.clone();
    sorted_events.sort_by(|a, b| {
        b.volatility
            .partial_cmp(&a.volatility)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_events: Vec<TopEvent> = sorted_events
        .iter()
        .take(5)
        .map(|e| TopEvent {
            name: e.event_name.clone(),
            datetime: e.datetime.clone(),
            volatility: e.volatility,
        })
        .collect();

    let avg_volatility = if valid_count > 0 {
        total_volatility / valid_count as f64
    } else {
        0.0
    };
    let avg_multiplier = if valid_count > 0 {
        total_multiplier / valid_count as f64
    } else {
        0.0
    };

    Ok(PairEventHistory {
        symbol: pair_symbol,
        period: format!("{} derniers mois", months),
        total_events: valid_count,
        avg_volatility,
        max_volatility: max_vol,
        avg_multiplier,
        events: event_history,
        top_events,
    })
}
