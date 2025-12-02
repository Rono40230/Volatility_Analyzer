-- Migration: Ajouter colonne peak_delay_json à calendar_events
-- Stocke l'historique des Peak Delays agrégés par type d'événement
-- Format: {"NFP": [2.3, 1.8, 2.5], "Jobless": [4.1, 3.9]}
-- Phase 7b: Enrichissement rétrospectif (OPTIONNEL)

ALTER TABLE calendar_events ADD COLUMN peak_delay_json TEXT DEFAULT '{}';
-- JSON agrégé par type d'événement pour analyse cross-event
