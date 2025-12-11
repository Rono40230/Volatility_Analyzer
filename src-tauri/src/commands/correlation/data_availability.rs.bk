// data_availability.rs
// Utilitaires pour vérifier la disponibilité des données de candles
// pour les événements calendrier

use crate::services::candle_index::CandleIndex;
use chrono::{Duration, NaiveDateTime, TimeZone, Utc};

/// Vérifie si des candles existent dans la fenêtre temporelle d'un événement
/// Fenêtre: [event_time - 30min, event_time + 30min]
///
/// # Arguments
/// * `candle_index` - Index des candles chargés en mémoire
/// * `pair_symbol` - Symbole de la paire (ex: "ADAUSD")
/// * `event_dt` - NaiveDateTime de l'événement
///
/// # Retour
/// `true` si au moins une candle existe dans la fenêtre, `false` sinon
pub fn has_candles_for_event(
    candle_index: &CandleIndex,
    pair_symbol: &str,
    event_dt: NaiveDateTime,
) -> bool {
    // Élargir la fenêtre pour inclure les candles qui ont COMMENCÉ avant l'événement
    // mais qui le couvrent (ex: H1). On regarde 2h avant et 1h après.
    // Cela permet de capturer une candle H1 de 14:00 pour un événement à 14:59.
    let event_window_start = event_dt - Duration::hours(2);
    let event_window_end = event_dt + Duration::hours(1);

    // Chercher les candles dans la fenêtre
    let candles = candle_index.get_candles_in_range(
        pair_symbol,
        event_window_start.date(),
        event_window_end.date(),
    );

    match candles {
        Some(candle_list) => {
            // Convertir les NaiveDateTime en DateTime<Utc> pour comparaison
            let window_start_utc = Utc.from_utc_datetime(&event_window_start);
            let window_end_utc = Utc.from_utc_datetime(&event_window_end);

            // Vérifier qu'au moins une candle est dans la fenêtre élargie
            candle_list.iter().any(|(candle_dt, _, _)| {
                *candle_dt >= window_start_utc && *candle_dt <= window_end_utc
            })
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_has_candles_for_event_with_data() {
        // Test avec données fictives minimales
        let index = CandleIndex::new();
        let event_dt =
            match NaiveDateTime::parse_from_str("2024-01-01 14:30:00", "%Y-%m-%d %H:%M:%S") {
                Ok(dt) => dt,
                Err(_) => panic!("Invalid test datetime"),
            };

        // Index vide → pas de candles
        let result = has_candles_for_event(&index, "ADAUSD", event_dt);
        assert!(!result);
    }
}
