// services/movement_analyzer.rs - Analyse qualité mouvements événementiels
// Conforme .clinerules: <300L, Result<T, ServiceError>, pas d'unwrap

use crate::models::{Candle, EventMovementQuality, VolatilityError};

// ============================================================================
// MODULE PRIVÉ: Calculs auxiliaires
// ============================================================================
mod calculations {
    use crate::models::{Candle, VolatilityError, AssetProperties};

    /// Calcule l'ATR (Average True Range) sur une série de candles
    pub fn calculer_atr(candles: &[Candle]) -> Result<f64, VolatilityError> {
        if candles.len() < 2 {
            return Err(VolatilityError::InsufficientData(
                "Besoin d'au moins 2 candles pour ATR".to_string(),
            ));
        }

        let mut true_ranges = Vec::new();

        for i in 1..candles.len() {
            let high = candles[i].high;
            let low = candles[i].low;
            let prev_close = candles[i - 1].close;

            let tr = (high - low)
                .max((high - prev_close).abs())
                .max((low - prev_close).abs());

            true_ranges.push(tr);
        }

        let atr = true_ranges.iter().sum::<f64>() / true_ranges.len() as f64;
        Ok(atr)
    }

    /// Analyse les mouvements post-événement
    pub fn analyser_mouvement_post_evenement(
        post_event_candles: &[Candle],
        pre_event_atr: f64,
        directional_threshold_atr_ratio: f64,
        reversal_window_minutes: i32,
        symbol: &str,
    ) -> Result<(f64, f64, f64), VolatilityError> {
        let asset_props = AssetProperties::from_symbol(symbol);
        let directional_threshold = pre_event_atr * directional_threshold_atr_ratio;

        let mut directional_count = 0;
        let mut whipsaw_count = 0;
        let mut total_pips_moved = 0.0;
        let mut analyzed_count = 0;

        let reversal_window_size = (reversal_window_minutes as usize).max(3);

        for i in 0..post_event_candles
            .len()
            .saturating_sub(reversal_window_size)
        {
            let window = &post_event_candles[i..i + reversal_window_size];

            if window.len() < 2 {
                continue;
            }

            let _initial_move = (window[window.len() - 1].close - window[0].close).abs();
            let max_high = window
                .iter()
                .map(|c| c.high)
                .fold(f64::NEG_INFINITY, f64::max);
            let min_low = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            let range = asset_props.normalize(max_high - min_low);

            total_pips_moved += range;
            analyzed_count += 1;

            if range > asset_props.normalize(directional_threshold) {
                directional_count += 1;

                if i + reversal_window_size < post_event_candles.len() {
                    let next_window_start = i + reversal_window_size;
                    let next_window_end =
                        (next_window_start + reversal_window_size).min(post_event_candles.len());

                    if next_window_end - next_window_start >= 2 {
                        let next_window = &post_event_candles[next_window_start..next_window_end];
                        let next_max_high = next_window
                            .iter()
                            .map(|c| c.high)
                            .fold(f64::NEG_INFINITY, f64::max);
                        let next_min_low = next_window
                            .iter()
                            .map(|c| c.low)
                            .fold(f64::INFINITY, f64::min);
                        let next_range = asset_props.normalize(next_max_high - next_min_low);

                        if next_range > asset_props.normalize(directional_threshold) {
                            whipsaw_count += 1;
                        }
                    }
                }
            }
        }

        if analyzed_count == 0 {
            return Ok((0.0, 0.0, 0.0));
        }

        let directional_move_rate = directional_count as f64 / analyzed_count as f64;
        let avg_pips_moved = total_pips_moved / analyzed_count as f64;
        let whipsaw_rate = if directional_count > 0 {
            whipsaw_count as f64 / directional_count as f64
        } else {
            0.0
        };

        Ok((
            directional_move_rate.min(1.0),
            avg_pips_moved,
            whipsaw_rate.min(1.0),
        ))
    }

    /// Calcule le score de qualité combiné (0-10)
    pub fn calculer_score_qualite(
        directional_move_rate: f64,
        success_rate: f64,
        whipsaw_rate: f64,
    ) -> f64 {
        let directional_score = directional_move_rate * 5.0;
        let stability_score = success_rate * 3.0;
        let whipsaw_penalty = whipsaw_rate * 2.0;

        let score = (directional_score + stability_score - whipsaw_penalty).clamp(0.0, 10.0);
        (score * 10.0).round() / 10.0
    }
}

use calculations::{analyser_mouvement_post_evenement, calculer_atr, calculer_score_qualite};

