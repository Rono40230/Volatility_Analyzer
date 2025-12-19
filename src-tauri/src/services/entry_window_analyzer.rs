// services/entry_window_analyzer.rs - Analyse de fenêtre d'entrée optimale
// Trouvez le meilleur timing pour entrer avant un événement

use crate::models::{Candle, EntryOffsetMetrics, EntryWindowAnalysisResult, Result, AssetProperties};
use chrono::{DateTime, Duration, Utc};
use tracing::info;

#[allow(dead_code)]
pub struct EntryWindowAnalyzer;

impl EntryWindowAnalyzer {
    /// Analyse la performance d'entrée à différents offsets avant un événement
    #[allow(dead_code)]
    pub fn analyze_entry_windows(
        candles: &[Candle],
        event_time: DateTime<Utc>,
        symbol: &str,
        event_type: &str,
        offsets: &[i32],
    ) -> Result<EntryWindowAnalysisResult> {
        info!(
            "🎯 Analyse fenêtre d'entrée pour {} / {}",
            symbol, event_type
        );

        let mut offset_metrics = Vec::new();

        for &offset_minutes in offsets {
            let entry_time = event_time - Duration::minutes(offset_minutes as i64);

            // Trouver la bougie au moment de l'entrée
            let entry_candle = candles.iter().find(|c| {
                c.datetime >= entry_time && c.datetime < entry_time + Duration::minutes(1)
            });

            if let Some(entry) = entry_candle {
                // Analyser les bougies suivantes (30min après événement)
                let analysis_window_start = event_time;
                let analysis_window_end = event_time + Duration::minutes(30);

                let window_candles: Vec<_> = candles
                    .iter()
                    .filter(|c| {
                        c.datetime >= analysis_window_start && c.datetime <= analysis_window_end
                    })
                    .collect();

                if !window_candles.is_empty() {
                    let metrics = Self::calculer_metriques_offset(
                        entry.close,
                        &window_candles,
                        offset_minutes,
                        symbol,
                    );
                    offset_metrics.push(metrics);
                }
            }
        }

        // Déterminer l'offset optimal
        let optimal = offset_metrics
            .iter()
            .max_by(|a, b| {
                a.win_rate
                    .partial_cmp(&b.win_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .unwrap_or(EntryOffsetMetrics {
                minutes_before_event: -5,
                sample_count: 0,
                winning_entries: 0,
                losing_entries: 0,
                win_rate: 0.0,
                avg_pips_gained: 0.0,
                avg_pips_lost: 0.0,
                max_pips_gained: 0.0,
                max_pips_lost: 0.0,
                profit_factor: 0.0,
            });

        let result = EntryWindowAnalysisResult {
            symbol: symbol.to_string(),
            event_type: event_type.to_string(),
            optimal_offset: optimal.minutes_before_event,
            optimal_win_rate: optimal.win_rate,
            offsets: offset_metrics,
            analysis_timestamp: Utc::now().timestamp(),
            total_events_analyzed: 1,
        };

        info!(
            "✅ Optimal entry: {}min avant (WR: {:.1}%)",
            optimal.minutes_before_event,
            optimal.win_rate * 100.0
        );

        Ok(result)
    }

    #[allow(dead_code)]
    fn calculer_metriques_offset(
        entry_price: f64,
        window_candles: &[&Candle],
        offset: i32,
        symbol: &str,
    ) -> EntryOffsetMetrics {
        let asset_props = AssetProperties::from_symbol(symbol);
        let mut winning = 0;
        let mut losing = 0;
        let mut total_pips_gained = 0.0;
        let mut total_pips_lost = 0.0;
        let mut max_gained: f64 = 0.0;
        let mut max_lost: f64 = 0.0;

        for candle in window_candles {
            // Calculer le mouvement en pips/points normalisés
            let pips_change = asset_props.normalize(candle.close - entry_price);

            if pips_change > 0.0 {
                winning += 1;
                total_pips_gained += pips_change;
                max_gained = max_gained.max(pips_change);
            } else {
                losing += 1;
                total_pips_lost += pips_change.abs();
                max_lost = max_lost.max(pips_change.abs());
            }
        }

        let sample_count = winning + losing;
        let win_rate = if sample_count > 0 {
            winning as f64 / sample_count as f64
        } else {
            0.0
        };

        let avg_pips_gained = if winning > 0 {
            total_pips_gained / winning as f64
        } else {
            0.0
        };

        let avg_pips_lost = if losing > 0 {
            total_pips_lost / losing as f64
        } else {
            0.0
        };

        let profit_factor = if total_pips_lost > 0.0 {
            total_pips_gained / total_pips_lost
        } else if total_pips_gained > 0.0 {
            999.0
        } else {
            0.0
        };

        EntryOffsetMetrics {
            minutes_before_event: offset,
            sample_count,
            winning_entries: winning,
            losing_entries: losing,
            win_rate,
            avg_pips_gained,
            avg_pips_lost,
            max_pips_gained: if max_gained > 0.0 { max_gained } else { 0.0 },
            max_pips_lost: if max_lost > 0.0 { max_lost } else { 0.0 },
            profit_factor,
        }
    }
}
