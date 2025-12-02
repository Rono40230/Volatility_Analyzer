// commands/volatility/straddle_metrics.rs - Command pour analyse Straddle
use super::straddle_metrics_types::*;
use tauri::command;

/// Calcule le meilleur moment d'entrée basé sur les trigger_minutes des whipsaws
/// Logique: moyenne des moments où les whipsaws se déclenchent
/// Pour éviter les whipsaws, on entre 1-2 minutes avant ce moment
fn calculate_optimal_entry_minutes(whipsaw_details: &[WhipsawDetailResponse]) -> i32 {
    if whipsaw_details.is_empty() {
        return 1; // Par défaut: 1 minute après le début
    }

    // Filtrer les trigger_minutes valides (> 0)
    let valid_triggers: Vec<i32> = whipsaw_details
        .iter()
        .filter(|w| w.trigger_minute > 0)
        .map(|w| w.trigger_minute)
        .collect();

    if valid_triggers.is_empty() {
        return 1; // Pas de données valides
    }

    // Calculer la moyenne des moments de trigger
    let mean_trigger = valid_triggers.iter().sum::<i32>() as f64 / valid_triggers.len() as f64;

    // Entrer 30-40% avant le moment moyen du whipsaw (sécurité)
    let optimal = (mean_trigger * 0.6) as i32;

    // Minimum 0, maximum avant timeout (25 min avec buffer)
    optimal.max(0).min(25)
}

/// Analyse complète Straddle: offset, win_rate, whipsaw
/// Candles doivent être pré-chargées (60x 1min) depuis DB
#[command]
pub async fn analyze_straddle_metrics(
    symbol: String,
    hour: u8,
    quarter: u8,
) -> Result<StraddleMetricsResponse, String> {
    use crate::db;
    use crate::services::candle_index::CandleIndex;
    use crate::services::database_loader::DatabaseLoader;
    use crate::services::slice_metrics_analyzer::analyze_slice_metrics;

    // Créer le pool de connexions pour la BD paires
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())?;
    let pairs_db_path = data_dir.join("volatility-analyzer").join("pairs.db");
    let pairs_db_url = format!("sqlite://{}", pairs_db_path.display());
    
    let pairs_pool = db::create_pool(&pairs_db_url)
        .map_err(|e| format!("Failed to create pairs DB pool: {}", e))?;

    // Créer un CandleIndex avec DatabaseLoader
    let db_loader = DatabaseLoader::new(pairs_pool);
    let mut candle_index = CandleIndex::with_db_loader(db_loader);

    // Charger les bougies pour ce symbole
    candle_index
        .load_pair_candles(&symbol)
        .map_err(|e| format!("Failed to load candles for {}: {}", symbol, e))?;

    // Analyser les métriques du créneau et récupérer les bougies
    let (metrics, candles) = analyze_slice_metrics(&candle_index, &symbol, hour as u32, quarter as u32)?;

    // Simuler la stratégie Straddle sur les bougies historiques
    use crate::services::straddle_simulator::simulate_straddle;
    let simulation = simulate_straddle(&candles, &symbol);

    // Convertir les détails des whipsaws (si disponibles)
    let whipsaw_details: Vec<WhipsawDetailResponse> = simulation.whipsaw_details.iter().map(|w| {
        // Calculer trigger_minute par le temps réel, pas par l'index
        let entry_time = candles[w.entry_index].datetime;
        let max_trigger_idx = w.buy_trigger_index.max(w.sell_trigger_index);
        
        let trigger_minute = if max_trigger_idx < candles.len() {
            let trigger_time = candles[max_trigger_idx].datetime;
            let duration = trigger_time.signed_duration_since(entry_time);
            duration.num_minutes() as i32
        } else {
            -1
        };
        
        WhipsawDetailResponse {
            entry_candle_index: w.entry_index,
            trigger_minute,
            entry_price: w.entry_price,
            buy_stop: w.buy_stop,
            sell_stop: w.sell_stop,
        }
    }).collect();

    // Calculer le meilleur moment d'entrée basé sur les whipsaws
    let optimal_entry_minutes = calculate_optimal_entry_minutes(&whipsaw_details);

    Ok(StraddleMetricsResponse {
        symbol,
        hour,
        candle_count: metrics.candle_count,
        offset_optimal: OptimalOffsetData {
            offset_pips: simulation.offset_optimal_pips,
            percentile_95_wicks: simulation.percentile_95_wicks,
            with_margin: simulation.offset_optimal_pips * 1.1,
            sl_adjusted_pips: simulation.sl_adjusted_pips,
        },
        win_rate: WinRateData {
            total_trades: simulation.total_trades,
            wins: simulation.wins,
            losses: simulation.losses,
            whipsaws: simulation.whipsaws,
            win_rate_percentage: simulation.win_rate_percentage,
            win_rate_adjusted: simulation.win_rate_adjusted,
        },
        whipsaw: WhipsawData {
            total_trades: simulation.total_trades,
            whipsaw_count: simulation.whipsaws,
            whipsaw_frequency_percentage: simulation.whipsaw_frequency_percentage,
            risk_level: simulation.risk_level,
            risk_color: simulation.risk_color,
            sl_adjusted_pips: simulation.sl_adjusted_pips,
            win_rate_adjusted: simulation.win_rate_adjusted,
            trailing_stop_adjusted: simulation.trailing_stop_adjusted,
            timeout_adjusted_minutes: simulation.timeout_adjusted_minutes,
            optimal_entry_minutes,
            whipsaw_details,
        },
    })
}
