// commands/volatility/straddle_metrics.rs - Command pour analyse Straddle
use super::straddle_metrics_types::*;
use tauri::command;

/// Obtenir le nombre de points par pip selon le symbole
/// Basé sur la norme MT5:
/// - Forex (EURUSD, CADJPY, USDJPY, etc): 1 pip = 10 points
/// - Or (XAUUSD, XAUJPY): 1 pip = 10 points
/// - Argent (XAGUSD): 1 pip = 1000 points
/// - Indices (USA500IDXUSD, etc): 1 pip = 1 point
/// - Crypto (BTCUSD, ETHUSD): 1 pip = 1 point
fn get_points_per_pip(symbol: &str) -> f64 {
    if symbol.contains("XAU") {
        10.0  // Or: 1 pip = 10 points
    } else if symbol.contains("XAG") {
        1000.0  // Argent: 1 pip = 1000 points
    } else if symbol.contains("US30") || symbol.contains("DE30") || symbol.contains("NAS100") || symbol.contains("SPX500") || symbol.contains("USA500") {
        1.0  // Indices: 1 pip = 1 point
    } else if symbol.contains("BTC") || symbol.contains("ETH") {
        1.0  // Crypto: 1 pip = 1 point
    } else {
        10.0  // Forex par défaut (EURUSD, CADJPY, USDJPY, etc): 1 pip = 10 points
    }
}

/// Convertir des pips en points
fn pips_to_points(pips: f64, symbol: &str) -> f64 {
    pips * get_points_per_pip(symbol)
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
    let data_dir =
        dirs::data_local_dir().ok_or_else(|| "Failed to get data directory".to_string())?;
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
    let (metrics, candles) =
        analyze_slice_metrics(&candle_index, &symbol, hour as u32, quarter as u32)?;

    // Simuler la stratégie Straddle sur les bougies historiques
    use crate::services::straddle_simulator::simulate_straddle;
    let simulation = simulate_straddle(&candles, &symbol);

    // Convertir les détails des whipsaws (si disponibles)
    let whipsaw_details: Vec<WhipsawDetailResponse> = simulation
        .whipsaw_details
        .iter()
        .map(|w| {
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
        })
        .collect();

    // Le meilleur moment d'entrée est calculé via entryWindowAnalysis (non pondéré)
    // Ne pas recalculer basé sur les whipsaws pour préserver la stabilité

    Ok(StraddleMetricsResponse {
        symbol: symbol.clone(),
        hour,
        candle_count: metrics.candle_count,
        offset_optimal: OptimalOffsetData {
            offset_points: pips_to_points(simulation.offset_optimal_pips, &symbol),
            percentile_95_wicks: simulation.percentile_95_wicks,
            with_margin: pips_to_points(simulation.offset_optimal_pips * 1.1, &symbol),
            sl_adjusted_points: pips_to_points(simulation.sl_adjusted_pips, &symbol),
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
            sl_adjusted_points: pips_to_points(simulation.sl_adjusted_pips, &symbol),
            win_rate_adjusted: simulation.win_rate_adjusted,
            trailing_stop_adjusted: pips_to_points(simulation.trailing_stop_adjusted, &symbol),
            timeout_adjusted_minutes: simulation.timeout_adjusted_minutes,
            whipsaw_details,
        },
    })
}
