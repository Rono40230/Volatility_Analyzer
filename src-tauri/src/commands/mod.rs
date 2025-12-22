// commands/mod.rs - Exports des commandes Tauri
pub mod archive_commands;
pub mod backtest;
pub mod calendar_commands;
pub mod calendar_db_helper;
pub mod calendar_import_commands;
pub mod calendar_parser;
pub mod candle_helpers;
pub mod candle_index_commands;
pub mod config_commands;
pub mod correlation;
pub mod csv_cleaner_commands;
pub mod deletion_commands;
pub mod economic_commands;
pub mod entry_window_analysis_commands;
pub mod event_metrics;
pub mod export_pdf_commands;
pub mod file_listing;
pub mod global_analysis_commands;
pub mod import_clean;
pub mod metadata;
pub mod movement_analysis_commands;
pub mod pair_data;
pub mod pair_importer;
pub mod planning;
pub mod retrospective_analysis;

pub mod session_commands;
pub mod volatility;
pub mod volatility_duration_commands;

pub use archive_commands::*;
pub use backtest::*;
pub use calendar_commands::get_upcoming_events;
pub use calendar_import_commands::*;
pub use candle_index_commands::{
    get_candle_index_stats, get_candles_for_hour, get_candles_for_quarter, get_pair_candles,
    init_candle_index, load_pair_candles,
};
pub use config_commands::*;
pub use correlation::*;
pub use csv_cleaner_commands::*;
pub use deletion_commands::*;
pub use economic_commands::{
    analyze_event_correlation, get_calendar_import_info, get_events_for_period,
    import_and_convert_calendar, load_economic_events_from_csv,
};
pub use entry_window_analysis_commands::analyze_entry_window;
pub use event_metrics::{
    calculer_metriques_evenement, clear_candles, get_available_symbols, load_candles_for_metrics,
};
pub use export_pdf_commands::exporter_formules_pdf;
pub use file_listing::*;
pub use global_analysis_commands::*;
pub use import_clean::import_and_clean_files;
pub use metadata::{
    get_calendar_id_by_filename, get_calendar_period_by_id, get_calendars_metadata,
    get_pair_metadata_from_db, get_pairs_metadata,
};
pub use movement_analysis_commands::{analyze_movement_quality, get_movement_qualities};
pub use pair_data::{import_pair_data, get_symbol_properties};
pub use planning::projection::project_stats_on_calendar;
// Phase 7: Retrospective analysis commands (fully integrated)
pub use retrospective_analysis::{analyze_decay_profile, analyze_peak_delay, get_event_types};
pub use session_commands::*;
pub use volatility::{
    analyze_quarter_entry_timing, analyze_slice_metrics, analyze_straddle_metrics, analyze_symbol,
    analyze_volatility_duration_for_slice, calculer_offset_optimal, calculer_frequence_whipsaw,
    calculer_taux_reussite, get_best_hours, get_cached_candles_for_hour, get_hourly_stats,
    get_quarter_events, load_candles_for_hour, load_symbols, ping,
};
pub use volatility_duration_commands::analyze_volatility_duration;
