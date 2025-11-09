// services/volatility/correlation.rs - Corrélation avec événements économiques
// Conforme .clinerules : < 120L, pas d'unwrap()

use crate::db::DbPool;
use crate::models::{Candle, CorrelatedEvent, HourlyStats, Result, VolatilityError};
use crate::services::CalendarScraper;
use chrono::Timelike;
use tracing::{info, warn};

/// Corrélateur d'événements économiques
/// NOTE: Cette structure est conservée pour usage futur (Phase 2 - corrélations avancées)
#[allow(dead_code)]
pub(super) struct EventCorrelator;

impl EventCorrelator {
    /// Corrèle les événements économiques avec les pics de volatilité
    /// NOTE: Cette fonction est conservée pour usage futur
    #[allow(dead_code)]
    pub(super) fn correlate(
        candles: &[Candle],
        symbol: &str,
        hourly_stats: &[HourlyStats],
        pool: DbPool,
    ) -> Result<Vec<CorrelatedEvent>> {
        // Détermine la plage de dates à analyser
        if candles.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = candles.first()
            .ok_or_else(|| VolatilityError::InsufficientData("Empty candles".to_string()))?
            .datetime.naive_utc();
        let end_time = candles.last()
            .ok_or_else(|| VolatilityError::InsufficientData("Empty candles".to_string()))?
            .datetime.naive_utc();

        // Récupère les événements économiques pour cette période
        let scraper = CalendarScraper::new(pool);
        let events = match scraper.get_historical_events(symbol, start_time, end_time) {
            Ok(events) => events,
            Err(e) => {
                warn!("Failed to fetch economic events: {}", e);
                return Ok(Vec::new());
            }
        };

        if events.is_empty() {
            info!("No economic events found for {} in this period", symbol);
            return Ok(Vec::new());
        }

        // Calcule la volatilité moyenne globale
        let mean_volatility: f64 = hourly_stats
            .iter()
            .map(|s| s.volatility_mean)
            .sum::<f64>()
            / hourly_stats.len() as f64;

        // Corrèle chaque événement avec les heures de haute volatilité
        let mut correlated = Vec::new();

        for event in events {
            let event_hour = event.event_time.hour() as u8;

            // Trouve les stats de cette heure
            if let Some(stats) = hourly_stats.iter().find(|s| s.hour == event_hour) {
                // Vérifie si la volatilité est significativement plus élevée
                if stats.volatility_mean > mean_volatility * 1.2 {
                    let volatility_increase =
                        ((stats.volatility_mean / mean_volatility) - 1.0) * 100.0;

                    // Score de corrélation basé sur l'impact et l'augmentation
                    let impact_score = match event.impact.to_uppercase().as_str() {
                        "HIGH" => 100.0,
                        "MEDIUM" => 60.0,
                        _ => 30.0,
                    };

                    let increase_factor = (volatility_increase / 50.0).min(1.0);
                    let correlation_score = impact_score * increase_factor;

                    correlated.push(CorrelatedEvent {
                        event,
                        volatility_hour: event_hour,
                        volatility_increase,
                        correlation_score,
                    });
                }
            }
        }

        // Trie par score de corrélation décroissant
        correlated.sort_by(|a, b| {
            b.correlation_score
                .partial_cmp(&a.correlation_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        info!(
            "Found {} correlated events for {}",
            correlated.len(),
            symbol
        );

        Ok(correlated)
    }
}
