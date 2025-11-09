// commands/correlation/mod.rs
// Point d'entrée pour les commandes de corrélation événements × paires
// Conforme .clinerules: < 100 lignes, exports seulement

mod types;
mod past_events;
mod event_impact;
mod pair_history;
mod heatmap;
mod volatility_helpers;

// Ré-exporter les commandes Tauri
pub use past_events::get_past_events;
pub use event_impact::get_event_impact_by_pair;
pub use pair_history::get_pair_event_history;
pub use heatmap::get_correlation_heatmap;

