// commands/volatility/analyze_quarter_entry_timing_command.rs
// Commande pour analyser le meilleur moment d'entrée dans un quarter spécifique
// Teste chaque minute du quarter sur tout l'historique et retourne l'offset optimal

use super::analyze_quarter_entry_timing_helpers::*;
use crate::commands::pair_data::PairDataState;
use serde::{Deserialize, Serialize};
use tauri::{command, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct QuarterEntryTimingResponse {
    pub symbol: String,
    pub hour: u8,
    pub quarter: u8,
    pub optimal_offset_minutes: u8, // 0-14 minutes dans le quarter
    pub optimal_win_rate: f64,      // 0-1
    pub total_occurrences_analyzed: usize,
    pub confidence_score: f64, // 0-100: comment confident on est dans ce résultat
}

#[command]
pub async fn analyze_quarter_entry_timing(
    symbol: String,
    hour: u8,
    quarter: u8,
    pair_state: State<'_, PairDataState>,
) -> Result<QuarterEntryTimingResponse, String> {
    use crate::services::candle_index::CandleIndex;
    use crate::services::database_loader::DatabaseLoader;

    tracing::info!(
        "🎯 Analyse timing entrée: {} {}:{}* (quarter={})",
        symbol,
        hour,
        quarter * 15,
        quarter
    );

    let pool = pair_state
        .pool
        .lock()
        .map_err(|e| format!("Lock pair pool failed: {e}"))?
        .clone()
        .ok_or("Pair database pool not initialized")?;

    let db_loader = DatabaseLoader::new(pool);
    let mut candle_index = CandleIndex::with_db_loader(db_loader);

    // Charger la paire
    candle_index
        .load_pair_candles(&symbol)
        .map_err(|e| format!("Failed to load pair {}: {}", symbol, e))?;

    // Récupérer tous les candles du quarter (toutes les occurrences historiques)
    let start_minute = quarter as u32 * 15;
    let end_minute = start_minute + 15;

    let all_quarter_candles = candle_index.get_candles_for_slice_all_history(
        &symbol,
        hour as u32,
        start_minute,
        end_minute,
    );

    if all_quarter_candles.is_empty() {
        return Err(format!(
            "Aucun candle trouvé pour {}:{:02} (quarter {})",
            hour, start_minute, quarter
        ));
    }

    tracing::info!(
        "📊 Candles chargées: {} pour {}:{}* (quarter={})",
        all_quarter_candles.len(),
        hour,
        start_minute,
        quarter
    );

    // Grouper les candles par jour (chaque occurence du quarter)
    let daily_occurrences = group_candles_by_day(&all_quarter_candles);

    tracing::info!(
        "📅 Occurrences trouvées: {} jours différents",
        daily_occurrences.len()
    );

    // 1. Trouver le meilleur minute GLOBALEMENT (sur tout l'historique agrégé)
    // Cela correspond au "pic" visible sur le graphique moyen (évite le problème de la moyenne des indices)
    let global_best_minute = find_best_minute_in_quarter(&all_quarter_candles)?;

    tracing::info!(
        "🌍 Meilleur minute globale (pic moyen): +{} min",
        global_best_minute
    );

    // 2. Analyser la consistance jour par jour (pour la confiance et stats)
    let mut offsets: Vec<u8> = Vec::new();
    let mut total_win_rates: f64 = 0.0;

    for daily_candles in &daily_occurrences {
        if daily_candles.is_empty() {
            continue;
        }
        // Scorer chaque minute du quarter pour ce jour
        let best_minute = find_best_minute_in_quarter(daily_candles)?;
        offsets.push(best_minute);

        // Calculer win-rate pour ce minute
        let win_rate = estimate_win_rate_for_minute(daily_candles)?;
        total_win_rates += win_rate;
    }

    if offsets.is_empty() {
        return Err("Aucun offset valide trouvé".to_string());
    }

    // L'offset optimal est celui qui ressort le mieux sur l'ensemble de l'historique
    let optimal_offset = global_best_minute;

    // Calculer le win-rate moyen
    let avg_win_rate = total_win_rates / offsets.len() as f64;

    // Calculer la confiance (basée sur la consistance des offsets par rapport à l'optimal)
    let confidence = calculer_confiance(&offsets, optimal_offset);

    tracing::info!(
        "✅ Timing analysé: offset={}min, win_rate={:.1}%, confidence={:.0}%",
        optimal_offset,
        avg_win_rate * 100.0,
        confidence
    );

    Ok(QuarterEntryTimingResponse {
        symbol,
        hour,
        quarter,
        optimal_offset_minutes: optimal_offset,
        optimal_win_rate: avg_win_rate,
        total_occurrences_analyzed: offsets.len(),
        confidence_score: confidence,
    })
}
