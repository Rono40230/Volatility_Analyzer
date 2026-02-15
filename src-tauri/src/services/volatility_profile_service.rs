// services/volatility_profile_service.rs - Asset-class-specific volatility profiles
// FIX 2.3: Replace hardcoded 2.5 min with DB-driven asset_class duration
// Max line 300 (currently ~120 lines, well under limit)

use crate::db::DbPool;
use crate::models::volatility_profile::VolatilityProfile;
use crate::models::asset_class::AssetType;
use crate::schema::volatility_profiles;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use tracing::{error, warn};

#[derive(Clone)]
pub struct VolatilityProfileService {
    pool: DbPool,
}

impl VolatilityProfileService {
    pub fn new(pool: DbPool) -> Self {
        VolatilityProfileService { pool }
    }

    /// Retrieve volatility profile for a specific asset_type
    /// Returns profile from DB, or fallback defaults if not found
    pub fn get_profile(&self, asset_type: &str) -> Result<VolatilityProfile, String> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("❌ Pool error: {}", e);
            e.to_string()
        })?;

        let profile = volatility_profiles::table
            .filter(volatility_profiles::asset_type.eq(asset_type))
            .first::<VolatilityProfile>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Error querying volatility_profile for {}: {}", asset_type, e);
                e.to_string()
            })?;

        match profile {
            Some(p) => {
                tracing::debug!("✅ Got volatility profile for {}: half_life={} min", 
                    asset_type, p.half_life_minutes);
                Ok(p)
            }
            None => {
                warn!("⚠️  No volatility profile found for {}. Using defaults.", asset_type);
                Ok(self.default_profile(asset_type))
            }
        }
    }

    /// Get profile by AssetType enum (converts to string)
    pub fn get_profile_by_type(&self, asset_type: AssetType) -> Result<VolatilityProfile, String> {
        let type_str = match asset_type {
            AssetType::ForexMajor => "ForexMajor",
            AssetType::ForexJpy => "ForexJpy",
            AssetType::Gold => "Gold",
            AssetType::Silver => "Silver",
            AssetType::Crypto => "Crypto",
            AssetType::Index => "Index",
            AssetType::Commodity => "Commodity",
            AssetType::Unknown => "Unknown",
        };
        self.get_profile(type_str)
    }

    /// List all volatility profiles
    pub fn list_profiles(&self) -> Result<Vec<VolatilityProfile>, String> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("❌ Pool error: {}", e);
            e.to_string()
        })?;

        volatility_profiles::table
            .order(volatility_profiles::asset_type.asc())
            .load::<VolatilityProfile>(&mut conn)
            .map_err(|e| {
                error!("Error loading volatility profiles: {}", e);
                e.to_string()
            })
    }

    /// Update a volatility profile (by asset_type)
    pub fn update_profile(&self, asset_type: &str, half_life: f64, multiplier: f64) -> Result<VolatilityProfile, String> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("❌ Pool error: {}", e);
            e.to_string()
        })?;

        let now = chrono::Utc::now().naive_utc();
        
        diesel::update(volatility_profiles::table.filter(volatility_profiles::asset_type.eq(asset_type)))
            .set((
                volatility_profiles::half_life_minutes.eq(half_life),
                volatility_profiles::recommended_multiplier.eq(multiplier),
                volatility_profiles::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                error!("Error updating volatility profile for {}: {}", asset_type, e);
                e.to_string()
            })?;

        volatility_profiles::table
            .filter(volatility_profiles::asset_type.eq(asset_type))
            .first::<VolatilityProfile>(&mut conn)
            .map_err(|e| {
                error!("Error retrieving updated profile: {}", e);
                e.to_string()
            })
    }

    /// Default profiles when not found in DB (fallback values)
    fn default_profile(&self, asset_type: &str) -> VolatilityProfile {
        match asset_type {
            "ForexMajor" => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 1.8,
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            "ForexJpy" => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 2.0,
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            "Crypto" => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 5.0,
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            "Commodity" => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 3.5,
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            "Index" => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 3.0,
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            "Gold" => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 2.2,
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            "Silver" => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 2.2,
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            _ => VolatilityProfile {
                id: 0,
                asset_type: asset_type.to_string(),
                half_life_minutes: 2.5, // Ultimate fallback
                recommended_multiplier: 2.0,
                data_source: "hardcoded_default".to_string(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile_forex_major() {
        let service = VolatilityProfileService::new(
            crate::db::establish_connection_pool().expect("DB pool"),
        );
        let profile = service.default_profile("ForexMajor");
        assert_eq!(profile.half_life_minutes, 1.8);
        assert_eq!(profile.recommended_multiplier, 2.0);
    }

    #[test]
    fn test_default_profile_crypto() {
        let service = VolatilityProfileService::new(
            crate::db::establish_connection_pool().expect("DB pool"),
        );
        let profile = service.default_profile("Crypto");
        assert_eq!(profile.half_life_minutes, 5.0);
        assert_eq!(profile.recommended_multiplier, 2.0);
    }

    #[test]
    fn test_default_profile_forex_jpy() {
        let service = VolatilityProfileService::new(
            crate::db::establish_connection_pool().expect("DB pool"),
        );
        let profile = service.default_profile("ForexJpy");
        assert_eq!(profile.half_life_minutes, 2.0);
        assert_eq!(profile.recommended_multiplier, 2.0);
    }
}
