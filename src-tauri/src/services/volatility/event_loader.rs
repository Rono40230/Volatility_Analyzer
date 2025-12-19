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

        tracing::info!(
            "EventLoader: Found {} events for period {} to {}",
            events.len(),
            start_time,
            end_time
        );

        // Debug: afficher les heures disponibles dans stats
        let stats_hours: Vec<u8> = hourly_stats.iter().map(|s| s.hour).collect();
        tracing::info!("   Stats available hours (UTC): {:?}", stats_hours);

        let mut associated_count = 0;
        for (i, event) in events.iter().enumerate() {
            // On utilise l'heure UTC pour matcher les stats UTC
            let event_hour = event.event_time.hour() as u8;

            if i < 3 {
                // Logger les 3 premiers pour debug
                tracing::info!(
                    "   Event sample: '{}' at {:?} (UTC Hour: {})",
                    event.description,
                    event.event_time,
                    event_hour
                );
            }

            if let Some(stat) = hourly_stats.iter_mut().find(|s| s.hour == event_hour) {
                let impact_upper = event.impact.to_uppercase();
                let normalized_impact = match impact_upper.as_str() {
                    "H" | "HIGH" => "HIGH",
                    "M" | "MEDIUM" => "MEDIUM",
                    "L" | "LOW" => "LOW",
                    _ => "NONE",
                };

                let event_in_hour = EventInHour {
                    event_name: event.description.clone(),
                    impact: normalized_impact.to_string(),
                    datetime: event.event_time.format("%H:%M:%S").to_string(),
                    volatility_increase: 0.0,
                };
                stat.events.push(event_in_hour);
                associated_count += 1;
            } else if i < 3 {
                tracing::warn!("⚠️ Could not find stat for UTC hour {}", event_hour);
            }
        }

        tracing::info!(
            "✅ [EventLoader] Associated {} events to hourly stats",
            associated_count
        );

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
            tracing::warn!("⚠️ No database pool provided for 15min events");
            return Ok(());
        };

        // Charger les événements du calendrier pour la période analysée
        let start_time = candles.first().map(|c| c.datetime.naive_utc()).ok_or(
            VolatilityError::InsufficientData("No candles to determine event period".to_string()),
        )?;

        let end_time = candles.last().map(|c| c.datetime.naive_utc()).ok_or(
            VolatilityError::InsufficientData("No candles to determine event period".to_string()),
        )?;

        tracing::info!(
            "🔍 EventLoader 15min starting for {} from {} to {}",
            symbol,
            start_time,
            end_time
        );

        // Log des slices disponibles
        let slice_keys: Vec<String> = stats_15min
            .iter()
            .map(|s| format!("{}:{}", s.hour, s.quarter))
            .collect();
        tracing::info!(
            "📦 Available 15min slices: {} - First few: {:?}",
            stats_15min.len(),
            &slice_keys[..std::cmp::min(10, slice_keys.len())]
        );

        // Charger événements via EventCorrelationService
        let event_service = crate::services::EventCorrelationService::new(pool);
        let events = event_service
            .get_events_for_period(symbol, start_time, end_time)
            .map_err(|e| VolatilityError::DatabaseError(e.to_string()))?;

        tracing::info!(
            "🔍 EventLoader 15min: Loaded {} total events for {}",
            events.len(),
            symbol
        );

        // Filtrer HIGH/MEDIUM impact et compter par tranche de 15 minutes (UTC)
        tracing::info!("🔍 Checking {} events against {} stats slots", events.len(), stats_15min.len());
        
        let mut matched_count = 0;
        let mut unmatched_count = 0;
        let mut skipped_impact_count = 0;

        for (i, event) in events.iter().enumerate() {
            if i < 3 {
                 tracing::debug!("Event sample: {} - Impact: {}", event.description, event.impact);
            }

            let impact_upper = event.impact.to_uppercase();
            let is_high = impact_upper == "HIGH" || impact_upper == "H";
            let is_medium = impact_upper == "MEDIUM" || impact_upper == "M";

            if !is_high && !is_medium {
                skipped_impact_count += 1;
                continue;
            }

            let normalized_impact = if is_high { "HIGH" } else { "MEDIUM" };

            // Utiliser UTC directement
            let utc_hour = event.event_time.hour() as u8;
            let utc_minute = event.event_time.minute() as u8;
            let quarter = utc_minute / 15;

            // Trouver la tranche de 15 minutes correspondante
            if let Some(slot) = stats_15min
                .iter_mut()
                .find(|s| s.hour == utc_hour && s.quarter == quarter)
            {
                let event_in_hour = EventInHour {
                    event_name: event.description.clone(),
                    impact: normalized_impact.to_string(),
                    datetime: event.event_time.format("%H:%M:%S").to_string(),
                    volatility_increase: 0.0,
                };
                slot.events.push(event_in_hour);
                matched_count += 1;
                if matched_count <= 5 {
                    tracing::debug!(
                        "✅ Event matched: {} at {}:{} (quarter {})",
                        event.description,
                        utc_hour,
                        utc_minute,
                        quarter
                    );
                }
            } else {
                unmatched_count += 1;
                if unmatched_count <= 5 {
                    tracing::debug!(
                        "❌ Event NOT matched: {} at UTC {}:{} (quarter {})",
                        event.description,
                        utc_hour,
                        utc_minute,
                        quarter
                    );
                }
            }
        }
        
        tracing::info!("📊 Event Matching Summary: Matched={}, Unmatched={}, Skipped(Low Impact)={}", matched_count, unmatched_count, skipped_impact_count);

        tracing::info!(
            "📊 EventLoader 15min result: {} matched, {} unmatched",
            matched_count,
            unmatched_count
        );

        // Log final: afficher les slices avec events
        let slices_with_events: Vec<_> = stats_15min
            .iter()
            .filter(|s| !s.events.is_empty())
            .map(|s| format!("{}:{} ({})", s.hour, s.quarter, s.events.len()))
            .collect();
        if !slices_with_events.is_empty() {
            tracing::info!("✅ Slices with events: {:?}", slices_with_events);
        } else {
            tracing::warn!("⚠️ NO slices have events after association!");
        }

        Ok(())
    }
}
