// services/volatility/quarterly_aggregator.rs - Agrégation des stats par quarter sur toute la période
// Calcule les moyennes historiques de peak_duration, half_life et trade_exp

use crate::models::Stats15Min;
use crate::services::StraddleParameterService;

/// Agrégateur de statistiques par quarter
pub(super) struct QuarterlyAggregator;

impl QuarterlyAggregator {
    /// Calcule les moyennes historiques pour chaque quarter (96 = 24h × 4 quarters)
    /// Prend tous les stats_15min (toute la période) et retourne les moyennes par quarter
    pub(super) fn aggregate(stats_15min: &[Stats15Min], pip_value: f64, symbol: &str) -> Vec<Stats15Min> {
        // Groupe les stats par (hour, quarter)
        let mut quarterly_groups: std::collections::HashMap<(u8, u8), Vec<&Stats15Min>> =
            std::collections::HashMap::new();

        for stat in stats_15min {
            if stat.candle_count > 0 {
                quarterly_groups
                    .entry((stat.hour, stat.quarter))
                    .or_default()
                    .push(stat);
            }
        }

        // Calcule les moyennes pour chaque quarter
        let mut averaged_stats = Vec::new();

        for hour in 0..24 {
            for quarter in 0..4 {
                if let Some(instances) = quarterly_groups.get(&(hour, quarter)) {
                    if instances.is_empty() {
                        // Quarter vide : créer un stat vide
                        averaged_stats.push(Stats15Min {
                            hour,
                            quarter,
                            candle_count: 0,
                            atr_mean: 0.0,
                            atr_max: 0.0,
                            max_true_range: 0.0,
                            volatility_mean: 0.0,
                            range_mean: 0.0,
                            body_range_mean: 0.0,
                            p95_wick: 0.0,
                            shadow_ratio_mean: 0.0,
                            volume_imbalance_mean: 0.0,
                            noise_ratio_mean: 0.0,
                            breakout_percentage: 0.0,
                            events: Vec::new(),
                            peak_duration_minutes: None,
                            volatility_half_life_minutes: None,
                            recommended_trade_expiration_minutes: None,
                            peak_duration_mean: None,
                            volatility_half_life_mean: None,
                            recommended_trade_expiration_mean: None,
                            straddle_parameters: None,
                            volatility_profile: None,
                            optimal_entry_minute: None,
                        });
                    } else {
                        // Calculer les moyennes des métriques ordinaires
                        let count = instances.len() as f64;
                        let atr_mean_avg =
                            instances.iter().map(|s| s.atr_mean).sum::<f64>() / count;
                        let atr_max_avg = instances.iter().map(|s| s.atr_max).sum::<f64>() / count;
                        let max_true_range_avg = instances.iter().map(|s| s.max_true_range).sum::<f64>() / count;
                        let volatility_mean_avg =
                            instances.iter().map(|s| s.volatility_mean).sum::<f64>() / count;
                        let range_mean_avg =
                            instances.iter().map(|s| s.range_mean).sum::<f64>() / count;
                        let body_range_mean_avg =
                            instances.iter().map(|s| s.body_range_mean).sum::<f64>() / count;
                        let p95_wick_avg =
                            instances.iter().map(|s| s.p95_wick).sum::<f64>() / count;
                        let shadow_ratio_mean_avg =
                            instances.iter().map(|s| s.shadow_ratio_mean).sum::<f64>() / count;
                        let volume_imbalance_mean_avg = instances
                            .iter()
                            .map(|s| s.volume_imbalance_mean)
                            .sum::<f64>()
                            / count;
                        let noise_ratio_mean_avg =
                            instances.iter().map(|s| s.noise_ratio_mean).sum::<f64>() / count;
                        let breakout_percentage_avg =
                            instances.iter().map(|s| s.breakout_percentage).sum::<f64>() / count;

                        // Calculer les moyennes des peak/half-life/trade_exp
                        let peak_duration_with_values: Vec<u16> = instances
                            .iter()
                            .filter_map(|s| s.peak_duration_minutes)
                            .collect();
                        let peak_duration_mean = if !peak_duration_with_values.is_empty() {
                            let avg = peak_duration_with_values
                                .iter()
                                .map(|v| *v as f64)
                                .sum::<f64>()
                                / peak_duration_with_values.len() as f64;
                            Some(avg as u16)
                        } else {
                            None
                        };

                        let half_life_with_values: Vec<u16> = instances
                            .iter()
                            .filter_map(|s| s.volatility_half_life_minutes)
                            .collect();
                        let volatility_half_life_mean = if !half_life_with_values.is_empty() {
                            let avg = half_life_with_values.iter().map(|v| *v as f64).sum::<f64>()
                                / half_life_with_values.len() as f64;
                            Some(avg as u16)
                        } else {
                            None
                        };

                        let trade_exp_with_values: Vec<u16> = instances
                            .iter()
                            .filter_map(|s| s.recommended_trade_expiration_minutes)
                            .collect();
                        let recommended_trade_expiration_mean = if !trade_exp_with_values.is_empty()
                        {
                            let avg = trade_exp_with_values.iter().map(|v| *v as f64).sum::<f64>()
                                / trade_exp_with_values.len() as f64;
                            Some(avg as u16)
                        } else {
                            None
                        };

                        // Compter les candles totaux sur la période
                        let total_candle_count: usize =
                            instances.iter().map(|s| s.candle_count).sum();

                        // Calcul des paramètres Straddle moyens (Harmonisation Straddle simultané V2)
                        let straddle_params = StraddleParameterService::calculate_parameters(
                            atr_mean_avg,
                            noise_ratio_mean_avg,
                            pip_value,
                            symbol,
                            volatility_half_life_mean,
                            Some(p95_wick_avg),
                        );

                        // Agrégation du profil de volatilité (moyenne minute par minute)
                        let mut aggregated_profile = vec![0.0; 15];
                        let mut profile_count = 0;

                        for stat in instances {
                            if let Some(profile) = &stat.volatility_profile {
                                if profile.len() == 15 {
                                    for (i, val) in profile.iter().enumerate() {
                                        aggregated_profile[i] += val;
                                    }
                                    profile_count += 1;
                                }
                            }
                        }

                        let final_profile = if profile_count > 0 {
                            aggregated_profile
                                .iter_mut()
                                .for_each(|v| *v /= profile_count as f64);
                            Some(aggregated_profile)
                        } else {
                            None
                        };

                        // Recalcul de la minute optimale d'entrée basée sur le profil moyen
                        let optimal_entry_minute = final_profile.as_ref().and_then(|profile| {
                            profile
                                .iter()
                                .enumerate()
                                .max_by(|(_, a), (_, b)| {
                                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                                })
                                .map(|(index, _)| index as u8)
                        });

                        averaged_stats.push(Stats15Min {
                            hour,
                            quarter,
                            candle_count: total_candle_count,
                            atr_mean: atr_mean_avg,
                            atr_max: atr_max_avg,
                            max_true_range: max_true_range_avg,
                            volatility_mean: volatility_mean_avg,
                            range_mean: range_mean_avg,
                            body_range_mean: body_range_mean_avg,
                            p95_wick: p95_wick_avg,
                            shadow_ratio_mean: shadow_ratio_mean_avg,
                            volume_imbalance_mean: volume_imbalance_mean_avg,
                            noise_ratio_mean: noise_ratio_mean_avg,
                            breakout_percentage: breakout_percentage_avg,
                            events: Vec::new(),
                            peak_duration_minutes: None, // Remplacé par _mean
                            volatility_half_life_minutes: None, // Remplacé par _mean
                            recommended_trade_expiration_minutes: None, // Remplacé par _mean
                            peak_duration_mean,
                            volatility_half_life_mean,
                            recommended_trade_expiration_mean,
                            straddle_parameters: Some(straddle_params),
                            volatility_profile: final_profile,
                            optimal_entry_minute,
                        });
                    }
                } else {
                        // Quarter sans aucune donnée sur la période
                    averaged_stats.push(Stats15Min {
                        hour,
                        quarter,
                        candle_count: 0,
                        atr_mean: 0.0,
                        atr_max: 0.0,
                        max_true_range: 0.0,
                        volatility_mean: 0.0,
                        range_mean: 0.0,
                        body_range_mean: 0.0,
                        p95_wick: 0.0,
                        shadow_ratio_mean: 0.0,
                        volume_imbalance_mean: 0.0,
                        noise_ratio_mean: 0.0,
                        breakout_percentage: 0.0,
                        events: Vec::new(),
                        peak_duration_minutes: None,
                        volatility_half_life_minutes: None,
                        recommended_trade_expiration_minutes: None,
                        peak_duration_mean: None,
                        volatility_half_life_mean: None,
                        recommended_trade_expiration_mean: None,
                        straddle_parameters: None,
                        volatility_profile: None,
                        optimal_entry_minute: None,
                    });
                }
            }
        }

        averaged_stats
    }
}
