-- Migration: Ajouter colonnes pour Peak Delay et Decay Profile à stats_15min
-- Phase 7b: Enrichissement rétrospectif

ALTER TABLE stats_15min ADD COLUMN peak_delay_minutes INTEGER DEFAULT NULL;
-- peak_delay_minutes: délai en minutes entre événement et pic ATR
-- Valeurs possibles: -5 (avant) à +15 (après)

ALTER TABLE stats_15min ADD COLUMN decay_rate_pips_per_minute REAL DEFAULT NULL;
-- decay_rate: taux de décroissance ATR après pic (pips/minute)
-- Valeurs: 0.5 (très lent) à 5.0 (très rapide)

ALTER TABLE stats_15min ADD COLUMN decay_speed TEXT DEFAULT 'MEDIUM';
-- decay_speed: classification FAST | MEDIUM | SLOW
-- Basé sur decay_rate: FAST (>3.0), MEDIUM (1.5-3.0), SLOW (<1.5)
