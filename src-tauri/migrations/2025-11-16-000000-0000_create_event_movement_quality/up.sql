-- Migration UP: Création table event_movement_quality
-- Stocke les métriques de qualité des mouvements par événement et paire

CREATE TABLE IF NOT EXISTS event_movement_quality (
    -- ID auto-incrémenté
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Clé composite: paire + type événement
    symbol TEXT NOT NULL,
    event_type TEXT NOT NULL,
    
    -- Métriques de qualité
    -- Proportion d'événements générant mouvement directionnel > ATR × 0.75
    directional_move_rate REAL NOT NULL,
    
    -- Proportion d'événements avec reversal dans les 15 minutes
    whipsaw_rate REAL NOT NULL,
    
    -- Mouvement moyen en pips
    avg_pips_moved REAL NOT NULL,
    
    -- Proportion d'événements avec succès (mouvement sans reversal rapide)
    success_rate REAL NOT NULL,
    
    -- Score combiné (0-10)
    quality_score REAL NOT NULL,
    
    -- Nombre d'occurrences analysées
    sample_size INTEGER NOT NULL,
    
    -- Timestamps
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    
    -- Contrainte: combinaison unique par paire+événement
    UNIQUE(symbol, event_type)
);

-- Index pour requêtes fréquentes
CREATE INDEX IF NOT EXISTS idx_movement_quality_score 
    ON event_movement_quality(quality_score DESC);

CREATE INDEX IF NOT EXISTS idx_movement_quality_pair 
    ON event_movement_quality(symbol);

CREATE INDEX IF NOT EXISTS idx_movement_quality_event_type 
    ON event_movement_quality(event_type);

CREATE INDEX IF NOT EXISTS idx_movement_quality_composite 
    ON event_movement_quality(symbol, event_type);

CREATE INDEX IF NOT EXISTS idx_movement_quality_success_rate 
    ON event_movement_quality(success_rate DESC);

CREATE INDEX IF NOT EXISTS idx_movement_quality_whipsaw_rate 
    ON event_movement_quality(whipsaw_rate);
