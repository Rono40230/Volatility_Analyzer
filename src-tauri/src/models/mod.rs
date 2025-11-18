// models/mod.rs - Module principal des modèles de données
// Niveau 4 de l'architecture DAG

pub mod analysis_result;
pub mod calendar_event;
pub mod candle;
pub mod entry_window_analysis;
pub mod errors;
pub mod event_metrics;
pub mod event_movement_quality;
pub mod hourly_stats;
pub mod stats_15min;
pub mod trading_recommendation;
pub mod volatility_duration;

// Re-exports pour faciliter les imports
pub use analysis_result::{AnalysisResult, CorrelatedEvent, GlobalMetrics};
pub use calendar_event::CalendarEvent;
pub use candle::Candle;
pub use entry_window_analysis::{EntryOffsetMetrics, EntryWindowAnalysisResult};
pub use errors::{Result, VolatilityError};
pub use event_metrics::EventMetrics;
pub use event_movement_quality::EventMovementQuality;
pub use hourly_stats::{EventInHour, HourlyStats};
pub use stats_15min::Stats15Min;
pub use trading_recommendation::{RiskLevel, TradingRecommendation};
pub use volatility_duration::VolatilityDuration;
