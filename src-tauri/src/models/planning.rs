use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedEvent {
    pub id: String,
    pub time: String,
    pub name: String,
    pub currency: String,
    pub impact: String,
    pub pair: String, // The pair this projection is for
    pub offset: f64,
    pub tp: f64,
    pub sl: f64,
    pub confidence_score: f64,
    pub source: String,
}
