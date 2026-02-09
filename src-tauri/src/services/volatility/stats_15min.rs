// services/volatility/stats_15min.rs - Calcul des statistiques pour 15 minutes (scalping)
// Conforme .clinerules : < 150L, pas d'unwrap()

use super::utils::{max, mean};
use crate::models::{AssetProperties, Candle, Result, Stats15Min};
use crate::services::{MetricsCalculator, StraddleParameterService, VolatilityDurationAnalyzer};
use chrono::Timelike;
use std::collections::HashMap;
use tracing::debug;
use std::cmp::Ordering;

/// Calculateur de statistiques pour tranches de 15 minutes
pub(super) struct Stats15MinCalculator<'a> {
    candles: &'a [Candle],
}

impl<'a> Stats15MinCalculator<'a> {
    pub(super) fn new(candles: &'a [Candle]) -> Self {
        Self { candles }
    }

    /// Calcule les statistiques pour chaque tranche de 15 minutes (en UTC)
    /// Les candles sont en UTC, on les groupe par 15min UTC
    pub(super) fn calculer(&self) -> Result<Vec<Stats15Min>> {
        debug!("Calculating 15-minute statistics (UTC)");

        // Groupe les bougies par tranche de 15 minutes UTC
        let mut groups_15min: HashMap<(u8, u8), Vec<&Candle>> = HashMap::new();

        for candle in self.candles {
            let utc_hour = candle.hour_utc() as u8;
            let utc_minute = candle.datetime.minute() as i32;

            let quarter = (utc_minute / 15) as u8;

            groups_15min
                .entry((utc_hour, quarter))
                .or_default()
                .push(candle);
        }

        // Calcule les stats pour chaque tranche de 15 minutes
        let mut stats = Vec::new();

        for hour in 0..24 {
            for quarter in 0..4 {
                if let Some(candles) = groups_15min.get(&(hour, quarter)) {
                    let hour_stats = self.calculer_pour_tranche(hour, quarter, candles)?;
                    stats.push(hour_stats);
                } else {
                    // Tranche sans données : stats vides
                    stats.push(Stats15Min {
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

        debug!(
            "Calculated stats for {} 15-minute slices (UTC)",
            stats.len()
        );
        Ok(stats)
    }

    /// Calcule les statistiques pour une tranche de 15 minutes spécifique
    fn calculer_pour_tranche(
        &self,
        hour: u8,
        quarter: u8,
        candles: &[&Candle],
    ) -> Result<Stats15Min> {
        let candle_count = candles.len();

        if candle_count == 0 {
            return Ok(Stats15Min {
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

        // Crée un vecteur owned pour le calculateur
        let owned_candles: Vec<Candle> = candles.iter().map(|&c| c.clone()).collect();
        let calc = MetricsCalculator::new(&owned_candles);

        // Calcule les métriques (avec gestion d'erreur si pas assez de données)
        let atr_values = calc.calculer_atr(14).unwrap_or_default();
        let volatility_values = calc.calculer_volatilite(20).unwrap_or_default();
        let body_ranges = calc.calculer_ranges_corps();
        let shadow_ratios = calc.calculer_ratios_ombres();
        let tick_qualities = calc.calculer_qualite_tick();
        let noise_ratios = calc.calculer_ratio_bruit();
        let tr_dist = calc.calculer_distribution_true_range()?;

        // Normalisation des valeurs (Pips/Points) — DB override en priorité
        let symbol = candles
            .first()
            .map(|c| c.symbol.as_str())
            .unwrap_or("EURUSD");
        let asset_props = crate::services::pair_data::symbol_properties::get_asset_properties(symbol);

        // Calcule les moyennes
        let raw_atr_mean = mean(&atr_values); // FIX-01: Moyenne au lieu de last()
        let raw_atr_max = max(&atr_values);
        let volatility_mean = mean(&volatility_values);
        // TÂCHE 3: Utiliser True Range au lieu de simple H-L
        let raw_range_mean = mean(&tr_dist.true_ranges);
        let raw_max_true_range = tr_dist.percentile_95; // FIX-01: Max Spike stabilisé (95e percentile)
        
        let atr_mean = asset_props.normalize(raw_atr_mean);
        let atr_max = asset_props.normalize(raw_atr_max);
        let range_mean = asset_props.normalize(raw_range_mean);
        let max_true_range = asset_props.normalize(raw_max_true_range);

        let body_range_mean = mean(&body_ranges);
        let shadow_ratio_mean = mean(&shadow_ratios);
        let _tick_quality_mean = mean(&tick_qualities);
        let noise_ratio_mean = mean(&noise_ratios);
        let p95_wick = calculer_p95_wick(&owned_candles, &asset_props);

        // Calculate breakout percentage first
        let breakout_count = tr_dist.is_breakout.iter().filter(|&&b| b).count();
        let breakout_percentage =
            (breakout_count as f64 / tr_dist.is_breakout.len() as f64) * 100.0;

        // Direction Strength: Force directionnelle = (|directionalite| * cassures) / 10000
        // Note: Both values are percentages (0-100), so divide by 10000 to get result in 0-100 range
        let direction_strength = (body_range_mean.abs() * breakout_percentage) / 10000.0;

        // TÂCHE 4: Analyse réelle de décroissance de volatilité
        let (peak_duration, half_life, trade_exp) =
            match VolatilityDurationAnalyzer::analyser_depuis_bougies(hour, quarter, candles) {
                Ok(vd) => {
                    debug!(
                        "✅ TÂCHE 4 OK: {}:{} peak={} half_life={} trade_exp={}",
                        hour,
                        quarter,
                        vd.peak_duration_minutes,
                        vd.volatility_half_life_minutes,
                        vd.recommended_trade_expiration_minutes
                    );
                    (
                        Some(vd.peak_duration_minutes),
                        Some(vd.volatility_half_life_minutes),
                        Some(vd.recommended_trade_expiration_minutes),
                    )
                }
                Err(e) => {
                    debug!("⚠️ TÂCHE 4 ERREUR: {}:{} - {}", hour, quarter, e);
                    (None, None, None)
                }
            };

        // Calcul des paramètres Straddle (Harmonisation Straddle simultané V2)
        let straddle_params = StraddleParameterService::calculate_parameters(
            atr_mean,
            noise_ratio_mean,
            asset_props.pip_value,
            symbol,
            half_life,
            Some(p95_wick),
            Some(hour as u32),
        );

        // Calcul du profil de volatilité minute par minute (0-14) pour le graphique
        let mut minute_ranges: Vec<Vec<f64>> = vec![Vec::new(); 15];
        for candle in candles {
            let minute_idx = (candle.datetime.minute() % 15) as usize;
            if minute_idx < 15 {
                let raw_range = candle.high - candle.low;
                let normalized_range = asset_props.normalize(raw_range);
                minute_ranges[minute_idx].push(normalized_range);
            }
        }

        let volatility_profile: Vec<f64> = minute_ranges
            .iter()
            .map(|ranges| {
                if ranges.is_empty() {
                    0.0
                } else {
                    ranges.iter().sum::<f64>() / ranges.len() as f64
                }
            })
            .collect();

        // Détermination de la minute optimale (début de l'accélération ou pic)
        // On cherche le pic de volatilité moyenne
        let optimal_entry_minute = volatility_profile
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index as u8);

        Ok(Stats15Min {
            hour,
            quarter,
            candle_count,
            atr_mean,
            atr_max,
            max_true_range,
            volatility_mean,
            range_mean,
            body_range_mean,
            p95_wick,
            shadow_ratio_mean,
            volume_imbalance_mean: direction_strength,
            noise_ratio_mean,
            breakout_percentage,
            events: Vec::new(),
            peak_duration_minutes: peak_duration,
            volatility_half_life_minutes: half_life,
            recommended_trade_expiration_minutes: trade_exp,
            peak_duration_mean: peak_duration,
            volatility_half_life_mean: half_life,
            recommended_trade_expiration_mean: trade_exp,
            straddle_parameters: Some(straddle_params),
            volatility_profile: Some(volatility_profile),
            optimal_entry_minute,
        })
    }
}

fn calculer_p95_wick(candles: &[Candle], asset_props: &AssetProperties) -> f64 {
    let mut wicks: Vec<f64> = Vec::new();

    for candle in candles {
        let upper = candle.high - candle.close.max(candle.open);
        let lower = candle.open.min(candle.close) - candle.low;
        if upper > 0.0 {
            wicks.push(upper);
        }
        if lower > 0.0 {
            wicks.push(lower);
        }
    }

    if wicks.is_empty() {
        return 0.0;
    }

    wicks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let index = ((wicks.len() as f64) * 0.95).ceil() as usize;
    let safe_index = index.min(wicks.len().saturating_sub(1));
    let raw_p95 = wicks.get(safe_index).copied().unwrap_or(0.0);

    asset_props.normalize(raw_p95)
}
