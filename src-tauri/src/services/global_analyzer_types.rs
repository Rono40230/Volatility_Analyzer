// services/global_analyzer_types.rs - Types de désérialisation pour l'analyseur global
use serde::Deserialize;

// Structure interne pour désérialiser uniquement ce dont on a besoin de manière très permissive
#[derive(Debug, Deserialize)]
pub struct AnalyzableGlobalMetrics {
    #[serde(default)]
    pub mean_volatility: f64,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzableArchiveData {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub best_hours: Vec<u8>,
    #[serde(default)]
    pub confidence_score: f64,
    #[serde(default)]
    pub global_metrics: Option<AnalyzableGlobalMetrics>,
}

// Wrapper pour gérer la structure réelle des archives stockées
#[derive(Debug, Deserialize)]
pub struct ArchiveWrapper {
    #[serde(rename = "analysisResult")]
    pub analysis_result: AnalyzableArchiveData,
}

// Structures pour désérialiser les archives de corrélation événement/paire
#[derive(Debug, Deserialize)]
pub struct EventImpactArchive {
    #[serde(rename = "eventImpact")]
    pub event_impact: EventImpactData,
}

#[derive(Debug, Deserialize)]
pub struct EventImpactData {
    pub event_name: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub event_count: usize,
    #[serde(default)]
    pub pair_impacts: Vec<PairImpact>,
}

#[derive(Debug, Deserialize)]
pub struct PairImpact {
    pub symbol: String,
    pub event_volatility: f64,
    pub baseline_volatility: f64,
}

// Structures pour désérialiser les archives de corrélation paire/événement
#[derive(Debug, Deserialize)]
pub struct PairCorrelationArchive {
    #[serde(rename = "pairCorrelation")]
    pub pair_correlation: PairCorrelationData,
}

#[derive(Debug, Deserialize)]
pub struct PairCorrelationData {
    pub pair: String,
    #[serde(default)]
    pub events: Vec<PairEvent>,
}

#[derive(Debug, Deserialize)]
pub struct PairEvent {
    pub name: String,
    pub count: usize,
    #[serde(default)]
    pub volatility_total: f64,
    #[serde(default)]
    pub volatility_before: f64,
    #[serde(default)]
    pub volatility_after: f64,
}

// Structure interne pour données pondérées
pub struct WeightedArchiveData {
    pub data: AnalyzableArchiveData,
    pub weight: f64,
    pub created_at: chrono::NaiveDateTime,
}
