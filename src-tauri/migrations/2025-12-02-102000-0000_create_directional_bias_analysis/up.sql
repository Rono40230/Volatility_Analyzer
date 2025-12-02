-- Migration: Créer table directional_bias_analysis
-- Stocke l'asymétrie UP vs DOWN des événements
-- Phase 7b: Enrichissement rétrospectif

CREATE TABLE directional_bias_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    event_type TEXT NOT NULL,                    -- NFP, Jobless Claims, CPI, etc
    pair TEXT NOT NULL,                          -- EURUSD, GBPUSD, etc
    up_wins_count INTEGER NOT NULL,              -- nombre de trades UP gagnants
    down_wins_count INTEGER NOT NULL,            -- nombre de trades DOWN gagnants
    whipsaw_count INTEGER NOT NULL,              -- nombre de faux déclenchements
    up_bias REAL NOT NULL,                       -- -1.0 to +1.0 (asymétrie)
    asymmetry_percent REAL NOT NULL,             -- 0-100 (%) = |up_bias| * 100
    classification TEXT NOT NULL,                -- UP_BIASED, DOWN_BIASED, NEUTRAL
    sample_size INTEGER NOT NULL,                -- total trades
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(event_type, pair)
);

-- Indices pour recherches rapides
CREATE INDEX idx_directional_bias_event_pair ON directional_bias_analysis(event_type, pair);
CREATE INDEX idx_directional_bias_classification ON directional_bias_analysis(classification);
CREATE INDEX idx_directional_bias_created_at ON directional_bias_analysis(created_at);
