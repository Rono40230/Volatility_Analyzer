// services/straddle/mod.rs - Unified Straddle API (adapter/re-exports)
// This module provides a single surface for straddle-related simulations
// and re-exports existing implementations to simplify imports and
// prepare for later consolidation/refactor.

pub use crate::services::straddle_simulator::simulate_straddle;
pub use crate::services::straddle_simulator_helpers::{
    simulate_trade_outcome, calculate_global_p95_wick, calculate_dynamic_offset,
    calculer_atr_moyen, calculate_ema, get_asset_cost, TradeOutcome,
};
pub use crate::services::volatility::whipsaw_simulator::{simulate_straddle_trade, TradeResult};

// Also expose risk helper for convenience
pub use crate::services::straddle_simulator_helpers::calculate_risk_level;

// NOTE: This file intentionally avoids duplicating logic. It is an
// adapter that centralizes exported symbols so call-sites can gradually
// migrate to `crate::services::straddle::...` without changing underlying
// implementations. Future work: merge implementations into this module
// and remove the legacy files.
