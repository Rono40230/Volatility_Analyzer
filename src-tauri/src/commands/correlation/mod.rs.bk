// commands/correlation/mod.rs
// Point d'entrée pour les commandes de corrélation événements × paires
// Conforme .clinerules: < 100 lignes, exports seulement

mod data_availability;
mod heatmap_command;
mod heatmap_helpers;
mod pair_history;
mod past_events;
mod types;
mod utils;
mod volatility_helpers;

// Ré-exporter les commandes Tauri
pub use heatmap_command::get_correlation_heatmap;
pub use pair_history::get_pair_event_history;
pub use past_events::get_past_events;
