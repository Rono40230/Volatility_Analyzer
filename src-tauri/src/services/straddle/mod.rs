// services/straddle/mod.rs - Unified Straddle API (adapter/re-exports)
// This module provides a single surface for straddle-related simulations
// and re-exports existing implementations to simplify imports and
// prepare for later consolidation/refactor.

mod implementation;

pub use implementation::*;

// This module now centralizes the canonical implementations under
// `services::straddle::...`. Legacy modules in `services/` delegate to
// these implementations to keep a stable migration path.
