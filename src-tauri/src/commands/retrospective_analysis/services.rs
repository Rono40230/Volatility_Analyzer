use super::impact_analyzer::ImpactAnalyzer;
use super::simple_analyzers::{DecayProfileAnalyzer, PeakDelayAnalyzer};

pub struct RetroAnalysisService;

impl RetroAnalysisService {
    pub fn new() -> Self {
        Self
    }

    pub async fn calculer_delai_pic(
        pair: &str,
        event_type: &str,
        events: &[crate::models::CalendarEvent],
        loader: &crate::services::DatabaseLoader,
    ) -> Result<(Vec<i16>, Vec<f64>), String> {
        PeakDelayAnalyzer::calculer(pair, event_type, events, loader).await
    }

    pub async fn calculer_profil_decroissance(
        pair: &str,
        events: &[crate::models::CalendarEvent],
        loader: &crate::services::DatabaseLoader,
    ) -> Result<(Vec<f64>, Vec<f64>), String> {
        DecayProfileAnalyzer::calculer(pair, events, loader).await
    }

    pub async fn calculer_impact_evenement(
        pair: &str,
        event_type: &str,
        events: &[crate::models::CalendarEvent],
        loader: &crate::services::DatabaseLoader,
    ) -> Result<super::types::EventImpactResult, String> {
        ImpactAnalyzer::calculer(pair, event_type, events, loader).await
    }
}

