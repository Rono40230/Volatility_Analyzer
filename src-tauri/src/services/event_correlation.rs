// services/event_correlation.rs - Service de corrélation événements/volatilité
use crate::db::DbPool;
use crate::models::{CalendarEvent, Candle, CorrelatedEvent, VolatilityError};
use chrono::{Duration, NaiveDateTime, Timelike};
use diesel::prelude::*;

/// Service pour analyser la corrélation entre événements économiques et volatilité
pub struct EventCorrelationService {
    pool: DbPool,
}

impl EventCorrelationService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Récupère les événements économiques pour une période donnée
    pub fn get_events_for_period(
        &self,
        _symbol: &str,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        use crate::db::schema::calendar_events::dsl::*;

        let mut conn = self.pool.get()?;

        let events = calendar_events
            .filter(event_time.ge(start_time))
            .filter(event_time.le(end_time))
            .order(event_time.asc())
            .load::<CalendarEvent>(&mut conn)?;

        Ok(events)
    }

    /// Analyse la corrélation entre un événement et la volatilité observée
    /// NOTE: Méthode conservée pour usage futur (Phase 2 - analyses avancées)
    ///
    /// CORRECTION PHASE 2: Filtre maintenant les événements LOW-impact
    #[allow(dead_code)]
    pub fn correlate_event_with_volatility(
        &self,
        event: &CalendarEvent,
        candles: &[Candle],
    ) -> Option<CorrelatedEvent> {
        // ✅ CORRECTION: Ignorer les événements LOW-impact (bruit)
        if event.impact == "LOW" {
            return None;
        }

        // Cherche les bougies autour de l'événement (1h avant, pendant, 1h après)
        let event_hour = event.event_time.hour() as u8;

        // Calcule la volatilité (Max True Range) 1h avant l'événement (Baseline)
        // Fenêtre Baseline : -60 min à -10 min
        let before_metrics = self.calculer_metriques_volatilite(
            event.event_time - Duration::minutes(60),
            event.event_time - Duration::minutes(10),
            candles,
        );

        // Calcule la volatilité (Max True Range) autour de l'événement (Impact)
        // Fenêtre Impact : -10 min à +30 min (FIX-03)
        let impact_metrics = self.calculer_metriques_volatilite(
            event.event_time - Duration::minutes(10),
            event.event_time + Duration::minutes(30),
            candles,
        );

        // Si on a des données valides
        if let (Some((before_mean, _)), Some((_, impact_max))) = (before_metrics, impact_metrics) {
            if before_mean > 0.0 {
                // On compare le PIC de l'événement (Max TR) à la MOYENNE avant (Mean TR)
                // Cela donne le ratio d'explosivité
                let increase_pct = ((impact_max - before_mean) / before_mean) * 100.0;

                // Score de corrélation basé sur :
                // - Impact de l'événement (HIGH = bonus)
                // - Magnitude de l'augmentation
                // - Cohérence temporelle
                let impact_multiplier = if event.impact == "HIGH" { 1.5 } else { 1.0 };
                let correlation_score = (increase_pct.abs() * impact_multiplier).min(100.0);

                // Ne retourne que les corrélations significatives (> 50% d'augmentation pour un pic vs moyenne)
                if increase_pct > 50.0 {
                    return Some(CorrelatedEvent {
                        event: event.clone(),
                        volatility_hour: event_hour,
                        volatility_increase: increase_pct,
                        correlation_score,
                    });
                }
            }
        }

        None
    }

    /// Calcule les métriques de volatilité (Mean TR, Max TR) dans une fenêtre temporelle
    fn calculer_metriques_volatilite(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
        candles: &[Candle],
    ) -> Option<(f64, f64)> {
        let matching_candles: Vec<&Candle> = candles
            .iter()
            .filter(|c| {
                let naive_time = c.datetime.naive_utc();
                naive_time >= start && naive_time <= end
            })
            .collect();

        if matching_candles.is_empty() {
            return None;
        }

        // Calcul du True Range pour chaque bougie
        // TR = Max(H-L, |H-PC|, |L-PC|)
        // Pour simplifier ici (et car on a des slices), on utilise H-L qui est une bonne approx intraday
        // Sauf si on veut être très précis sur les gaps.
        // Utilisons H-L pour l'instant pour rester performant et simple sans MetricsCalculator complet
        let true_ranges: Vec<f64> = matching_candles
            .iter()
            .map(|c| c.high - c.low) // Approximation "Range Brut" demandée par le prompt
            .collect();

        let max_tr = true_ranges
            .iter()
            .fold(0.0f64, |acc, &x| if x > acc { x } else { acc });
            
        let mean_tr = true_ranges.iter().sum::<f64>() / true_ranges.len() as f64;

        Some((mean_tr, max_tr))
    }

    /// Analyse complète : trouve tous les événements corrélés avec des pics de volatilité
    /// NOTE: Méthode conservée pour usage futur (Phase 2 - analyses avancées)
    #[allow(dead_code)]
    pub fn analyze_correlations(
        &self,
        symbol: &str,
        candles: &[Candle],
    ) -> Result<Vec<CorrelatedEvent>, Box<dyn std::error::Error>> {
        if candles.is_empty() {
            return Ok(vec![]);
        }

        // Période couverte par les bougies
        let start_time = candles
            .first()
            .ok_or_else(|| VolatilityError::InsufficientData("Empty candles".to_string()))?
            .datetime
            .naive_utc();
        let end_time = candles
            .last()
            .ok_or_else(|| VolatilityError::InsufficientData("Empty candles".to_string()))?
            .datetime
            .naive_utc();

        // Récupère les événements économiques dans cette période
        let events = self.get_events_for_period(symbol, start_time, end_time)?;

        // Corrèle chaque événement avec la volatilité
        let mut correlations: Vec<CorrelatedEvent> = events
            .iter()
            .filter_map(|event| self.correlate_event_with_volatility(event, candles))
            .collect();

        // Trie par score de corrélation décroissant
        correlations.sort_by(|a, b| {
            b.correlation_score
                .partial_cmp(&a.correlation_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(correlations)
    }
}

/// Statistiques de corrélation globales
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorrelationStats {
    pub total_events: usize,
    pub high_impact_count: usize,
    pub medium_impact_count: usize,
    pub avg_volatility_increase_high: f64,
    pub avg_volatility_increase_medium: f64,
    pub top_correlations: Vec<CorrelatedEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_stats_calculation() {
        // Test basique de la structure
        let stats = CorrelationStats {
            total_events: 10,
            high_impact_count: 6,
            medium_impact_count: 4,
            avg_volatility_increase_high: 45.2,
            avg_volatility_increase_medium: 23.1,
            top_correlations: vec![],
        };

        assert_eq!(stats.total_events, 10);
        assert_eq!(stats.high_impact_count, 6);
    }
}
