// commands/volatility/pair_correlation_command.rs
// Commande Tauri pour vérifier les corrélations inter-paires

use crate::services::pair_correlation::{analyze_pair_correlations, PairCorrelationResult};
use tauri::command;

/// Vérifie les corrélations entre les paires sélectionnées.
/// Renvoie des warnings si des paires corrélées >0.7 sont tradées ensemble.
#[command]
pub fn check_pair_correlations(
    pairs: Vec<String>,
) -> Result<PairCorrelationResult, String> {
    if pairs.is_empty() {
        return Err("Aucune paire fournie".into());
    }

    tracing::info!("Checking pair correlations for {} pairs: {:?}", pairs.len(), pairs);

    let result = analyze_pair_correlations(&pairs);

    if !result.warnings.is_empty() {
        tracing::warn!(
            "⚠️ {} corrélation(s) dangereuse(s) détectée(s), diversification: {:.0}%",
            result.warnings.len(),
            result.diversification_score
        );
    }

    Ok(result)
}
