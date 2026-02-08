use super::models::*;
use crate::models::Candle;

/// Construit la série ATR (Average True Range) pour une liste de candles.
pub fn build_atr_series(candles: &[Candle], period: usize) -> Vec<f64> {
    if candles.is_empty() {
        return Vec::new();
    }

    let period = period.max(1);
    let mut trs = Vec::with_capacity(candles.len());
    let mut prev_close: Option<f64> = None;
    for candle in candles {
        trs.push(candle.true_range(prev_close));
        prev_close = Some(candle.close);
    }

    let mut atr = vec![0.0; candles.len()];
    let mut sum = 0.0;
    for (i, tr) in trs.iter().enumerate() {
        sum += tr;
        if i >= period {
            sum -= trs[i - period];
        }
        let denom = if i + 1 < period { i + 1 } else { period } as f64;
        atr[i] = sum / denom;
    }

    atr
}

/// Met à jour les extremes (MFE/MAE) d'une position Long.
pub fn update_long_excursion(pos: &mut Position, candle: &Candle) {
    if candle.high > pos.highest_price {
        pos.highest_price = candle.high;
    }
    if candle.low < pos.lowest_price {
        pos.lowest_price = candle.low;
    }
    pos.mfe = pos.highest_price - pos.entry_price;
    pos.mae = pos.entry_price - pos.lowest_price;
}

/// Met à jour les extremes (MFE/MAE) d'une position Short.
pub fn update_short_excursion(pos: &mut Position, candle: &Candle) {
    if candle.high > pos.highest_price {
        pos.highest_price = candle.high;
    }
    if candle.low < pos.lowest_price {
        pos.lowest_price = candle.low;
    }
    pos.mfe = pos.entry_price - pos.lowest_price;
    pos.mae = pos.highest_price - pos.entry_price;
}

/// Met à jour le trailing stop d'une position Long.
/// Retourne true si le stop a été relevé.
pub fn update_long_trailing(
    pos: &mut Position,
    candle: &Candle,
    trail_dist: f64,
    last_trail_update: &mut Option<chrono::DateTime<chrono::Utc>>,
    refresh_seconds: i64,
) -> bool {
    let should_refresh = match *last_trail_update {
        None => true,
        Some(last) => refresh_seconds == 0
            || (candle.datetime - last).num_seconds() >= refresh_seconds,
    };
    if should_refresh {
        let candidate = candle.high - trail_dist;
        if candidate > pos.stop_loss {
            pos.stop_loss = candidate;
            *last_trail_update = Some(candle.datetime);
            return true;
        }
        *last_trail_update = Some(candle.datetime);
    }
    false
}

/// Met à jour le trailing stop d'une position Short.
/// Retourne true si le stop a été abaissé.
pub fn update_short_trailing(
    pos: &mut Position,
    candle: &Candle,
    trail_dist: f64,
    last_trail_update: &mut Option<chrono::DateTime<chrono::Utc>>,
    refresh_seconds: i64,
) -> bool {
    let should_refresh = match *last_trail_update {
        None => true,
        Some(last) => refresh_seconds == 0
            || (candle.datetime - last).num_seconds() >= refresh_seconds,
    };
    if should_refresh {
        let candidate = candle.low + trail_dist;
        if candidate < pos.stop_loss {
            pos.stop_loss = candidate;
            *last_trail_update = Some(candle.datetime);
            return true;
        }
        *last_trail_update = Some(candle.datetime);
    }
    false
}

/// Paramètres pour construire le résultat final du trade.
pub struct TradeResultParams {
    pub event_time: chrono::DateTime<chrono::Utc>,
    pub entry_time_final: Option<chrono::DateTime<chrono::Utc>>,
    pub exit_time_final: chrono::DateTime<chrono::Utc>,
    pub long_pos: Option<Position>,
    pub short_pos: Option<Position>,
    pub long_pips: f64,
    pub short_pips: f64,
    pub timeout_triggered: bool,
    pub point_value: f64,
    pub logs: Vec<String>,
}

/// Construit le TradeResult final à partir de l'état des positions.
pub fn build_trade_result(params: TradeResultParams) -> TradeResult {
    let total_pips = params.long_pips + params.short_pips;
    let outcome = if params.long_pos.is_none() && params.short_pos.is_none() {
        TradeOutcome::NoEntry
    } else if params.timeout_triggered {
        TradeOutcome::Timeout
    } else if total_pips > 0.0 {
        TradeOutcome::TakeProfit
    } else {
        TradeOutcome::StopLoss
    };

    let mut logs = params.logs;
    if params.long_pos.is_none() && params.short_pos.is_none() {
        logs.push("⚠️ Aucun déclenchement avant timeout".to_string());
    }

    let entry_time_output = params.entry_time_final
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default();
    let exit_time_output = if params.long_pos.is_none() && params.short_pos.is_none() {
        "".to_string()
    } else {
        params.exit_time_final.to_rfc3339()
    };

    let mfe_total = params.long_pos.as_ref().map(|p| p.mfe).unwrap_or(0.0)
        + params.short_pos.as_ref().map(|p| p.mfe).unwrap_or(0.0);
    let mae_total = params.long_pos.as_ref().map(|p| p.mae).unwrap_or(0.0)
        + params.short_pos.as_ref().map(|p| p.mae).unwrap_or(0.0);

    TradeResult {
        event_date: params.event_time.to_rfc3339(),
        entry_time: entry_time_output,
        exit_time: exit_time_output,
        duration_minutes: if params.entry_time_final.is_some() {
            (params.exit_time_final - params.event_time).num_minutes() as i32
        } else {
            0
        },
        pips_net: total_pips,
        outcome,
        max_favorable_excursion: mfe_total / params.point_value,
        max_adverse_excursion: mae_total / params.point_value,
        logs,
    }
}
