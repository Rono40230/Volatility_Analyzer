// commands/correlation/mod.rs
// Point d'entrée pour les commandes de corrélation événements × paires
// Conforme .clinerules: < 100 lignes, exports seulement

mod event_impact;
mod heatmap_command;
mod heatmap_helpers;
mod pair_correlation;
mod pair_correlation_command;
mod pair_correlation_helpers;
mod pair_history;
mod past_events;
mod types;
mod volatility_helpers;

// Ré-exporter les commandes Tauri
pub use event_impact::get_event_impact_by_pair;
pub use heatmap_command::get_correlation_heatmap;
pub use pair_correlation_command::get_pair_event_correlation;
pub use pair_history::get_pair_event_history;
pub use past_events::get_past_events;
