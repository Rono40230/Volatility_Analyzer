-- Rollback: Retirer colonnes Phase 7b

ALTER TABLE stats_15min DROP COLUMN peak_delay_minutes;
ALTER TABLE stats_15min DROP COLUMN decay_rate_pips_per_minute;
ALTER TABLE stats_15min DROP COLUMN decay_speed;
