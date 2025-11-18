CREATE TABLE volatility_durations (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  symbol VARCHAR(10) NOT NULL,
  hour INTEGER NOT NULL,
  quarter INTEGER NOT NULL,
  peak_duration_minutes INTEGER NOT NULL,
  volatility_half_life_minutes INTEGER NOT NULL,
  recommended_trade_expiration_minutes INTEGER NOT NULL,
  confidence_score INTEGER NOT NULL CHECK (confidence_score >= 50 AND confidence_score <= 100),
  sample_size INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(symbol, hour, quarter),
  CHECK (hour >= 0 AND hour <= 23),
  CHECK (quarter >= 0 AND quarter <= 3)
);

CREATE INDEX idx_volatility_durations_symbol ON volatility_durations(symbol);
CREATE INDEX idx_volatility_durations_hour ON volatility_durations(hour);
