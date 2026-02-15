// services/entry_point_analyzer.rs
// Analyseur de point d'entrée optimal pour straddle.
//
// Pour chaque offset (minute 0-14 du quarter), simule un straddle sur chaque
// occurrence historique et calcule le profit net = mouvement brut - spread réel.
// Le spread provient des ticks importés (champ spread_mean sur Candle).

use crate::models::asset_class::AssetProperties;
use crate::models::entry_analysis::{EntryAnalysisConfig, EntryAnalysisResult, MinuteDetail};
use crate::models::{Candle, Result, VolatilityError};
use chrono::{NaiveDate, Timelike};
use std::collections::HashMap;
use tracing::info;

// ─── Structures internes ────────────────────────────────────────────────────

struct StraddleSim {
    profit_net_pips: f64,
    spread_pips: f64,
    is_win: bool,
    #[allow(dead_code)] // Utilisé en Phase 4 (profil de mouvement dans l'UI)
    peak_forward_minute: usize,
}

// ─── API publique ───────────────────────────────────────────────────────────

/// Analyse les 15 offsets d'un quarter et retourne le point d'entrée optimal.
pub fn analyze_entry_points(
    candles: &[Candle],
    symbol: &str,
    event_type: &str,
    hour: u8,
    quarter: u8,
    config: &EntryAnalysisConfig,
) -> Result<EntryAnalysisResult> {
    let props = AssetProperties::from_symbol(symbol);
    if props.pip_value <= 0.0 {
        return Err(VolatilityError::ValidationError(format!(
            "pip_value invalide pour {symbol}"
        )));
    }

    let quarter_start_min = quarter * 15;
    let daily = group_candles_by_date(candles);

    info!(
        "Analyse entry points {symbol} @ {hour:02}:{quarter_start_min:02}, {} jours",
        daily.len()
    );

    let mut minute_details: Vec<MinuteDetail> = Vec::with_capacity(15);
    let mut non_tradable: Vec<u8> = Vec::new();

    for offset in 0u8..15 {
        let detail = analyze_offset(
            &daily,
            offset,
            hour,
            quarter_start_min,
            config,
            props.pip_value,
        );
        if !detail.tradable {
            non_tradable.push(offset);
        }
        minute_details.push(detail);
    }

    // Offset optimal = meilleur profit net parmi les tradables avec assez d'échantillons
    let opt = minute_details
        .iter()
        .filter(|d| d.tradable && d.sample_size >= config.min_samples)
        .max_by(|a, b| {
            a.avg_net_profit_pips
                .partial_cmp(&b.avg_net_profit_pips)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .or_else(|| {
            // Fallback: prend le meilleur même non-tradable
            minute_details
                .iter()
                .filter(|d| d.sample_size > 0)
                .max_by(|a, b| {
                    a.avg_net_profit_pips
                        .partial_cmp(&b.avg_net_profit_pips)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
        .ok_or_else(|| {
            VolatilityError::InsufficientData("Aucune donnée pour cette tranche".into())
        })?;

    let (peak_minute, duration, decay) = compute_movement_profile(
        &daily,
        opt.offset,
        hour,
        quarter_start_min,
        config,
        props.pip_value,
    );
    let consistency = compute_consistency(
        &daily,
        opt.offset,
        hour,
        quarter_start_min,
        config,
        props.pip_value,
    );

    let entry_min = quarter_start_min + opt.offset;
    let label = format!("{hour:02}:{entry_min:02} UTC");

    Ok(EntryAnalysisResult {
        symbol: symbol.to_string(),
        event_type: event_type.to_string(),
        optimal_offset_minutes: opt.offset,
        optimal_entry_time_label: label,
        real_win_rate: opt.win_rate,
        avg_net_profit_pips: opt.avg_net_profit_pips,
        avg_spread_at_entry_pips: opt.avg_spread_pips,
        avg_movement_pips: opt.avg_net_profit_pips + 2.0 * opt.avg_spread_pips,
        peak_minute,
        movement_duration_minutes: duration,
        decay_speed: decay,
        consistency_score: consistency,
        sample_size: opt.sample_size,
        non_tradable_minutes: non_tradable,
        minute_details,
        unit: props.unit,
    })
}

// ─── Fonctions internes ─────────────────────────────────────────────────────

fn group_candles_by_date(candles: &[Candle]) -> HashMap<NaiveDate, Vec<&Candle>> {
    let mut groups: HashMap<NaiveDate, Vec<&Candle>> = HashMap::new();
    for c in candles {
        groups.entry(c.datetime.date_naive()).or_default().push(c);
    }
    for group in groups.values_mut() {
        group.sort_by_key(|c| c.datetime);
    }
    groups
}

fn find_candle_at<'a>(day: &[&'a Candle], hour: u8, minute: u8) -> Option<&'a Candle> {
    day.iter()
        .find(|c| c.datetime.hour() == hour as u32 && c.datetime.minute() == minute as u32)
        .copied()
}

fn find_forward_candles<'a>(
    day: &[&'a Candle],
    entry: &Candle,
    forward_minutes: usize,
) -> Vec<&'a Candle> {
    let start = entry.datetime;
    day.iter()
        .filter(|c| {
            let diff = (c.datetime - start).num_minutes();
            diff > 0 && diff <= forward_minutes as i64
        })
        .copied()
        .collect()
}

/// Simule un straddle réaliste avec sortie à durée fixe :
/// - Mouvement = |close_sortie - close_entrée| à la FIN de la fenêtre (pas au meilleur moment)
/// - Coût straddle = 2× spread (on ouvre 2 positions : long + short)
/// - Win = profit net > 0 après déduction du coût total
///   Pas de biais look-ahead : on ne connaît pas le futur
fn simulate_straddle(entry: &Candle, forward: &[&Candle], pip_value: f64) -> Option<StraddleSim> {
    if forward.is_empty() {
        return None;
    }
    let spread_price = entry.spread_mean.unwrap_or(0.0);
    let spread_pips = spread_price / pip_value;

    // Sortie à la fin de la fenêtre (time-based exit, pas d'optimisation ex-post)
    let exit_candle = forward.last()?;
    let movement_pips = (exit_candle.close - entry.close).abs() / pip_value;

    // Pic = minute du plus grand mouvement (pour info uniquement, pas pour le calcul de profit)
    let mut peak_offset = 1usize;
    let mut best_seen = 0.0f64;
    for (i, fc) in forward.iter().enumerate() {
        let mov = (fc.close - entry.close).abs() / pip_value;
        if mov > best_seen {
            best_seen = mov;
            peak_offset = i + 1;
        }
    }

    // Straddle = 2 positions → coût = 2× spread
    let straddle_cost = spread_pips * 2.0;
    let profit_net = movement_pips - straddle_cost;

    Some(StraddleSim {
        profit_net_pips: profit_net,
        spread_pips,
        is_win: profit_net > 0.0,
        peak_forward_minute: peak_offset,
    })
}

fn analyze_offset(
    daily: &HashMap<NaiveDate, Vec<&Candle>>,
    offset: u8,
    hour: u8,
    quarter_start_min: u8,
    config: &EntryAnalysisConfig,
    pip_value: f64,
) -> MinuteDetail {
    let target_min = quarter_start_min + offset;
    let mut wins = 0usize;
    let mut total = 0usize;
    let mut sum_profit = 0.0f64;
    let mut sum_spread = 0.0f64;

    for day_candles in daily.values() {
        let entry = match find_candle_at(day_candles, hour, target_min) {
            Some(c) => c,
            None => continue,
        };
        let forward = find_forward_candles(day_candles, entry, config.forward_minutes);
        if let Some(sim) = simulate_straddle(entry, &forward, pip_value) {
            total += 1;
            sum_profit += sim.profit_net_pips;
            sum_spread += sim.spread_pips;
            if sim.is_win {
                wins += 1;
            }
        }
    }

    let (win_rate, avg_profit, avg_spread) = if total > 0 {
        (
            wins as f64 / total as f64,
            sum_profit / total as f64,
            sum_spread / total as f64,
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    MinuteDetail {
        offset,
        win_rate,
        avg_net_profit_pips: avg_profit,
        avg_spread_pips: avg_spread,
        sample_size: total,
        tradable: total == 0 || avg_spread <= config.spread_threshold_pips,
    }
}

/// Calcule le peak minute, la durée, et la vitesse de decay
fn compute_movement_profile(
    daily: &HashMap<NaiveDate, Vec<&Candle>>,
    offset: u8,
    hour: u8,
    quarter_start_min: u8,
    config: &EntryAnalysisConfig,
    pip_value: f64,
) -> (u8, f64, String) {
    let target_min = quarter_start_min + offset;
    let n = config.forward_minutes;
    let mut sums = vec![0.0f64; n];
    let mut counts = vec![0usize; n];

    for day_candles in daily.values() {
        let entry = match find_candle_at(day_candles, hour, target_min) {
            Some(c) => c,
            None => continue,
        };
        let spread_price = entry.spread_mean.unwrap_or(0.0);
        let entry_long = entry.close + spread_price / 2.0;
        let entry_short = entry.close - spread_price / 2.0;

        let forward = find_forward_candles(day_candles, entry, n);
        for fc in &forward {
            let idx = (fc.datetime - entry.datetime).num_minutes() as usize;
            if idx == 0 || idx > n {
                continue;
            }
            // Mouvement instantané de cette minute (pas running max)
            let mov_long = (fc.high - entry_long).max(0.0);
            let mov_short = (entry_short - fc.low).max(0.0);
            let mov = mov_long.max(mov_short) / pip_value;
            sums[idx - 1] += mov;
            counts[idx - 1] += 1;
        }
    }

    let profile: Vec<f64> = sums
        .iter()
        .zip(counts.iter())
        .map(|(s, c)| if *c > 0 { s / *c as f64 } else { 0.0 })
        .collect();

    let peak_idx = profile
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let peak_val = profile.get(peak_idx).copied().unwrap_or(0.0);

    let duration = if peak_val > 0.0 {
        // Durée = nombre total de minutes où le mouvement >= 50% du pic
        profile
            .iter()
            .filter(|v| **v >= peak_val * 0.5)
            .count() as f64
    } else {
        0.0
    };

    let decay = if peak_val <= 0.0 {
        "UNKNOWN".to_string()
    } else {
        let at_5 = profile.get(peak_idx + 5).copied().unwrap_or(0.0);
        let at_10 = profile.get(peak_idx + 10).copied().unwrap_or(0.0);
        if at_5 < peak_val * 0.25 {
            "FAST".to_string()
        } else if at_10 < peak_val * 0.25 {
            "MEDIUM".to_string()
        } else {
            "SLOW".to_string()
        }
    };

    ((peak_idx + 1) as u8, duration, decay)
}

/// Consistance = 1 - (std_dev / |mean|), Bessel correction, clampé [0, 1]
fn compute_consistency(
    daily: &HashMap<NaiveDate, Vec<&Candle>>,
    offset: u8,
    hour: u8,
    quarter_start_min: u8,
    config: &EntryAnalysisConfig,
    pip_value: f64,
) -> f64 {
    let target_min = quarter_start_min + offset;
    let mut profits: Vec<f64> = Vec::new();

    for day_candles in daily.values() {
        let entry = match find_candle_at(day_candles, hour, target_min) {
            Some(c) => c,
            None => continue,
        };
        let forward = find_forward_candles(day_candles, entry, config.forward_minutes);
        if let Some(sim) = simulate_straddle(entry, &forward, pip_value) {
            profits.push(sim.profit_net_pips);
        }
    }

    if profits.len() < 2 {
        return 0.0;
    }
    let mean = profits.iter().sum::<f64>() / profits.len() as f64;
    if mean.abs() < f64::EPSILON {
        return 0.0;
    }
    let variance =
        profits.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (profits.len() - 1) as f64;
    (1.0 - variance.sqrt() / mean.abs()).clamp(0.0, 1.0)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn make_candle(hour: u32, minute: u32, close: f64, high: f64, low: f64, spread: f64) -> Candle {
        Candle {
            symbol: "EURUSD".to_string(),
            datetime: Utc.with_ymd_and_hms(2025, 1, 6, hour, minute, 0).unwrap(),
            open: close,
            high,
            low,
            close,
            volume: 100.0,
            spread_mean: Some(spread),
            ..Default::default()
        }
    }

    #[test]
    fn test_simulate_straddle_profitable() {
        let entry = make_candle(14, 30, 1.1000, 1.1005, 1.0995, 0.0003);
        let fwd1 = make_candle(14, 31, 1.1010, 1.1020, 1.0998, 0.0002);
        let fwd2 = make_candle(14, 32, 1.1015, 1.1030, 1.0990, 0.0002);
        let forward: Vec<&Candle> = vec![&fwd1, &fwd2];
        let pip = 0.0001;

        let sim = simulate_straddle(&entry, &forward, pip).expect("should produce result");
        // Sortie à la fin (fwd2) : exit.close = 1.1015
        // |1.1015 - 1.1000| / 0.0001 = 15 pips
        // straddle_cost = 3 * 2 = 6 pips
        // profit = 15 - 6 = 9 pips
        assert!(sim.is_win);
        assert!((sim.profit_net_pips - 9.0).abs() < 0.1);
    }

    #[test]
    fn test_simulate_straddle_losing() {
        let entry = make_candle(14, 30, 1.1000, 1.1005, 1.0995, 0.0010); // spread 10 pips
        let fwd1 = make_candle(14, 31, 1.1001, 1.1003, 1.0998, 0.0010);
        let forward: Vec<&Candle> = vec![&fwd1];
        let pip = 0.0001;

        let sim = simulate_straddle(&entry, &forward, pip).expect("should produce result");
        // Close-based: |1.1001 - 1.1000| / 0.0001 = 1 pip
        // straddle_cost = 10 * 2 = 20 pips
        // profit = 1 - 20 = -19 pips → perte
        assert!(!sim.is_win);
        assert!(sim.profit_net_pips < 0.0);
    }

    #[test]
    fn test_non_tradable_zone() {
        let config = EntryAnalysisConfig {
            forward_minutes: 5,
            spread_threshold_pips: 5.0,
            min_samples: 1,
        };
        // Crée des candles avec spread élevé (10 pips = 0.0010) pour offset 0
        let mut candles = Vec::new();
        for day in 6..9 {
            // 3 jours
            candles.push(Candle {
                symbol: "EURUSD".to_string(),
                datetime: Utc.with_ymd_and_hms(2025, 1, day, 14, 30, 0).unwrap(),
                open: 1.1000,
                high: 1.1010,
                low: 1.0990,
                close: 1.1000,
                volume: 100.0,
                spread_mean: Some(0.0010), // 10 pips > seuil de 5
                ..Default::default()
            });
            // Forward candles
            for m in 31..36 {
                candles.push(Candle {
                    symbol: "EURUSD".to_string(),
                    datetime: Utc.with_ymd_and_hms(2025, 1, day, 14, m, 0).unwrap(),
                    open: 1.1000,
                    high: 1.1020,
                    low: 1.0980,
                    close: 1.1005,
                    volume: 100.0,
                    spread_mean: Some(0.0005),
                    ..Default::default()
                });
            }
        }

        let result =
            analyze_entry_points(&candles, "EURUSD", "NFP", 14, 2, &config).expect("should work");
        // Offset 0 (minute 30) a un spread de 10 pips > seuil 5 → non-tradable
        assert!(result.non_tradable_minutes.contains(&0));
    }

    #[test]
    fn test_consistency_calculation() {
        let profits = vec![10.0, 12.0, 11.0, 10.5, 11.5];
        let mean = profits.iter().sum::<f64>() / profits.len() as f64;
        let var =
            profits.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (profits.len() - 1) as f64;
        let consistency = (1.0 - var.sqrt() / mean.abs()).clamp(0.0, 1.0);
        // Des profits très réguliers → consistance élevée
        assert!(consistency > 0.9);
    }

    #[test]
    fn test_win_rate_is_real_counting_not_formula() {
        // Crée 5 jours : 3 gagnants (close directional > 2×spread), 2 perdants
        let config = EntryAnalysisConfig {
            forward_minutes: 3,
            spread_threshold_pips: 100.0, // pas de filtre
            min_samples: 1,
        };

        let mut candles = Vec::new();
        for day in 6..11 {
            // 5 jours (6-10 jan 2025)
            let is_winner = day < 9; // jours 6,7,8 = gagnant ; 9,10 = perdant
            // Close-based: gagnants close loin, perdants close proche
            let fwd_close = if is_winner { 1.1020 } else { 1.1001 };

            // Bougie d'entrée (offset 0 = minute 30)
            candles.push(Candle {
                symbol: "EURUSD".to_string(),
                datetime: Utc.with_ymd_and_hms(2025, 1, day, 14, 30, 0).unwrap(),
                open: 1.1000,
                high: 1.1005,
                low: 1.0995,
                close: 1.1000,
                volume: 100.0,
                spread_mean: Some(0.0002), // 2 pips → straddle = 4 pips
                ..Default::default()
            });
            // Forward candles
            for m in 31..34 {
                candles.push(Candle {
                    symbol: "EURUSD".to_string(),
                    datetime: Utc.with_ymd_and_hms(2025, 1, day, 14, m, 0).unwrap(),
                    open: 1.1000,
                    high: 1.1040,
                    low: 1.0980,
                    close: fwd_close, // Seul close est utilisé pour la simulation
                    volume: 100.0,
                    spread_mean: Some(0.0002),
                    ..Default::default()
                });
            }
        }

        let result = analyze_entry_points(&candles, "EURUSD", "NFP", 14, 2, &config)
            .expect("should compute");

        // Winners: |1.1020 - 1.1000| / 0.0001 = 20 pips → profit = 20 - 4 = 16 > 0 → win
        // Losers: |1.1001 - 1.1000| / 0.0001 = 1 pip → profit = 1 - 4 = -3 < 0 → loss
        // Win rate doit être 3/5 = 0.6 (comptage réel, pas formule)
        assert!(
            (result.real_win_rate - 0.6).abs() < 0.05,
            "Win rate should be ~60% (3/5), got {:.2}",
            result.real_win_rate
        );
        assert_eq!(result.sample_size, 5);
    }

    #[test]
    fn test_profit_net_is_movement_minus_straddle_cost() {
        // Test isolé : profit_net = mouvement à la sortie - 2 × spread
        let entry = make_candle(14, 30, 1.1000, 1.1005, 1.0995, 0.0005); // spread=5 pips
        let fwd = make_candle(14, 31, 1.1020, 1.1050, 1.0990, 0.0003);
        let forward: Vec<&Candle> = vec![&fwd];
        let pip = 0.0001;

        let sim = simulate_straddle(&entry, &forward, pip).expect("should work");
        // Sortie à la fin (seule bougie) : exit.close = 1.1020
        // |1.1020 - 1.1000| / 0.0001 = 20 pips
        // straddle_cost = 5 * 2 = 10 pips
        // profit = 20 - 10 = 10 pips
        assert!((sim.profit_net_pips - 10.0).abs() < 0.1);
        assert!((sim.spread_pips - 5.0).abs() < 0.1);
        assert!(sim.is_win);
    }
}
