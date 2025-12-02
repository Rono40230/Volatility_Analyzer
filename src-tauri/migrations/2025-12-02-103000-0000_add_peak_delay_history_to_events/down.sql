-- Rollback: Retirer colonne peak_delay_json

ALTER TABLE calendar_events DROP COLUMN peak_delay_json;
