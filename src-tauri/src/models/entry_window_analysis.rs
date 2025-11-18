// models/entry_window_analysis.rs - Analyse de fenêtre d'entrée optimale
// Conforme .clinerules : < 150L, structures uniquement

use serde::{Deserialize, Serialize};

/// Analyse de performance pour un offset d'entrée spécifique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryOffsetMetrics {
    pub minutes_before_event: i32, // -60, -30, -15, -5, -1
    pub sample_count: usize,       // Nombre d'occurrence de cet événement analysées
    pub winning_entries: usize,    // Entrées profitables
    pub losing_entries: usize,     // Entrées perdantes
    pub win_rate: f64,             // %  (0.0 - 1.0)
    pub avg_pips_gained: f64,      // Pips moyens gagnés (peut être négatif)
    pub avg_pips_lost: f64,        // Pips moyens perdus
    pub max_pips_gained: f64,      // Meilleur cas
    pub max_pips_lost: f64,        // Pire cas
    pub profit_factor: f64,        // (total_gain / total_loss) - > 1.0 = profitable
}

/// Résultat de l'analyse complète de fenêtre d'entrée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryWindowAnalysisResult {
    pub symbol: String,
    pub event_type: String,
    pub offsets: Vec<EntryOffsetMetrics>,
    pub optimal_offset: i32, // Minutes avant événement recommandé
    pub optimal_win_rate: f64,
    pub analysis_timestamp: i64, // Unix timestamp ts_seconds
    pub total_events_analyzed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_window_result_creation() {
        let result = EntryWindowAnalysisResult {
            symbol: "ADAUSD".to_string(),
            event_type: "FOMC Member Barkin Speaks".to_string(),
            offsets: vec![],
            optimal_offset: -5,
            optimal_win_rate: 0.65,
            analysis_timestamp: 1763321992,
            total_events_analyzed: 10,
        };
        assert_eq!(result.symbol, "ADAUSD");
        assert_eq!(result.optimal_offset, -5);
    }
}
