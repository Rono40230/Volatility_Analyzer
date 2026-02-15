// commands/entry_point_commands.rs
// Commande Tauri pour l'analyse de point d'entrée optimal (Phase 2).
// Charge toutes les candles M1 d'un symbole et délègue au service entry_point_analyzer.

use crate::models::entry_analysis::{EntryAnalysisConfig, EntryAnalysisResult};
use serde::Deserialize;
use tauri::command;

/// Paramètres optionnels envoyés depuis le frontend
#[derive(Debug, Deserialize)]
pub struct EntryPointParams {
    pub symbol: String,
    pub hour: u8,
    pub quarter: u8,
    /// Type d'événement (ex: "NFP", "CPI"). Défaut: "quarter" si absent
    #[serde(default = "default_event_type")]
    pub event_type: String,
    /// Minutes forward à analyser (défaut: 30)
    #[serde(default = "default_forward")]
    pub forward_minutes: Option<usize>,
    /// Seuil spread en pips pour zone non-tradable (défaut: 10.0)
    #[serde(default)]
    pub spread_threshold_pips: Option<f64>,
    /// Échantillons minimum par offset (défaut: 5)
    #[serde(default)]
    pub min_samples: Option<usize>,
}

fn default_event_type() -> String {
    "quarter".to_string()
}
fn default_forward() -> Option<usize> {
    None
}

/// Analyse le point d'entrée optimal pour un quarter d'un symbole donné.
///
/// Retourne un `EntryAnalysisResult` avec le profit net réel (après spread),
/// le vrai win rate par comptage, les zones non-tradables et le profil de mouvement.
#[command]
pub async fn analyze_entry_points(params: EntryPointParams) -> Result<EntryAnalysisResult, String> {
    use crate::db;
    use crate::services::candle_index::CandleIndex;
    use crate::services::database_loader::DatabaseLoader;
    use crate::services::entry_point_analyzer;

    tracing::info!(
        "🎯 Entry point analysis: {} @ {:02}:{:02} (quarter={}) event={}",
        params.symbol,
        params.hour,
        params.quarter * 15,
        params.quarter,
        params.event_type
    );

    // Validation
    if params.symbol.is_empty() {
        return Err("Symbole requis".to_string());
    }
    if params.hour > 23 {
        return Err(format!("Heure invalide: {}", params.hour));
    }
    if params.quarter > 3 {
        return Err(format!("Quarter invalide: {} (0-3)", params.quarter));
    }

    // Ouvrir la BD paires
    let data_dir =
        dirs::data_local_dir().ok_or_else(|| "Répertoire de données introuvable".to_string())?;
    let pairs_db_path = data_dir.join("volatility-analyzer").join("pairs.db");
    let pairs_db_url = format!("sqlite://{}", pairs_db_path.display());

    let pairs_pool = db::create_pool(&pairs_db_url)
        .map_err(|e| format!("Connexion BD paires échouée: {e}"))?;

    // Charger toutes les candles M1 du symbole
    let db_loader = DatabaseLoader::new(pairs_pool);
    let mut candle_index = CandleIndex::with_db_loader(db_loader);

    candle_index
        .load_pair_candles(&params.symbol)
        .map_err(|e| format!("Chargement paire {} échoué: {e}", params.symbol))?;

    let all_candles = candle_index
        .get_all_candles(&params.symbol)
        .unwrap_or_default();

    if all_candles.is_empty() {
        return Err(format!(
            "Aucun candle trouvé pour {}",
            params.symbol
        ));
    }

    tracing::info!(
        "📊 {} candles chargées pour {}",
        all_candles.len(),
        params.symbol
    );

    // Configuration
    let config = EntryAnalysisConfig {
        forward_minutes: params.forward_minutes.unwrap_or(30),
        spread_threshold_pips: params.spread_threshold_pips.unwrap_or(10.0),
        min_samples: params.min_samples.unwrap_or(5),
    };

    // Analyse
    let result = entry_point_analyzer::analyze_entry_points(
        &all_candles,
        &params.symbol,
        &params.event_type,
        params.hour,
        params.quarter,
        &config,
    )
    .map_err(|e| format!("Analyse échouée: {e}"))?;

    tracing::info!(
        "✅ Entry point: offset={}min, win_rate={:.1}%, profit={:.1} pips, spread={:.1} pips",
        result.optimal_offset_minutes,
        result.real_win_rate * 100.0,
        result.avg_net_profit_pips,
        result.avg_spread_at_entry_pips
    );

    Ok(result)
}
