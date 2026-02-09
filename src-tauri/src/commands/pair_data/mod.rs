mod processor;
pub mod conversion_commands;

use crate::db::DbPool;
pub use processor::process_single_file;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::info;

pub struct PairDataState {
    #[allow(dead_code)]
    pub pool: Mutex<Option<DbPool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub pairs_updated: Vec<String>,
    pub timeframes: Vec<String>,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn import_pair_data(
    state: tauri::State<'_, PairDataState>,
    paths: Vec<String>,
) -> Result<ImportSummary, String> {
    use std::path::Path;

    info!("📥 ========== DÉBUT IMPORT PAIR DATA ==========");
    info!("📥 Import de {} fichiers de paires vers BD", paths.len());
    for (idx, path) in paths.iter().enumerate() {
        info!("   [{}] {}", idx + 1, path);
    }

    let mut summary = ImportSummary {
        total_files: paths.len(),
        successful: 0,
        failed: 0,
        pairs_updated: Vec::new(),
        timeframes: Vec::new(),
        errors: Vec::new(),
    };

    info!("🔐 Tentative d'accès au pool DB paires...");
    let pool = {
        let pool_opt = state.pool.lock().map_err(|_| "Failed to lock pool")?;
        match pool_opt.clone() {
            Some(p) => {
                info!("✅ Pool DB obtenu avec succès");
                p
            }
            None => {
                return Err("DB pool not initialized".to_string());
            }
        }
    };

    for (file_idx, path) in paths.into_iter().enumerate() {
        info!(
            "🔄 Traitement fichier [{}/{}]: {}",
            file_idx + 1,
            summary.total_files,
            path
        );

        match process_single_file(&path, &pool) {
            Ok((pair, timeframe, row_count)) => {
                summary.successful += 1;

                if !summary.pairs_updated.contains(&pair) {
                    summary.pairs_updated.push(pair.clone());
                }

                if !summary.timeframes.contains(&timeframe) {
                    summary.timeframes.push(timeframe.clone());
                }

                info!(
                    "✅ [{}/{}] Fichier importé avec succès: {} ({} lignes)",
                    file_idx + 1,
                    summary.total_files,
                    path,
                    row_count
                );
            }
            Err(e) => {
                summary.failed += 1;
                let file_name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let error_msg = format!("{}: {}", file_name, e);
                summary.errors.push(error_msg);
            }
        }
    }

    info!("📊 ========== IMPORT TERMINÉ ==========");
    info!(
        "📊 Résumé final: {} succès, {} échecs sur {} fichiers",
        summary.successful, summary.failed, summary.total_files
    );
    info!("📊 Paires mises à jour: {:?}", summary.pairs_updated);
    info!("📊 Timeframes: {:?}", summary.timeframes);
    if !summary.errors.is_empty() {
        info!("📊 Erreurs rencontrées:");
        for (idx, err) in summary.errors.iter().enumerate() {
            info!("   [{}] {}", idx + 1, err);
        }
    }

    Ok(summary)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolProperties {
    pub point_value: f64,
    pub pip_value: f64,
}

#[tauri::command]
pub fn get_symbol_properties(symbol: String) -> SymbolProperties {
    use crate::services::pair_data::{get_point_value, get_pip_value};
    
    SymbolProperties {
        point_value: get_point_value(&symbol),
        pip_value: get_pip_value(&symbol),
    }
}
