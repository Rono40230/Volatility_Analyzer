// models/global_analysis.rs - Structures pour l'analyse globale (IA Statistique)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFilters {
    pub start_date: Option<String>, // Format ISO "YYYY-MM-DD"
    pub end_date: Option<String>,   // Format ISO "YYYY-MM-DD"
    pub pairs: Option<Vec<String>>, // Liste des symboles à inclure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAnalysisResult {
    pub total_analyses: usize,
    pub total_days_analyzed: usize,
    pub filters_applied: Option<AnalysisFilters>, // Pour confirmer les filtres utilisés
    pub global_stats: GlobalStats,
    pub best_pairs: Vec<BestPair>,
    pub golden_hours: Vec<GoldenHour>,
    pub event_impacts: Vec<EventImpact>,
    pub tradable_events: Vec<TradableEventType>,
    pub pair_straddle_rates: Vec<StraddleSuccessRate>,
    pub optimal_time_windows: Vec<OptimalTimeWindow>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub average_volatility: f64,
    pub average_confidence: f64,
    pub most_analyzed_pair: String,
    pub most_frequent_recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPair {
    pub symbol: String,
    pub score: f64, // Score composite (Volatilité * Confiance)
    pub avg_volatility: f64,
    pub win_rate: f64, // Basé sur la qualité des mouvements
    pub analysis_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenHour {
    pub hour: u8,
    pub score: f64, // Fréquence d'apparition dans "best_hours"
    pub avg_volatility: f64,
    pub reliability: f64, // % de fois où cette heure est positive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventImpact {
    pub event_name: String,
    pub currency: String,
    pub avg_impact_pips: f64,
    pub occurrence_count: usize,
    pub impact_level: String, // "High", "Medium", "Low"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradableEventType {
    pub event_name: String,
    pub occurrence_count: usize,
    pub avg_volatility_increase: f64, // Ratio event_volatility / baseline_volatility
    pub tradability_score: f64,       // Score composite (0-100)
    pub affected_pairs: Vec<String>,  // Liste des paires impactées
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraddleSuccessRate {
    pub pair: String,
    pub total_events: usize,
    pub directional_move_rate: f64, // % de mouvements directionnels clairs
    pub non_event_rate: f64,        // % d'événements à faible volatilité (non-événements)
    pub avg_volatility: f64,        // Volatilité moyenne sur tous les événements
    pub straddle_score: f64, // Score composite (0-100) : directional_move_rate - non_event_rate
    pub top_events: Vec<String>, // Top 3 événements les plus impactants pour cette paire
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalTimeWindow {
    pub event_type: String, // Type d'événement (ex: "NFP", "CPI", "Fed Rate")
    pub occurrence_count: usize,
    pub avg_peak_time_minutes: f64, // Temps moyen pour atteindre le pic de volatilité (en minutes après l'événement)
    pub avg_entry_window_minutes: f64, // Fenêtre optimale d'entrée (en minutes avant l'événement)
    pub avg_return_to_normal_minutes: f64, // Temps moyen pour revenir à la normale
    pub consistency_score: f64, // Score de consistance (0-100) : à quel point ces timings sont fiables
    pub affected_pairs: Vec<String>, // Paires affectées par cet événement
}
