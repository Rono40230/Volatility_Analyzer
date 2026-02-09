// services/pair_data/mod.rs - Module de conversion de données de paires
// Conforme .clinerules : structure modulaire < 300L

mod converter;
mod datetime_parser;
mod formats;
mod metadata;
pub mod symbol_properties;
pub mod conversion_db;
mod types;

// Ré-exports publics
pub use converter::PairDataConverter;
pub use symbol_properties::*;
