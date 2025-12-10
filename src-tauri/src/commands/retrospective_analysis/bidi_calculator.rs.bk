/// Calcul des paramètres Bidi Straddle à partir des données d'impact volatilité
pub struct BidiCalculator;

impl BidiCalculator {
    /// Calcul des 4 paramètres Bidi à partir des données d'impact
    /// Retourne: (meilleur_moment, stop_loss, trailing_stop, timeout, offset, stop_loss_recovery)
    pub fn calculate_from_impact(
        atr_before: &[f64],
        atr_after: &[f64],
        noise_during: f64,
        volatility_increase: f64,
        _event_count: usize,
    ) -> (f64, f64, f64, i32, f64, f64) {
        let best_moment = Self::calculate_best_moment(atr_before);
        let stop_loss = Self::calculate_stop_loss(atr_before, atr_after, noise_during);
        let trailing_stop = Self::calculate_trailing_stop(atr_before, atr_after, noise_during);
        let timeout = Self::calculate_timeout(atr_after, volatility_increase);
        let offset = Self::calculate_offset(atr_before, noise_during);

        // Calcul du SL Recovery pour Straddle Simultané
        // Il doit être suffisant pour atteindre l'autre côté (2x offset) + une marge de sécurité
        // On prend le max entre le SL standard et 3.0x l'offset (augmenté de 2.5 pour plus de sécurité)
        let stop_loss_recovery = stop_loss.max(offset * 3.0).ceil();

        (best_moment, stop_loss, trailing_stop, timeout, offset, stop_loss_recovery)
    }

    fn calculate_best_moment(atr_before: &[f64]) -> f64 {
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

    fn calculate_offset(atr_before: &[f64], noise_during: f64) -> f64 {
        if atr_before.is_empty() {
            return 10.0; // Valeur par défaut
        }
        
        // On regarde l'ATR moyen sur les 5 dernières minutes avant l'événement
        // C'est la volatilité "récente" au moment où on place l'ordre
        let start_idx = atr_before.len().saturating_sub(5);
        let recent_atr_sum: f64 = atr_before[start_idx..].iter().sum();
        let count = atr_before.len() - start_idx;
        let recent_atr = if count > 0 { recent_atr_sum / count as f64 } else { 10.0 };

        // Si le marché est bruyant (Noise > 2.0), on s'écarte davantage (1.5x ATR)
        // Sinon on reste assez proche (1.2x ATR) pour ne pas rater le départ
        let multiplier = if noise_during > 2.0 { 1.5 } else { 1.2 };

        (recent_atr * multiplier).ceil()
    }

    fn calculate_stop_loss(atr_before: &[f64], atr_after: &[f64], noise_during: f64) -> f64 {
        let atr_mean = (atr_before.iter().sum::<f64>() + atr_after.iter().sum::<f64>())
            / (atr_before.len() + atr_after.len()) as f64;

        let sl_ratio = if noise_during > 3.0 {
            3.0
        } else if noise_during > 2.5 {
            2.5
        } else if noise_during > 2.0 {
            2.0
        } else if noise_during > 1.5 {
            1.75
        } else {
            1.5
        };

        (atr_mean * sl_ratio).ceil()
    }

    fn calculate_trailing_stop(atr_before: &[f64], atr_after: &[f64], noise_during: f64) -> f64 {
        let atr_mean = (atr_before.iter().sum::<f64>() + atr_after.iter().sum::<f64>())
            / (atr_before.len() + atr_after.len()) as f64;

        // NOUVELLE LOGIQUE : Plus de bruit = Plus d'espace pour respirer
        // On suit la logique du SL mais avec un ratio plus serré (environ 30-40% du SL)
        let ts_ratio = if noise_during > 3.0 {
            1.2 // Bruit extrême (>3) -> TS = 1.2x ATR (ex: ATR 40 -> TS 48)
        } else if noise_during > 2.0 {
            1.0 // Bruit fort (>2) -> TS = 1.0x ATR (ex: ATR 40 -> TS 40)
        } else if noise_during > 1.5 {
            0.8 // Bruit moyen (>1.5) -> TS = 0.8x ATR (ex: ATR 40 -> TS 32)
        } else {
            0.6 // Bruit faible -> TS = 0.6x ATR (ex: ATR 40 -> TS 24)
        };

        (atr_mean * ts_ratio).ceil()
    }

    fn calculate_timeout(atr_after: &[f64], volatility_increase: f64) -> i32 {
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
