-- Migration DOWN: Suppression table event_metrics

DROP INDEX IF EXISTS idx_event_metrics_recommendation;
DROP INDEX IF EXISTS idx_event_metrics_pair;
DROP INDEX IF EXISTS idx_event_metrics_tradability;
DROP TABLE IF EXISTS event_metrics;
