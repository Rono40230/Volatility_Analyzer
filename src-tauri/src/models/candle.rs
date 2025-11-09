// models/candle.rs - Structure représentant une bougie OHLCV
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Candle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub symbol: String,
    pub datetime: DateTime<Utc>,
    #[validate(range(min = 0.0))]
    pub open: f64,
    #[validate(range(min = 0.0))]
    pub high: f64,
    #[validate(range(min = 0.0))]
    pub low: f64,
    #[validate(range(min = 0.0))]
    pub close: f64,
    #[validate(range(min = 0.0))]
    pub volume: f64,
}

impl Candle {
    pub fn new(symbol: String, datetime: DateTime<Utc>, open: f64, high: f64, low: f64, close: f64, volume: f64) 
        -> Result<Self, validator::ValidationErrors> {
        let candle = Self { id: None, symbol, datetime, open, high, low, close, volume };
        candle.validate()?;
        Ok(candle)
    }
    
    pub fn true_range(&self, prev_close: Option<f64>) -> f64 {
        let high_low = self.high - self.low;
        match prev_close {
            Some(pc) => {
                let high_close = (self.high - pc).abs();
                let low_close = (self.low - pc).abs();
                high_low.max(high_close).max(low_close)
            }
            None => high_low,
        }
    }
    
    pub fn body_range(&self) -> f64 {
        let range = self.high - self.low;
        if range == 0.0 { return 0.0; }
        ((self.close - self.open) / range) * 100.0
    }
    
    pub fn shadow_ratio(&self) -> f64 {
        let upper_wick = self.high - self.open.max(self.close);
        let lower_wick = self.open.min(self.close) - self.low;
        if lower_wick == 0.0 { return 1.0; }
        upper_wick / lower_wick
    }
    
    pub fn hour_utc(&self) -> u32 {
        self.datetime.hour()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_creation() {
        let candle = Candle::new(
            "EURUSD".to_string(),
            Utc::now(),
            1.0950,
            1.0980,
            1.0940,
            1.0970,
            1200.0,
        );
        assert!(candle.is_ok());
    }

    #[test]
    fn test_body_range() {
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0950,
            close: 1.0970,
            high: 1.0980,
            low: 1.0940,
            volume: 1200.0,
        };
        
        let body_range = candle.body_range();
        // (0.0020 / 0.0040) * 100 = 50.0%
        assert!((body_range - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_true_range() {
        let candle = Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: Utc::now(),
            open: 1.0950,
            close: 1.0970,
            high: 1.0980,
            low: 1.0940,
            volume: 1200.0,
        };
        
        let tr = candle.true_range(Some(1.0960));
        assert!(tr > 0.0);
    }

    #[test]
    fn test_shadow_ratio() {
        let candle = Candle { id: None, symbol: "EURUSD".to_string(), datetime: Utc::now(), open: 1.0950, close: 1.0970, high: 1.0980, low: 1.0940, volume: 1200.0 };
        let ratio = candle.shadow_ratio();
        assert!(ratio >= 0.0);
    }

    #[test]
    fn test_invalid_candle_high_low() {
        let result = Candle::new(Utc::now(), 1.0950, 1.0940, 1.0980, 1.0970, 1200.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_hour_utc() {
        let candle = Candle { id: None, symbol: "EURUSD".to_string(), datetime: Utc::now(), open: 1.0, high: 1.1, low: 0.9, close: 1.05, volume: 1000.0 };
        assert!(candle.hour_utc() < 24);
    }
}
