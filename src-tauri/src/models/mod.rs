// models/mod.rs - Module principal des modèles de données
// Niveau 4 de l'architecture DAG

pub mod candle;
pub mod hourly_stats;
pub mod analysis_result;
pub mod calendar_event;
pub mod event_metrics;
pub mod errors;

// Re-exports pour faciliter les imports
pub use candle::Candle;
pub use hourly_stats::HourlyStats;
pub use analysis_result::{AnalysisResult, CorrelatedEvent, GlobalMetrics, TradingRecommendation, RiskLevel};
pub use calendar_event::CalendarEvent;
pub use event_metrics::{EventMetrics, EntryWindowAnalysis, TradingRecommendation as EventTradingRecommendation};
pub use errors::{VolatilityError, Result};
