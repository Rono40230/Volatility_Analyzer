use crate::services::straddle_parameter_service::StraddleParameterService;

/// Calcul des paramètres Bidi Straddle à partir des données d'impact volatilité
pub struct BidiCalculator;

impl BidiCalculator {
    /// Calcul des 4 paramètres Bidi à partir des données d'impact
    /// Retourne: (meilleur_moment, stop_loss, trailing_stop, timeout, offset, stop_loss_recovery, sl_sim, ts_sim, offset_sim)
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
    ) -> (f64, f64, f64, i32, f64, f64, f64, f64, f64, f64) {
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

        // 3. STRATÉGIE DIRECTIONNELLE (Standard)
        let mut params_dir = StraddleParameterService::calculate_parameters(
            recent_atr,
            noise_during,
            point_value,
            symbol,
            Some(timeout as u16),
        );

        // 4. STRATÉGIE SIMULTANÉE (Plus conservatrice)
        // Pour le simultané, on augmente artificiellement le bruit perçu pour durcir les paramètres
        // car le risque de whipsaw est double (Buy + Sell ouverts)
        let noise_simultaneous = noise_during * 1.2; // +20% de sensibilité au bruit
        let mut params_sim = StraddleParameterService::calculate_parameters(
            recent_atr,
            noise_simultaneous,
            point_value,
            symbol,
            Some(timeout as u16),
        );

        // Cap SL Recovery based on P95 Range (Max Spike Proxy)
        let sl_recovery_cap = if p95_range > 0.0 && point_value > 0.0 {
            (p95_range / point_value) * 1.5
        } else {
            f64::MAX
        };

        // Apply cap to initial values
        params_dir.sl_recovery_pips = params_dir.sl_recovery_pips.min(sl_recovery_cap).ceil();
        params_sim.sl_recovery_pips = params_sim.sl_recovery_pips.min(sl_recovery_cap).ceil();

        // 5. Override Offset avec P95 Wick (Logique Unifiée avec Volatilité Brute)
        if p95_wick > 0.0 && point_value > 0.0 {
            let p95_wick_points = p95_wick / point_value;
            let offset_p95 = p95_wick_points * 1.1;
            
            // Directionnel : On prend le max entre l'ATR et le P95
            params_dir.offset_pips = (offset_p95 + params_dir.spread_safety_margin_pips).ceil();
            params_dir.sl_recovery_pips = (params_dir.stop_loss_pips * 1.2).min(sl_recovery_cap).ceil();

            // Simultané : On est encore plus prudent sur l'offset pour éviter le déclenchement intempestif
            // On ajoute 10% de marge supplémentaire par rapport au directionnel
            params_sim.offset_pips = (params_dir.offset_pips * 1.1).ceil();
            params_sim.sl_recovery_pips = (params_sim.stop_loss_pips * 1.2).min(sl_recovery_cap).ceil();
        }

        let best_moment = Self::calculer_meilleur_moment(atr_before);

        (
            best_moment,
            params_dir.stop_loss_pips,
            params_dir.trailing_stop_pips,
            params_dir.timeout_minutes,
            params_dir.offset_pips,
            params_dir.sl_recovery_pips,
            // Nouveaux paramètres spécifiques Simultané
            params_sim.stop_loss_pips,     // SL Simultané (souvent plus large)
            params_sim.trailing_stop_pips, // TS Simultané
            params_sim.offset_pips,        // Offset Simultané
            params_sim.sl_recovery_pips,   // SL Recovery Simultané
        )
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
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(29);
        (29.0 - peak_idx as f64).max(0.0)
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
                let next_1 = atr_after.get(i + 1).unwrap_or(&atr);
                let next_2 = atr_after.get(i + 2).unwrap_or(&atr);
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
