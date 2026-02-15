// models/volatility_profile.rs - Asset-class-specific volatility decay profiles
// Respect .clinerules: Queryable vs Insertable structures

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::volatility_profiles;

/// Volatility profile for an asset class (SELECT query result)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = volatility_profiles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct VolatilityProfile {
    pub id: i32,
    pub asset_type: String,
    pub half_life_minutes: f64,
    pub recommended_multiplier: f64,
    pub data_source: String,
    pub updated_at: NaiveDateTime,
}

/// Structure for inserting a new volatility profile (INSERT)
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = volatility_profiles)]
pub struct NewVolatilityProfile {
    pub asset_type: String,
    pub half_life_minutes: f64,
    pub recommended_multiplier: f64,
    pub data_source: String,
}

impl VolatilityProfile {
    /// Calculate recommended trade expiration based on half-life
    pub fn calculate_recommended_duration(&self) -> i64 {
        (self.half_life_minutes * self.recommended_multiplier).ceil() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommended_duration_calculation() {
        let profile = VolatilityProfile {
            id: 1,
            asset_type: "ForexMajor".to_string(),
            half_life_minutes: 1.8,
            recommended_multiplier: 2.0,
            data_source: "manual".to_string(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        assert_eq!(profile.calculate_recommended_duration(), 4); // ceil(1.8 * 2.0) = 4
    }

    #[test]
    fn test_forex_jpy_profile() {
        let profile = VolatilityProfile {
            id: 2,
            asset_type: "ForexJpy".to_string(),
            half_life_minutes: 2.0,
            recommended_multiplier: 2.0,
            data_source: "manual".to_string(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        assert_eq!(profile.calculate_recommended_duration(), 4); // 2.0 * 2.0 = 4
    }

    #[test]
    fn test_crypto_profile() {
        let profile = VolatilityProfile {
            id: 5,
            asset_type: "Crypto".to_string(),
            half_life_minutes: 5.0,
            recommended_multiplier: 2.0,
            data_source: "manual".to_string(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        assert_eq!(profile.calculate_recommended_duration(), 10); // 5.0 * 2.0 = 10
    }
}
