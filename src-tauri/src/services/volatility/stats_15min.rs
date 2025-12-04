// services/volatility/stats_15min.rs - Calcul des statistiques pour 15 minutes (scalping)
// Conforme .clinerules : < 150L, pas d'unwrap()

use super::utils::{max, mean};
use crate::models::{Candle, Result, Stats15Min};
use crate::services::{MetricsCalculator, VolatilityDurationAnalyzer};
use chrono::Timelike;
use std::collections::HashMap;
use tracing::debug;

/// Calculateur de statistiques pour tranches de 15 minutes
pub(super) struct Stats15MinCalculator<'a> {
    candles: &'a [Candle],
}

impl<'a> Stats15MinCalculator<'a> {
    pub(super) fn new(candles: &'a [Candle]) -> Self {
        Self { candles }
    }

    /// Calcule les statistiques pour chaque tranche de 15 minutes (en heure de Paris, UTC+1)
    /// Les candles sont en UTC, on les groupe par 15min de Paris pour une analyse de scalping fine
    pub(super) fn calculate(&self) -> Result<Vec<Stats15Min>> {
        debug!("Calculating 15-minute statistics (Paris time)");

        const PARIS_OFFSET_HOURS: i32 = 1; // UTC+1 (hiver) - TODO: gérer DST pour l'été

        // Groupe les bougies par tranche de 15 minutes de Paris
        let mut groups_15min: HashMap<(u8, u8), Vec<&Candle>> = HashMap::new();

        for candle in self.candles {
            let utc_hour = candle.hour_utc() as i32;
            let utc_minute = candle.datetime.minute() as i32;

            // Convertir en Paris time
            let paris_hour = (utc_hour + PARIS_OFFSET_HOURS) % 24;
            let paris_minute = utc_minute; // Les minutes ne changent pas avec timezone

            let paris_hour_u8 = paris_hour as u8;
            let quarter = (paris_minute / 15) as u8;

            groups_15min
                .entry((paris_hour_u8, quarter))
                .or_default()
                .push(candle);
        }

        // Calcule les stats pour chaque tranche de 15 minutes
        let mut stats = Vec::new();

        for hour in 0..24 {
            for quarter in 0..4 {
                if let Some(candles) = groups_15min.get(&(hour, quarter)) {
                    let hour_stats = self.calculate_for_slice(hour, quarter, candles)?;
                    stats.push(hour_stats);
                } else {
                    // Tranche sans données : stats vides
                    stats.push(Stats15Min {
                        hour,
                        quarter,
                        candle_count: 0,
                        atr_mean: 0.0,
                        atr_max: 0.0,
                        volatility_mean: 0.0,
                        range_mean: 0.0,
                        body_range_mean: 0.0,
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
                    });
                }
            }
        }

        debug!(
            "Calculated stats for {} 15-minute slices (Paris time)",
            stats.len()
        );
        Ok(stats)
    }

    /// Calcule les statistiques pour une tranche de 15 minutes spécifique
    fn calculate_for_slice(
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
                volatility_mean: 0.0,
                range_mean: 0.0,
                body_range_mean: 0.0,
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
            });
        }

        // Crée un vecteur owned pour le calculateur
        let owned_candles: Vec<Candle> = candles.iter().map(|&c| c.clone()).collect();
        let calc = MetricsCalculator::new(&owned_candles);

        // Calcule les métriques (avec gestion d'erreur si pas assez de données)
        let atr_values = calc.calculate_atr(14).unwrap_or_default();
        let volatility_values = calc.calculate_volatility(20).unwrap_or_default();
        let body_ranges = calc.calculate_body_ranges();
        let shadow_ratios = calc.calculate_shadow_ratios();
        let tick_qualities = calc.calculate_tick_quality();
        let noise_ratios = calc.calculate_noise_ratio();
        let tr_dist = calc.calculate_true_range_distribution()?;

        // Calcule les moyennes
        let atr_mean = atr_values.last().copied().unwrap_or(0.0); // Dernière valeur ATR lissée (Wilder's)
        let atr_max = max(&atr_values);
        let volatility_mean = mean(&volatility_values);
        // TÂCHE 3: Utiliser True Range au lieu de simple H-L
        let range_mean = mean(&tr_dist.true_ranges);
        let body_range_mean = mean(&body_ranges);
        let shadow_ratio_mean = mean(&shadow_ratios);
        let _tick_quality_mean = mean(&tick_qualities);
        let noise_ratio_mean = mean(&noise_ratios);

        // Calculate breakout percentage first
        let breakout_count = tr_dist.is_breakout.iter().filter(|&&b| b).count();
        let breakout_percentage =
            (breakout_count as f64 / tr_dist.is_breakout.len() as f64) * 100.0;

        // Direction Strength: Force directionnelle = (|directionalite| * cassures) / 10000
        // Note: Both values are percentages (0-100), so divide by 10000 to get result in 0-100 range
        let direction_strength = (body_range_mean.abs() * breakout_percentage) / 10000.0;

        // TÂCHE 4: Analyse réelle de décroissance de volatilité
        let (peak_duration, half_life, trade_exp) =
            match VolatilityDurationAnalyzer::analyze_from_candles(hour, quarter, candles) {
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

        Ok(Stats15Min {
            hour,
            quarter,
            candle_count,
            atr_mean,
            atr_max,
            volatility_mean,
            range_mean,
            body_range_mean,
            shadow_ratio_mean,
            volume_imbalance_mean: direction_strength, // Remplacé par direction_strength
            noise_ratio_mean,
            breakout_percentage,
            events: Vec::new(),
            peak_duration_minutes: peak_duration,
            volatility_half_life_minutes: half_life,
            recommended_trade_expiration_minutes: trade_exp,
            peak_duration_mean: None,
            volatility_half_life_mean: None,
            recommended_trade_expiration_mean: None,
        })
    }
}
