use super::models::*;
use super::simulator::EventSimulator;
use crate::models::{CalendarEvent, AssetProperties};
use crate::services::database_loader::DatabaseLoader;
use chrono::Duration;

pub struct BacktestEngine;

impl BacktestEngine {
    pub fn run(
        pair: &str,
        events: &[CalendarEvent],
        config: BacktestConfig,
        mode: StrategyMode,
        loader: &DatabaseLoader,
    ) -> Result<BacktestResult, String> {
        let mut trades = Vec::new();

        for event in events {
            // 1. Charger les données (T-5 à T+Timeout+10)
            let start_time = event.event_time.and_utc() - Duration::minutes(5);
            let end_time = event.event_time.and_utc()
                + Duration::minutes(config.timeout_minutes as i64 + 10);

            let candles = loader
                .load_candles_by_pair(pair, "M1", start_time, end_time)
                .unwrap_or_default();

            if candles.is_empty() {
                continue;
            }

            // 2. Simuler l'événement
            let trade = EventSimulator::simulate(event, &candles, &config, mode);
            trades.push(trade);
        }

        // 3. Calculer les métriques globales
        // On récupère le nom de l'événement depuis le premier événement ou on utilise une valeur par défaut
        // Note: Le champ description contient le nom de l'événement (ex: "Non-Farm Employment Change")
        let event_name = events.first().map(|e| e.description.clone()).unwrap_or_else(|| "Unknown".to_string());
        Ok(Self::calculer_synthese(pair, &event_name, trades, mode))
    }

    fn calculer_synthese(pair: &str, event_name: &str, trades: Vec<TradeResult>, mode: StrategyMode) -> BacktestResult {
        let asset_props = AssetProperties::from_symbol(pair);
        let total_trades = trades.len();
        let mut winning = 0;
        let mut losing = 0;
        let mut no_entry = 0;
        let mut total_pips = 0.0;
        let mut current_dd = 0.0;
        let mut gross_profit = 0.0;
        let mut gross_loss = 0.0;

        for t in &trades {
            if t.outcome == TradeOutcome::NoEntry {
                no_entry += 1;
                continue;
            }

            total_pips += t.pips_net;

            if t.pips_net > 0.0 {
                winning += 1;
                gross_profit += t.pips_net;
            } else {
                losing += 1;
                gross_loss += t.pips_net.abs();
            }

            // Drawdown calculation (simplified)
            if total_pips < current_dd {
                current_dd = total_pips; // Track lowest point relative to start
            }
            // Note: Real DD calculation tracks peak-to-valley. 
            // This is just net PnL dip. Let's do peak-to-valley.
        }
        
        // Re-calc DD properly
        let mut peak = 0.0;
        let mut running_pnl = 0.0;
        let mut max_drawdown = 0.0;
        
        for t in &trades {
            if t.outcome == TradeOutcome::NoEntry { continue; }
            running_pnl += t.pips_net;
            if running_pnl > peak { peak = running_pnl; }
            let dd = peak - running_pnl;
            if dd > max_drawdown { max_drawdown = dd; }
        }

        let win_rate = if (winning + losing) > 0 {
            (winning as f64 / (winning + losing) as f64) * 100.0
        } else {
            0.0
        };

        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            if gross_profit > 0.0 { 999.0 } else { 0.0 }
        };

        BacktestResult {
            symbol: pair.to_string(),
            event_name: event_name.to_string(),
            unit: asset_props.unit,
            total_trades,
            winning_trades: winning,
            losing_trades: losing,
            no_entries: no_entry,
            win_rate_percent: win_rate,
            total_pips,
            average_pips_per_trade: if (winning + losing) > 0 { total_pips / (winning + losing) as f64 } else { 0.0 },
            max_drawdown_pips: max_drawdown,
            profit_factor,
            trades,
            strategy_mode: mode,
        }
    }
}
