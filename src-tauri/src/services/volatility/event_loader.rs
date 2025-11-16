// services/volatility/event_loader.rs - Chargement et association des événements économiques
// Module séparé pour respecter la limite de taille (analyzer.rs < 300L)

use crate::db::DbPool;
use crate::models::{EventInHour, HourlyStats, Result, Stats15Min, VolatilityError};
use chrono::Timelike;

/// Service de chargement des événements économiques
pub struct EventLoader;

impl EventLoader {
    /// Charge les événements économiques (HIGH/MEDIUM) et les associe aux heures
    pub fn load_and_associate_events(
        candles: &[crate::models::Candle],
        symbol: &str,
        hourly_stats: &mut [HourlyStats],
        pool: Option<DbPool>,
    ) -> Result<()> {
        // Si pas de pool, skip chargement des événements
        let Some(pool) = pool else {
            return Ok(());
        };

        // Charger les événements du calendrier pour la période analysée
        let start_time = candles.first().map(|c| c.datetime.naive_utc()).ok_or(
            VolatilityError::InsufficientData("No candles to determine event period".to_string()),
        )?;

        let end_time = candles.last().map(|c| c.datetime.naive_utc()).ok_or(
            VolatilityError::InsufficientData("No candles to determine event period".to_string()),
        )?;

        // Charger événements via EventCorrelationService
        let event_service = crate::services::EventCorrelationService::new(pool);
        let events = event_service
            .get_events_for_period(symbol, start_time, end_time)
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))?;

        // Filtrer HIGH/MEDIUM impact et compter par heure (Paris)
        // NOTE: Les candles sont en UTC, on les convertit en heure de Paris (UTC+1/+2 selon DST)
        // Paris: UTC+1 en hiver, UTC+2 en été
        // Pour simplifier, on utilise UTC+1 (heure d'hiver standard)
        const PARIS_OFFSET_HOURS: i32 = 1;

        for event in events {
            if event.impact != "HIGH" && event.impact != "MEDIUM" {
                continue;
            }

            // Convertir l'heure UTC en heure de Paris
            let utc_hour = event.event_time.hour() as i32;
            let paris_hour = (utc_hour + PARIS_OFFSET_HOURS) % 24;
            let paris_hour_u8 = paris_hour as u8;

            // Trouver l'heure correspondante dans hourly_stats
            if let Some(hour_stat) = hourly_stats.iter_mut().find(|h| h.hour == paris_hour_u8) {
                let event_in_hour = EventInHour {
                    event_name: event.description.clone(),
                    impact: event.impact.clone(),
                    datetime: event.event_time.format("%H:%M:%S").to_string(),
                    volatility_increase: 0.0,
                };
                hour_stat.events.push(event_in_hour);
            }
        }

        Ok(())
    }

    /// Associe les événements économiques aux tranches de 15 minutes
    pub fn load_and_associate_events_15min(
        candles: &[crate::models::Candle],
        symbol: &str,
        stats_15min: &mut [Stats15Min],
        pool: Option<DbPool>,
    ) -> Result<()> {
        // Si pas de pool, skip chargement des événements
        let Some(pool) = pool else {
            return Ok(());
        };

        // Charger les événements du calendrier pour la période analysée
        let start_time = candles.first().map(|c| c.datetime.naive_utc()).ok_or(
            VolatilityError::InsufficientData("No candles to determine event period".to_string()),
        )?;

        let end_time = candles.last().map(|c| c.datetime.naive_utc()).ok_or(
            VolatilityError::InsufficientData("No candles to determine event period".to_string()),
        )?;

        // Charger événements via EventCorrelationService
        let event_service = crate::services::EventCorrelationService::new(pool);
        let events = event_service
            .get_events_for_period(symbol, start_time, end_time)
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))?;

        // Filtrer HIGH/MEDIUM impact et compter par tranche de 15 minutes (Paris)
        const PARIS_OFFSET_HOURS: i32 = 1;

        for event in events {
            if event.impact != "HIGH" && event.impact != "MEDIUM" {
                continue;
            }

            // Convertir l'heure UTC en heure de Paris
            let utc_hour = event.event_time.hour() as i32;
            let utc_minute = event.event_time.minute() as i32;

            let paris_hour = (utc_hour + PARIS_OFFSET_HOURS) % 24;
            let paris_hour_u8 = paris_hour as u8;
            let quarter = (utc_minute / 15) as u8;

            // Trouver la tranche de 15 minutes correspondante
            if let Some(slot) = stats_15min
                .iter_mut()
                .find(|s| s.hour == paris_hour_u8 && s.quarter == quarter)
            {
                let event_in_hour = EventInHour {
                    event_name: event.description.clone(),
                    impact: event.impact.clone(),
                    datetime: event.event_time.format("%H:%M:%S").to_string(),
                    volatility_increase: 0.0,
                };
                slot.events.push(event_in_hour);
            }
        }

        Ok(())
    }
}
