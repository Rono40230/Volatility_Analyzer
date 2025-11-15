// services/volatility/mod.rs - Module public pour l'analyse de volatilité
// Conforme .clinerules : < 300L, structure modulaire

mod analyzer;
mod correlation;
mod hourly_stats;
mod metrics;
mod utils;
mod stats_15min;

// Ré-exporte l'analyseur principal
pub use analyzer::VolatilityAnalyzer;
