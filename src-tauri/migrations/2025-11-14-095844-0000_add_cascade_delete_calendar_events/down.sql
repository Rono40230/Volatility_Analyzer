-- Annuler la migration: Recréer la table sans la FK
PRAGMA foreign_keys = ON;

CREATE TABLE calendar_events_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    symbol TEXT NOT NULL,
    event_time TIMESTAMP NOT NULL,
    impact TEXT NOT NULL,
    description TEXT NOT NULL,
    actual REAL,
    forecast REAL,
    previous REAL,
    calendar_import_id INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO calendar_events_old 
  (id, symbol, event_time, impact, description, actual, forecast, previous, calendar_import_id, created_at)
SELECT 
  id, symbol, event_time, impact, description, actual, forecast, previous, calendar_import_id, created_at
FROM calendar_events;

DROP TABLE calendar_events;

ALTER TABLE calendar_events_old RENAME TO calendar_events;

CREATE INDEX idx_calendar_events_symbol ON calendar_events(symbol);
CREATE INDEX idx_calendar_events_time ON calendar_events(event_time);
