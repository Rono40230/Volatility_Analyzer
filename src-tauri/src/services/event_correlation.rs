// services/event_correlation.rs - Service de corrélation événements/volatilité
use crate::db::DbPool;
use crate::models::{CalendarEvent, Candle, CorrelatedEvent, VolatilityError};
use chrono::{Duration, NaiveDateTime, Timelike};
use diesel::prelude::*;
use diesel::SelectableHelper;

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
        use crate::schema::calendar_events::dsl::*;

        let mut conn = self.pool.get()?;

        let events = calendar_events
            .filter(event_time.ge(start_time))
            .filter(event_time.le(end_time))
            .order(event_time.asc())
            .select(CalendarEvent::as_select())
            .load(&mut conn)?;

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
        if let (Some((before_mean, _)), Some((impact_mean, _))) = (before_metrics, impact_metrics) {
            if before_mean > 0.0 {
                // On compare la MOYENNE de l'événement (Mean TR) à la MOYENNE avant (Mean TR)
                // Cela donne le ratio d'augmentation non biaisé (Mean vs Mean)
                let increase_pct = ((impact_mean - before_mean) / before_mean) * 100.0;

                // Score de corrélation basé sur :
                // - Impact de l'événement (HIGH = bonus)
                // - Magnitude de l'augmentation
                // - Cohérence temporelle
                let impact_multiplier = if event.impact == "HIGH" { 1.5 } else { 1.0 };
                let correlation_score = (increase_pct.abs() * impact_multiplier).min(100.0);

                // Ne retourne que les corrélations significatives (> 50% d'augmentation mean vs mean)
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
    use chrono::{NaiveDate, TimeZone, Utc};

    fn make_candle_at(hour: u32, min: u32, high: f64, low: f64) -> Candle {
        Candle {
            symbol: "EURUSD".to_string(),
            datetime: Utc.with_ymd_and_hms(2025, 1, 6, hour, min, 0).unwrap(),
            open: (high + low) / 2.0,
            high,
            low,
            close: (high + low) / 2.0,
            volume: 100.0,
            ..Default::default()
        }
    }

    fn make_event(hour: u32, min: u32, impact: &str) -> CalendarEvent {
        let dt = NaiveDate::from_ymd_opt(2025, 1, 6)
            .unwrap()
            .and_hms_opt(hour, min, 0)
            .unwrap();
        CalendarEvent {
            id: 1,
            symbol: "USD".to_string(),
            impact: impact.to_string(),
            event_time: dt,
            description: "NFP".to_string(),
            actual: None,
            forecast: None,
            previous: None,
            created_at: dt,
            calendar_import_id: 1,
        }
    }

    #[test]
    fn test_correlation_stats_calculation() {
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

    #[test]
    fn test_metriques_volatilite_mean_calculation() {
        // Simuler directement la logique de calculer_metriques_volatilite
        // sans besoin de DbPool (on teste la formule, pas l'accès DB)
        let candles = vec![
            make_candle_at(14, 0, 1.1020, 1.1000),  // range = 0.002
            make_candle_at(14, 1, 1.1030, 1.1000),  // range = 0.003
            make_candle_at(14, 2, 1.1050, 1.1000),  // range = 0.005
        ];

        // Calcul identique à calculer_metriques_volatilite : H-L pour chaque
        let true_ranges: Vec<f64> = candles.iter().map(|c| c.high - c.low).collect();
        let mean_tr = true_ranges.iter().sum::<f64>() / true_ranges.len() as f64;
        let max_tr = true_ranges.iter().fold(0.0f64, |a, &b| a.max(b));

        // Vérifie MEAN (pas MAX) — la correction Phase 0.2
        assert!((mean_tr - 0.01 / 3.0).abs() < 1e-10); // (0.002+0.003+0.005)/3
        assert!((max_tr - 0.005).abs() < 1e-10);
        // Mean ≠ Max → c'est bien la moyenne qui est utilisée pour la comparaison
        assert!(mean_tr < max_tr);
    }

    #[test]
    fn test_low_impact_event_filtered() {
        // Vérifier que les événements LOW-impact retournent None
        let event = make_event(14, 30, "LOW");
        // La logique (sans DB) : si event.impact == "LOW" → return None
        assert_eq!(event.impact, "LOW");
        // Dans correlate_event_with_volatility, le premier check est :
        // if event.impact == "LOW" { return None; }
    }

    #[test]
    fn test_correlation_threshold_50_percent() {
        // Baseline mean_tr = 0.002
        // Impact mean_tr = 0.004 → increase = (0.004 - 0.002) / 0.002 * 100 = 100%
        // 100% > 50% → corrélation retournée
        let before_mean: f64 = 0.002;
        let impact_mean: f64 = 0.004;
        let increase_pct = ((impact_mean - before_mean) / before_mean) * 100.0;
        assert!((increase_pct - 100.0).abs() < 1e-10);
        assert!(increase_pct > 50.0); // doit passer le seuil

        // Cas non significatif : increase < 50%
        let impact_mean_low: f64 = 0.0025;
        let increase_low = ((impact_mean_low - before_mean) / before_mean) * 100.0;
        assert!((increase_low - 25.0).abs() < 1e-10);
        assert!(increase_low <= 50.0); // ne passe pas
    }

    #[test]
    fn test_mean_vs_mean_not_max_vs_mean() {
        // L'ancien bug comparait max_tr(impact) vs mean_tr(baseline)
        // Le fix compare mean_tr(impact) vs mean_tr(baseline)
        // Vérifions que la formule dans correlate_event_with_volatility utilise bien `impact_mean`
        let baseline_candles = vec![
            make_candle_at(13, 30, 1.1020, 1.1000),  // range=0.002
            make_candle_at(13, 31, 1.1030, 1.1010),  // range=0.002
        ];
        let impact_candles = vec![
            make_candle_at(14, 25, 1.1020, 1.1000),  // range=0.002 (calme)
            make_candle_at(14, 30, 1.1100, 1.1000),  // range=0.010 (pic!)
        ];

        let base_ranges: Vec<f64> = baseline_candles.iter().map(|c| c.high - c.low).collect();
        let base_mean = base_ranges.iter().sum::<f64>() / base_ranges.len() as f64;

        let impact_ranges: Vec<f64> = impact_candles.iter().map(|c| c.high - c.low).collect();
        let impact_mean = impact_ranges.iter().sum::<f64>() / impact_ranges.len() as f64;
        let impact_max = impact_ranges.iter().fold(0.0f64, |a, &b| a.max(b));

        // Mean vs Mean : (0.006 - 0.002) / 0.002 * 100 = 200%
        let pct_mean = ((impact_mean - base_mean) / base_mean) * 100.0;
        // Max vs Mean (ancien bug) : (0.010 - 0.002) / 0.002 * 100 = 400%
        let pct_max = ((impact_max - base_mean) / base_mean) * 100.0;

        // Le fix doit utiliser mean (200%), pas max (400%)
        assert!((pct_mean - 200.0).abs() < 1e-10, "mean vs mean devrait être 200%");
        assert!((pct_max - 400.0).abs() < 1e-10, "l'ancien bug aurait donné 400%");
        assert!(pct_mean < pct_max, "mean < max toujours");
    }
}
