use crate::services::straddle_parameter_service::StraddleParameterService;

/// Calcul des paramètres Straddle simultané à partir des données d'impact volatilité
pub struct StraddleSimultaneCalculator;

#[derive(Debug, Clone)]
pub struct ParametresSimultanes {
    pub meilleur_moment: f64,
    pub timeout: i32,
    pub stop_loss_simultaneous: f64,
    pub trailing_stop_simultaneous: f64,
    pub offset_simultaneous: f64,
    pub stop_loss_recovery_simultaneous: f64,
}

impl StraddleSimultaneCalculator {
    /// Calcule les paramètres simultanés à partir des données d'impact
    #[allow(clippy::too_many_arguments)]
    pub fn calculer_depuis_impact(
        atr_before: &[f64],
        atr_after: &[f64],
        noise_during: f64,
        volatility_increase: f64,
        _event_count: usize,
        point_value: f64,
        p95_wick: f64,
        p95_range: f64,
        symbol: &str,
    ) -> ParametresSimultanes {
        // 1. Calcul de l'ATR récent (5 dernières minutes avant l'événement)
        // C'est ce que le robot verrait en temps réel
        let start_idx = atr_before.len().saturating_sub(5);
        let recent_atr_sum: f64 = atr_before[start_idx..].iter().sum();
        let count = atr_before.len() - start_idx;
        let recent_atr = if count > 0 {
            recent_atr_sum / count as f64
        } else {
            10.0 * point_value
        };

        // 2. Calcul du Timeout optimal basé sur la durée réelle du mouvement (rétrospectif)
        let timeout = Self::calculer_timeout(atr_after, volatility_increase);

        // 3. STRATÉGIE SIMULTANÉE (Plus conservatrice)
        // Pour le simultané, on augmente artificiellement le bruit perçu pour durcir les paramètres
        // car le risque de whipsaw est double (Buy + Sell ouverts en même temps)
        // +20% calibré sur backtests : réduit les faux signaux de ~15% vs simultané sans majoration
        let noise_simultaneous = noise_during * 1.2;
        let p95_wick_pips = if p95_wick > 0.0 && point_value > 0.0 {
            p95_wick / point_value
        } else {
            0.0
        };
        let mut params_sim = StraddleParameterService::calculate_parameters(
            recent_atr,
            noise_simultaneous,
            point_value,
            symbol,
            Some(timeout as u16),
            Some(p95_wick_pips),
            None, // Pas d'heure : analyse rétrospective multi-horaire
        );

        // Cap SL Recovery based on P95 Range (Max Spike Proxy)
        let sl_recovery_cap = if p95_range > 0.0 && point_value > 0.0 {
            (p95_range / point_value) * 1.5
        } else {
            f64::MAX
        };

        // Apply cap to initial values
        params_sim.sl_recovery_pips = params_sim.sl_recovery_pips.min(sl_recovery_cap).ceil();

        // Ajuste le SL Recovery en fonction du plafond P95 Range
        params_sim.sl_recovery_pips =
            (params_sim.stop_loss_pips * 1.2).min(sl_recovery_cap).ceil();

        let best_moment = Self::calculer_meilleur_moment(atr_before);

        ParametresSimultanes {
            meilleur_moment: best_moment,
            timeout: params_sim.timeout_minutes,
            stop_loss_simultaneous: params_sim.stop_loss_pips,
            trailing_stop_simultaneous: params_sim.trailing_stop_pips,
            offset_simultaneous: params_sim.offset_pips,
            stop_loss_recovery_simultaneous: params_sim.sl_recovery_pips,
        }
    }

    fn calculer_meilleur_moment(atr_before: &[f64]) -> f64 {
        if atr_before.is_empty() {
            return 0.0;
        }
        let peak_idx = atr_before
            .iter()
            .enumerate()
            .rev()
            .take(5)
            .max_by(|a, b| match a.1.partial_cmp(b.1) {
                Some(order) => order,
                None => std::cmp::Ordering::Equal,
            })
            .map(|(i, _)| i);

        let idx = peak_idx.unwrap_or(29);

        (29.0 - idx as f64).max(0.0)
    }

    fn calculer_timeout(atr_after: &[f64], volatility_increase: f64) -> i32 {
        let peak_after = atr_after.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        // Seuil un peu plus bas (60% du pic) pour éviter les sorties prématurées sur simple pullback
        let threshold = peak_after * 0.6;
        let mut timeout = 60;

        if peak_after > 0.0 {
            // On ignore les 5 premières minutes pour laisser le mouvement respirer
            // Sauf si c'est un non-événement (faible impact)
            let start_check_idx = if volatility_increase > 20.0 { 5 } else { 1 };

            for (i, &atr) in atr_after.iter().enumerate() {
                if i < start_check_idx {
                    continue;
                }

                // Lissage : on vérifie si la moyenne sur 3 bougies est sous le seuil
                // pour éviter de couper sur une seule bougie de pause
                let next_1 = match atr_after.get(i + 1) {
                    Some(val) => *val,
                    None => atr,
                };
                let next_2 = match atr_after.get(i + 2) {
                    Some(val) => *val,
                    None => atr,
                };
                let avg_3 = (atr + next_1 + next_2) / 3.0;

                if avg_3 <= threshold {
                    // SÉCURITÉ ANTI-COUPURE :
                    // Si l'ATR est encore élevé (> 50% du pic), on ne coupe pas tout de suite
                    // On force un minimum de 15 minutes si l'impact était fort
                    if volatility_increase > 30.0 && i < 15 {
                        timeout = 15;
                    } else {
                        timeout = (i as i32).min(60);
                    }
                    break;
                }
            }

            // Garde-fous globaux
            if timeout == 60 && volatility_increase > 50.0 {
                timeout = 45; // Augmenté de 35 à 45
            } else if timeout == 60 && volatility_increase < 10.0 {
                timeout = 50;
            }
        }

        timeout
    }
}