/// Analyseur de qualité des mouvements d'événements économiques
pub struct MovementAnalyzer;

/// Configuration pour l'analyse des mouvements
pub struct MovementAnalysisConfig {
    /// Seuil de mouvement directional en ratio ATR (ex: 0.75 = 75% du ATR)
    pub directional_threshold_atr_ratio: f64,
    /// Fenêtre de temps (en minutes) pour détecter reversals
    pub reversal_window_minutes: i32,
    /// Nombre minimum de candles requises pour une analyse valide
    pub min_required_candles: usize,
}

impl Default for MovementAnalysisConfig {
    fn default() -> Self {
        Self {
            directional_threshold_atr_ratio: 0.75,
            reversal_window_minutes: 15,
            min_required_candles: 30,
        }
    }
}

impl MovementAnalyzer {
    /// Calcule les métriques de qualité de mouvement pour un événement
    pub fn analyze_movement_quality(
        symbol: &str,
        event_type: &str,
        event_candles: &[Candle],
        config: &MovementAnalysisConfig,
    ) -> Result<EventMovementQuality, VolatilityError> {
        if event_candles.len() < config.min_required_candles {
            return Err(VolatilityError::InsufficientData(format!(
                "Besoin de {} candles, {} fournies",
                config.min_required_candles,
                event_candles.len()
            )));
        }

        let pre_event_atr =
            calculer_atr(&event_candles[..event_candles.len() / 2]).map_err(|_| {
                VolatilityError::MetricCalculationError(
                    "Impossible de calculer ATR pré-événement".to_string(),
                )
            })?;

        let post_event_start = event_candles.len() / 2;
        let post_event_candles = &event_candles[post_event_start..];

        // Calculer les métriques de mouvement
        let (directional_move_rate, avg_pips_moved, whipsaw_rate) = analyser_mouvement_post_evenement(
            post_event_candles,
            pre_event_atr,
            config.directional_threshold_atr_ratio,
            config.reversal_window_minutes,
            symbol,
        )?;

        let success_rate = 1.0 - whipsaw_rate;
        let quality_score =
            calculer_score_qualite(directional_move_rate, success_rate, whipsaw_rate);

        Ok(EventMovementQuality::new(
            symbol.to_string(),
            event_type.to_string(),
            directional_move_rate,
            whipsaw_rate,
            avg_pips_moved,
            success_rate,
            quality_score,
            event_candles.len() as i32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn create_test_candle(
        offset_minutes: i32,
        price: f64,
        high_delta: f64,
        low_delta: f64,
    ) -> Candle {
        Candle {
            id: None,
            symbol: "EURUSD".to_string(),
            datetime: DateTime::<Utc>::from_timestamp_millis(
                1609459200000 + (offset_minutes as i64 * 60000),
            )
            .expect("timestamp must be valid"),
            open: price,
            high: price + high_delta,
            low: price - low_delta,
            close: price,
            volume: 100.0,
        }
    }

    #[test]
    fn test_analyze_movement_quality_success() {
        let config = MovementAnalysisConfig {
            min_required_candles: 4,
            ..Default::default()
        };

        let mut candles = Vec::new();
        for i in 0..40 {
            if i < 20 {
                // Phase pré-événement: petit mouvement
                candles.push(create_test_candle(
                    i,
                    1.1000 + (i as f64 * 0.00001),
                    0.00005,
                    0.00005,
                ));
            } else {
                // Phase post-événement: grand mouvement directionnel
                candles.push(create_test_candle(
                    i,
                    1.1002 + ((i - 20) as f64 * 0.0001),
                    0.001,
                    0.0005,
                ));
            }
        }

        let result = MovementAnalyzer::analyze_movement_quality("EURUSD", "NFP", &candles, &config);
        assert!(result.is_ok());

        let quality = result.expect("should be ok");
        assert_eq!(quality.symbol, "EURUSD");
        assert_eq!(quality.event_type, "NFP");
        assert!(quality.directional_move_rate > 0.0);
        assert!(quality.quality_score >= 0.0 && quality.quality_score <= 10.0);
    }

    #[test]
    fn test_insufficient_candles() {
        let config = MovementAnalysisConfig::default();
        let candles = vec![
            create_test_candle(0, 1.1000, 0.0010, 0.0010),
            create_test_candle(1, 1.1001, 0.0010, 0.0010),
        ];

        let result = MovementAnalyzer::analyze_movement_quality("EURUSD", "NFP", &candles, &config);
        assert!(result.is_err());
    }
}
