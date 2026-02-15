use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedEvent {
    pub id: String,
    pub time: String,
    pub name: String,
    pub currency: String,
    pub impact: String,
    pub pair: String, // The pair this projection is for
    pub confidence_score: f64,
    pub source: String,
    pub has_history: bool,
    pub occurrence_count: i64,
}
