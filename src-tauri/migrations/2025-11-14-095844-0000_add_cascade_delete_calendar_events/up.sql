-- Migration: Ajouter contrainte ON DELETE CASCADE pour calendar_events
-- Problème: Avant cette migration, supprimer un calendar_import ne supprimait pas ses events en cascade
-- Solution: Recréer la table calendar_events avec une FK PRAGMA foreign_keys ON

-- Vérifier que foreign_keys est activé
PRAGMA foreign_keys = ON;

-- Créer une table temporaire avec la même structure + la contrainte FK
CREATE TABLE calendar_events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    symbol TEXT NOT NULL,
    event_time TIMESTAMP NOT NULL,
    impact TEXT NOT NULL,
    description TEXT NOT NULL,
    actual REAL,
    forecast REAL,
    previous REAL,
    calendar_import_id INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(calendar_import_id) REFERENCES calendar_imports(id) ON DELETE CASCADE
);

-- Copier les données existantes
INSERT INTO calendar_events_new 
  (id, symbol, event_time, impact, description, actual, forecast, previous, calendar_import_id, created_at)
SELECT 
  id, symbol, event_time, impact, description, actual, forecast, previous, calendar_import_id, created_at
FROM calendar_events;

-- Supprimer l'ancienne table
DROP TABLE calendar_events;

-- Renommer la nouvelle table
ALTER TABLE calendar_events_new RENAME TO calendar_events;

-- Recréer les indices
CREATE INDEX idx_calendar_events_symbol ON calendar_events(symbol);
CREATE INDEX idx_calendar_events_time ON calendar_events(event_time);
CREATE INDEX idx_calendar_events_import_id ON calendar_events(calendar_import_id);
