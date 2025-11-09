// models/event_metrics.rs - Métriques événementielles pour optimisation EA Straddle
// Conforme .clinerules : < 150L, structures uniquement, pas de logique métier

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Métriques complètes calculées pour un événement économique spécifique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    pub id: Option<i32>,
    pub event_name: String,
    pub event_time: DateTime<Utc>,
    pub symbol: String,
    pub peak_duration_minutes: i32,
    pub return_to_normal_minutes: i32,
    pub peak_time_minutes: i64,
    pub baseline_atr: f64,
    pub win_rate: f64,
    pub loss_rate: f64,
    pub whipsaw_rate: f64,
    pub risk_reward_ratio: f64,
    pub best_entry_minutes_before: i32,
    pub best_entry_win_rate: f64,
    pub worst_entry_minutes_before: i32,
    pub worst_entry_win_rate: f64,
    pub atr_before_event: f64,
    pub atr_after_event: f64,
    pub atr_ratio: f64,
    pub max_atr_spike: f64,
    pub recommended_sl_multiplier: f64,
    pub recommended_tp_multiplier: f64,
    pub baseline_volatility: String,
    pub sample_size: i32,
    pub created_at: DateTime<Utc>,
}

/// Recommandation de trading
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingRecommendation {
    Trade,
    Caution,
    Avoid,
}

/// Analyse de fenêtre d'entrée optimale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryWindowAnalysis {
    pub optimal_entry_minutes_before: i32,
    pub win_rate_at_optimal: f64,
    pub alternative_entries: Vec<(i32, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_enum_exists() {
        // Simple test juste pour vérifier que les variants existent
        let _ = TradingRecommendation::Trade;
        let _ = TradingRecommendation::Caution;
        let _ = TradingRecommendation::Avoid;
    }
}
