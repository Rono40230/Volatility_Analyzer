-- Migration: Créer table entry_timing_analysis
-- Stocke la profitabilité stratifiée par offset d'entrée (T-10, T-5, T-0, T+3)
-- Phase 7b: Enrichissement rétrospectif

CREATE TABLE entry_timing_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    event_type TEXT NOT NULL,                    -- NFP, Jobless Claims, CPI, etc
    pair TEXT NOT NULL,                          -- EURUSD, GBPUSD, etc
    entry_offset_minutes INTEGER NOT NULL,       -- -10, -5, 0, 3
    win_rate REAL NOT NULL,                      -- 0-100 (%)
    whipsaw_rate REAL NOT NULL,                  -- 0-100 (%)
    avg_profit_pips REAL NOT NULL,               -- pips (peut être négatif)
    sample_size INTEGER NOT NULL,                -- nombre de trades analysés
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(event_type, pair, entry_offset_minutes)
);

-- Indices pour recherches rapides
CREATE INDEX idx_entry_timing_event_pair ON entry_timing_analysis(event_type, pair);
CREATE INDEX idx_entry_timing_offset ON entry_timing_analysis(entry_offset_minutes);
CREATE INDEX idx_entry_timing_created_at ON entry_timing_analysis(created_at);
