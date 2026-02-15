// commands/dukascopy_commands.rs
// Commandes Tauri pour télécharger les données Dukascopy.

use crate::commands::candle_db_writer;
use crate::commands::tick_import_commands::ImportTickResult;
use crate::services::dukascopy_downloader::{self, DownloadProgress};
use crate::services::dukascopy_instruments;
use crate::services::tick_aggregator::{self, RawTick};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tracing::info;

/// Information instrument pour le frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentInfo {
    pub id: String,
    pub display: String,
    pub category: String,
}

/// Retourne la liste des instruments disponibles, synchronisée avec les conversions.
#[tauri::command]
pub async fn get_dukascopy_instruments(
    state: tauri::State<'_, crate::commands::pair_data::PairDataState>,
) -> Result<Vec<InstrumentInfo>, String> {
    use crate::services::pair_data::conversion_db;
    
    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")? ;
        pool_opt.clone().ok_or("DB pool not initialized")?
    };

    // 1. Récupérer les réglages personnalisés (masqués ou non)
    let all_custom = conversion_db::get_all_conversions(&pool)
        .map_err(|e| format!("DB error: {}", e))?;
    
    // 2. Récupérer tous les instruments par défaut de Dukascopy
    let all_duka = dukascopy_instruments::get_instruments();
    
    let mut results = Vec::new();

    // 3. Ajouter les instruments Dukascopy s'ils ne sont pas masqués
    for inst in all_duka {
        let custom = all_custom.iter().find(|c| c.symbol.eq_ignore_ascii_case(inst.id));
        
        // Si la paire est masquée en BD, on l'exclut du dropdown
        if let Some(c) = custom {
            if c.hidden { continue; }
        }

        results.push(InstrumentInfo {
            id: inst.id.to_string(),
            display: inst.display.to_string(),
            category: inst.category.to_string(),
        });
    }

    // 4. Ajouter les paires "Custom" créées par l'utilisateur qui ne sont pas déjà dans Dukascopy
    for custom in all_custom {
        if custom.hidden { continue; }
        
        let upper = custom.symbol.to_uppercase();
        if !results.iter().any(|r| r.id == upper) {
            results.push(InstrumentInfo {
                id: upper.clone(),
                display: format!("{} (Perso)", upper),
                category: "Custom / Others".to_string(),
            });
        }
    }

    Ok(results)
}

/// Télécharge les données tick Dukascopy, agrège en M1 et sauvegarde en BD.
#[tauri::command]
pub async fn download_dukascopy_data(
    app: tauri::AppHandle,
    symbol: String,
    date_from: String,
    date_to: String,
) -> Result<ImportTickResult, String> {
    // Validation symbole
    let instrument = dukascopy_instruments::find_instrument(&symbol)
        .ok_or_else(|| format!("Symbole non supporté : {}", symbol))?;

    // Validation dates
    let from = NaiveDate::parse_from_str(&date_from, "%Y-%m-%d")
        .map_err(|e| format!("Date début invalide '{}' : {}", date_from, e))?;
    let to = NaiveDate::parse_from_str(&date_to, "%Y-%m-%d")
        .map_err(|e| format!("Date fin invalide '{}' : {}", date_to, e))?;

    if from > to {
        return Err("La date de début doit être avant la date de fin".into());
    }

    let days = (to - from).num_days();
    if days > 730 {
        return Err(format!(
            "Plage trop grande ({} jours). Maximum : 730 jours (2 ans)",
            days
        ));
    }

    info!("📡 Téléchargement Dukascopy : {} du {} au {}", symbol, date_from, date_to);

    // Télécharger avec callback de progression
    let app_clone = app.clone();
    let dukascopy_ticks = dukascopy_downloader::download_range(
        &symbol,
        from,
        to,
        instrument.point_value,
        move |progress: DownloadProgress| {
            let _ = app_clone.emit("dukascopy-progress", &progress);
        },
    )
    .await?;

    if dukascopy_ticks.is_empty() {
        return Err("Aucun tick téléchargé pour cette période".into());
    }

    // Conversion DukascopyTick → RawTick (couche commande = glue entre services)
    let raw_ticks: Vec<RawTick> = dukascopy_ticks
        .into_iter()
        .map(|t| RawTick {
            datetime_utc: t.datetime_utc,
            bid: t.bid,
            ask: t.ask,
            bid_volume: t.bid_volume as f64,
            ask_volume: t.ask_volume as f64,
        })
        .collect();

    let total_ticks = raw_ticks.len();
    info!("🔄 Agrégation de {} ticks en M1…", total_ticks);

    let candles = tick_aggregator::aggregate_to_m1(&raw_ticks);
    if candles.is_empty() {
        return Err("Aucune bougie M1 générée".into());
    }

    let avg_spread = candles.iter().map(|c| c.spread_mean).sum::<f64>() / candles.len() as f64;
    let avg_ticks = total_ticks as f64 / candles.len() as f64;
    let date_start = candles.first().map(|c| c.datetime_utc.to_rfc3339()).unwrap_or_default();
    let date_end = candles.last().map(|c| c.datetime_utc.to_rfc3339()).unwrap_or_default();

    // Sauvegarde en BD
    let source = format!("dukascopy-{}-{}-{}", symbol, date_from, date_to);
    let conn = candle_db_writer::open_pairs_db()?;
    candle_db_writer::save_enriched_candles(&conn, &symbol, &candles, &source)?;

    info!("✅ Import Dukascopy terminé : {} M1 pour {}", candles.len(), symbol);

    Ok(ImportTickResult {
        symbol,
        minutes_generated: candles.len(),
        total_ticks,
        date_start,
        date_end,
        avg_spread,
        avg_ticks_per_minute: avg_ticks,
    })
}
