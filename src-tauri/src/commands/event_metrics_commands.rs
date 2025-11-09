// commands/event_metrics_commands.rs - Commandes Tauri pour calcul de métriques d'événements
// Conforme .clinerules : < 200L, pas d'unwrap()

use crate::models::{Candle, EventMetrics};
use crate::services::event_metrics_aggregator::{EventMetricsAggregator, MetricsConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

/// État partagé pour les données de candles
pub struct CandlesState {
    pub candles: std::sync::Mutex<Vec<Candle>>,
}

/// Paramètres pour le calcul des métriques
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalculateMetricsParams {
    pub event_name: String,
    pub event_time: String, // ISO 8601
    pub symbol: String,
    pub atr_period: Option<usize>,
    pub atr_multiplier_sl: Option<f64>,
    pub atr_multiplier_tp: Option<f64>,
    pub max_trade_duration_minutes: Option<usize>,
}

/// Réponse de la commande
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsCommandResult {
    pub success: bool,
    pub metrics: Option<EventMetrics>,
    pub error: Option<String>,
}

/// Commande Tauri : Calculer les métriques d'un événement
#[tauri::command]
pub async fn calculate_event_metrics(
    params: CalculateMetricsParams,
    candles_state: State<'_, CandlesState>,
) -> Result<MetricsCommandResult, String> {
    info!("Calculating metrics for event: {}", params.event_name);

    // Parser la date de l'événement
    let event_time = DateTime::parse_from_rfc3339(&params.event_time)
        .map_err(|e| format!("Invalid event time format: {}", e))?
        .with_timezone(&Utc);

    // Récupérer les candles
    let candles_guard = candles_state
        .candles
        .lock()
        .map_err(|e| format!("Failed to lock candles: {}", e))?;

    if candles_guard.is_empty() {
        return Ok(MetricsCommandResult {
            success: false,
            metrics: None,
            error: Some("No candles loaded. Please import data first.".to_string()),
        });
    }

    // Filtrer les candles pour le symbole demandé
    let symbol_candles: Vec<Candle> = candles_guard
        .iter()
        .filter(|c| c.symbol == params.symbol)
        .cloned()
        .collect();

    if symbol_candles.is_empty() {
        return Ok(MetricsCommandResult {
            success: false,
            metrics: None,
            error: Some(format!(
                "No candles found for symbol {}. Available symbols: {:?}",
                params.symbol,
                get_unique_symbols(&candles_guard)
            )),
        });
    }

    // Configuration
    let config = MetricsConfig {
        atr_period: params.atr_period.unwrap_or(14),
        atr_multiplier_sl: params.atr_multiplier_sl.unwrap_or(2.0),
        atr_multiplier_tp: params.atr_multiplier_tp.unwrap_or(3.0),
        max_trade_duration_minutes: params.max_trade_duration_minutes.unwrap_or(120),
    };

    // Créer l'agrégateur et calculer
    let aggregator =
        EventMetricsAggregator::new(&symbol_candles, event_time, params.event_name.clone());

    match aggregator.calculate_all_metrics(config) {
        Ok(metrics) => {
            info!("Metrics calculated successfully for {}", params.event_name);
            Ok(MetricsCommandResult {
                success: true,
                metrics: Some(metrics),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to calculate metrics: {}", e);
            Ok(MetricsCommandResult {
                success: false,
                metrics: None,
                error: Some(format!("Calculation error: {}", e)),
            })
        }
    }
}

/// Commande Tauri : Charger les candles dans l'état
#[tauri::command]
pub async fn load_candles_for_metrics(
    candles: Vec<Candle>,
    candles_state: State<'_, CandlesState>,
) -> Result<String, String> {
    info!("Loading {} candles into state", candles.len());

    let mut candles_guard = candles_state
        .candles
        .lock()
        .map_err(|e| format!("Failed to lock candles: {}", e))?;

    *candles_guard = candles;

    Ok(format!("Loaded {} candles", candles_guard.len()))
}

/// Commande Tauri : Obtenir les symboles disponibles
#[tauri::command]
pub async fn get_available_symbols(
    candles_state: State<'_, CandlesState>,
) -> Result<Vec<String>, String> {
    let candles_guard = candles_state
        .candles
        .lock()
        .map_err(|e| format!("Failed to lock candles: {}", e))?;

    Ok(get_unique_symbols(&candles_guard))
}

/// Utilitaire : extraire les symboles uniques
fn get_unique_symbols(candles: &[Candle]) -> Vec<String> {
    let mut symbols: Vec<String> = candles
        .iter()
        .map(|c| c.symbol.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    symbols.sort();
    symbols
}

/// Commande Tauri : Vider les candles de la mémoire
#[tauri::command]
pub async fn clear_candles(candles_state: State<'_, CandlesState>) -> Result<String, String> {
    let mut candles_guard = candles_state
        .candles
        .lock()
        .map_err(|e| format!("Failed to lock candles: {}", e))?;

    let count = candles_guard.len();
    candles_guard.clear();

    info!("Cleared {} candles from memory", count);
    Ok(format!("Cleared {} candles", count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_unique_symbols() {
        let candles = vec![
            Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: Utc::now(),
                open: 1.1,
                high: 1.1,
                low: 1.1,
                close: 1.1,
                volume: 100.0,
            },
            Candle {
                id: None,
                symbol: "GBPUSD".to_string(),
                datetime: Utc::now(),
                open: 1.3,
                high: 1.3,
                low: 1.3,
                close: 1.3,
                volume: 100.0,
            },
            Candle {
                id: None,
                symbol: "EURUSD".to_string(),
                datetime: Utc::now(),
                open: 1.1,
                high: 1.1,
                low: 1.1,
                close: 1.1,
                volume: 100.0,
            },
        ];

        let symbols = get_unique_symbols(&candles);
        assert_eq!(symbols.len(), 2);
        assert!(symbols.contains(&"EURUSD".to_string()));
        assert!(symbols.contains(&"GBPUSD".to_string()));
    }
}
