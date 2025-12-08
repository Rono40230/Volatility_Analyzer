// services/volatility/mod.rs - Module public pour l'analyse de volatilité
// Conforme .clinerules : < 300L, structure modulaire

mod analyzer;
mod best_quarter_finder;
mod confidence_scorer;
mod correlation;
mod event_loader;
mod hourly_stats;
mod metrics;
mod offset_calculator;
mod quarterly_aggregator;
mod stats_15min;
mod utils;
mod volatility_duration_analyzer;
mod volatility_heuristics;
mod whipsaw_detector;
mod whipsaw_simulator;
mod win_rate_calculator;

// Ré-exporte l'analyseur principal
pub use analyzer::VolatilityAnalyzer;
pub use offset_calculator::calculate_optimal_offset;
pub use volatility_duration_analyzer::VolatilityDurationAnalyzer;
pub use whipsaw_detector::calculate_whipsaw_frequency;
pub use win_rate_calculator::simulate_straddle_win_rate;
