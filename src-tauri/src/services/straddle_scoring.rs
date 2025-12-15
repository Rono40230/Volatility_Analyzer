use crate::models::Candle;

#[derive(Debug, Clone)]
pub struct StraddleScore {
    pub total_score: f64, // 0-100
    #[allow(dead_code)]
    pub volatility_score: f64, // 0-100
    pub directionality_score: f64, // 0-100
    #[allow(dead_code)]
    pub reliability_score: f64, // 0-100
    pub whipsaw_risk: f64, // 0-100 (0 = safe, 100 = dangerous)
}

pub struct StraddleScoreCalculator;

impl StraddleScoreCalculator {
    /// Calcule le score Straddle complet
    pub fn calculer(
        candles_before: &[Candle],
        candles_after: &[Candle],
        point_value: f64,
    ) -> StraddleScore {
        if candles_after.is_empty() {
            return StraddleScore {
                total_score: 0.0,
                volatility_score: 0.0,
                directionality_score: 0.0,
                reliability_score: 0.0,
                whipsaw_risk: 0.0,
            };
        }

        let volatility =
            Self::calculer_score_volatilite(candles_before, candles_after, point_value);
        let directionality = Self::calculer_score_directionnalite(candles_after);
        let whipsaw = Self::calculer_risque_whipsaw(candles_after, point_value);

        // Reliability nécessite un historique d'événements, ici on analyse une seule occurrence
        // Pour l'instant on met 100, la moyenne sera faite par l'appelant sur plusieurs occurrences
        let reliability = 100.0;

        // Formule du score : (Vol * Dir * Rel) / (1 + Whipsaw)
        // On pondère pour rester entre 0 et 100

        // Pénalité Whipsaw exponentielle
        // Assouplissement v2 : seuils 40/60 au lieu de 30/50 pour tolérer plus de bruit (crypto)
        let whipsaw_penalty = if whipsaw > 60.0 {
            0.1
        } else if whipsaw > 40.0 {
            0.5
        } else {
            1.0
        };

        // Assouplissement Directionnalité :
        // Au lieu de multiplier directement (ce qui tue le score si directionnalité faible),
        // on pondère : même avec 0 directionnalité, on garde 40% du score de volatilité.
        // Cela permet aux mouvements volatils mais "sales" (mèches) de rester tradables.
        let dir_factor = 0.4 + 0.6 * (directionality / 100.0);

        let raw_score = volatility * dir_factor * whipsaw_penalty;

        StraddleScore {
            total_score: raw_score.min(100.0),
            volatility_score: volatility,
            directionality_score: directionality,
            reliability_score: reliability,
            whipsaw_risk: whipsaw,
        }
    }

    fn calculer_score_volatilite(before: &[Candle], after: &[Candle], _point_value: f64) -> f64 {
        let atr_before = Self::calculer_atr(before);
        let atr_after = Self::calculer_atr(after);

        if atr_before == 0.0 {
            return 0.0;
        }

        let increase_pct = ((atr_after - atr_before) / atr_before) * 100.0;

        // Score 0-100 :
        // < 0% = 0
        // 80% increase = 100 (Score Max atteint plus vite)
        // Ajustement v2 : Diviseur 0.8 pour booster les scores de volatilité
        (increase_pct / 0.8).clamp(0.0, 100.0)
    }

    fn calculer_score_directionnalite(after: &[Candle]) -> f64 {
        // Moyenne du ratio Body/Range sur les bougies après l'événement
        let mut total_ratio = 0.0;
        let mut count = 0;

        for c in after.iter().take(5) {
            // On regarde les 5 premières minutes (réaction immédiate)
            let range = c.high - c.low;
            let body = (c.close - c.open).abs();

            if range > 0.0 {
                total_ratio += body / range;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        (total_ratio / count as f64) * 100.0
    }

    fn calculer_risque_whipsaw(after: &[Candle], _point_value: f64) -> f64 {
        // Détection de mèches opposées importantes
        let mut max_whipsaw = 0.0;

        for c in after.iter().take(5) {
            let body_top = c.open.max(c.close);
            let body_bottom = c.open.min(c.close);

            let upper_wick = c.high - body_top;
            let lower_wick = body_bottom - c.low;

            let range = c.high - c.low;

            if range > 0.0 {
                // Si les deux mèches sont grandes (> 20% du range chacune), c'est un whipsaw
                if upper_wick > range * 0.2 && lower_wick > range * 0.2 {
                    let risk = ((upper_wick + lower_wick) / range) * 100.0;
                    if risk > max_whipsaw {
                        max_whipsaw = risk;
                    }
                }
            }
        }

        max_whipsaw
    }

    fn calculer_atr(candles: &[Candle]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }
        let sum: f64 = candles.iter().map(|c| c.high - c.low).sum();
        sum / candles.len() as f64
    }
}
