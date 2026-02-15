-- Create volatility_profiles table for asset-class-specific half-lives
-- FIX 2.3: Replace hardcoded 2.5 min with asset_class-specific duration values
CREATE TABLE volatility_profiles (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  asset_type VARCHAR(20) NOT NULL,
  half_life_minutes REAL NOT NULL,
  recommended_multiplier REAL NOT NULL DEFAULT 2.0,
  data_source VARCHAR(50) NOT NULL DEFAULT 'manual',
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(asset_type)
);

-- Insert default volatility profiles by asset class
-- Based on empirical forex/crypto volatility decay analysis
INSERT INTO volatility_profiles (asset_type, half_life_minutes, recommended_multiplier, data_source, updated_at) VALUES
  ('ForexMajor', 1.8, 2.0, 'manual', CURRENT_TIMESTAMP),
  ('ForexJpy', 2.0, 2.0, 'manual', CURRENT_TIMESTAMP),
  ('Gold', 2.2, 2.0, 'manual', CURRENT_TIMESTAMP),
  ('Silver', 2.2, 2.0, 'manual', CURRENT_TIMESTAMP),
  ('Crypto', 5.0, 2.0, 'manual', CURRENT_TIMESTAMP),
  ('Index', 3.0, 2.0, 'manual', CURRENT_TIMESTAMP),
  ('Commodity', 3.5, 2.0, 'manual', CURRENT_TIMESTAMP);

CREATE INDEX idx_volatility_profiles_asset_type ON volatility_profiles(asset_type);
