mod calculator;
mod candles;

pub use calculator::calculer_metriques_evenement;
pub use candles::{clear_candles, get_available_symbols, load_candles_for_metrics};

use crate::models::Candle;
use std::sync::Mutex;

#[derive(Clone)]
pub struct CandlesState {
    pub candles: std::sync::Arc<Mutex<Vec<Candle>>>,
}

impl Default for CandlesState {
    fn default() -> Self {
        Self {
            candles: std::sync::Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candles_state_creation() {
        let state = CandlesState::default();
        let candles = state.candles.lock().expect("Failed to lock candles");
        assert_eq!(candles.len(), 0);
    }
}
