// models/mod.rs - Module principal des modèles de données
// Niveau 4 de l'architecture DAG

pub mod analysis_result;
pub mod calendar_event;
pub mod candle;
pub mod errors;
pub mod event_metrics;
pub mod hourly_stats;
pub mod stats_15min;

// Re-exports pour faciliter les imports
pub use analysis_result::{
    AnalysisResult, CorrelatedEvent, GlobalMetrics, RiskLevel, TradingRecommendation,
};
pub use calendar_event::CalendarEvent;
pub use candle::Candle;
pub use errors::{Result, VolatilityError};
pub use event_metrics::EventMetrics;
pub use hourly_stats::{EventInHour, HourlyStats};
pub use stats_15min::Stats15Min;
