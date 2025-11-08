-- Migration UP: Création table event_metrics
-- Stocke les métriques calculées par événement pour optimisation EA Straddle

CREATE TABLE IF NOT EXISTS event_metrics (
    -- Clé composite: type événement + paire
    event_type TEXT NOT NULL,
    pair_symbol TEXT NOT NULL,
    
    -- Métriques temporelles
    avg_duration_minutes REAL NOT NULL,
    peak_time_minutes REAL NOT NULL,
    return_to_normal_minutes REAL NOT NULL,
    
    -- Métriques performance
    win_rate REAL NOT NULL,
    avg_movement_pips REAL NOT NULL,
    max_movement_pips REAL NOT NULL,
    whipsaw_rate REAL NOT NULL,
    
    -- Timing optimal
    best_entry_minutes_before INTEGER NOT NULL,
    
    -- ATR contextualisé
    contextual_atr_before REAL NOT NULL,
    contextual_atr_after REAL NOT NULL,
    atr_increase_ratio REAL NOT NULL,
    
    -- Recommandations
    recommended_sl_multiplier REAL NOT NULL,
    recommended_tp_multiplier REAL NOT NULL,
    tradability_score REAL NOT NULL,
    recommendation TEXT NOT NULL,
    
    -- Métadonnées
    sample_size INTEGER NOT NULL,
    last_calculated TEXT NOT NULL,
    
    -- Clé primaire composite
    PRIMARY KEY (event_type, pair_symbol)
);

-- Index pour requêtes fréquentes
CREATE INDEX IF NOT EXISTS idx_event_metrics_tradability 
    ON event_metrics(tradability_score DESC);

CREATE INDEX IF NOT EXISTS idx_event_metrics_pair 
    ON event_metrics(pair_symbol);

CREATE INDEX IF NOT EXISTS idx_event_metrics_recommendation 
    ON event_metrics(recommendation);
