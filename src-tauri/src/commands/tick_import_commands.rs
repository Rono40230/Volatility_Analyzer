// commands/tick_import_commands.rs
// Commande Tauri pour importer un fichier tick Dukascopy → M1 enrichies en BD.

use crate::commands::candle_db_writer;
use crate::services::tick_aggregator;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Résultat retourné au frontend après import tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTickResult {
    pub symbol: String,
    pub minutes_generated: usize,
    pub total_ticks: usize,
    pub date_start: String,
    pub date_end: String,
    pub avg_spread: f64,
    pub avg_ticks_per_minute: f64,
}

/// Importe un fichier tick Dukascopy (bid+ask) et stocke les M1 enrichies en BD.
#[tauri::command]
pub async fn import_tick_file(file_path: String) -> Result<ImportTickResult, String> {
    if file_path.is_empty() {
        return Err("Chemin de fichier vide".to_string());
    }

    // 1. Agréger les ticks en M1 enrichies
    let result = tick_aggregator::aggregate_ticks_to_m1(&file_path)?;

    if result.candles.is_empty() {
        return Err("Aucune bougie M1 générée à partir des ticks".to_string());
    }

    // 2. Sauvegarder en BD via le writer partagé
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("tick_import");

    let conn = candle_db_writer::open_pairs_db()?;
    candle_db_writer::save_enriched_candles(&conn, &result.symbol, &result.candles, filename)?;

    info!(
        "✅ Import tick terminé : {} M1 enrichies pour {} (spread moyen {:.5})",
        result.candles.len(),
        result.symbol,
        result.avg_spread
    );

    Ok(ImportTickResult {
        symbol: result.symbol,
        minutes_generated: result.candles.len(),
        total_ticks: result.total_ticks,
        date_start: result.date_start,
        date_end: result.date_end,
        avg_spread: result.avg_spread,
        avg_ticks_per_minute: result.avg_ticks_per_minute,
    })
}
