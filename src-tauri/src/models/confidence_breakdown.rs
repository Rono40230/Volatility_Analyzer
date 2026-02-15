/// ConfidenceBreakdown: Détail composant-par-composant du score de confiance
/// Permet au trader de voir POURQUOI le score est X
/// et de valider la décision manuellement au lieu de faire confiance aveuglément
#[derive(Debug, Clone)]
pub struct ConfidenceBreakdown {
    /// Score ATR (pips) - volatilité soutenue
    pub atr_score: f64,
    pub atr_points: f64,
    pub atr_reasoning: String,

    /// Score Body Range (%) - directionnalité des bougies
    pub body_range_score: f64,
    pub body_range_points: f64,
    pub body_range_reasoning: String,

    /// Score Volatility (%) - bonus si marché bouge
    pub volatility_score: f64,
    pub volatility_points: f64,
    pub volatility_reasoning: String,

    /// Score Noise Ratio - signal/bruit
    pub noise_ratio_score: f64,
    pub noise_ratio_points: f64,
    pub noise_ratio_reasoning: String,

    /// Score Breakout % - % de cassures
    pub breakout_score: f64,
    pub breakout_points: f64,
    pub breakout_reasoning: String,

    /// Bonus données
    pub bonus_points: f64,
    pub bonus_reasoning: String,

    /// Liste des pénalités appliquées
    pub penalties: Vec<(f64, String)>, // (points, reason)

    /// Total brut (avant clamp)
    pub raw_total: f64,

    /// Total final [0, 100]
    pub total: f64,

    /// Interprétation du score
    pub interpretation: String,
}

impl ConfidenceBreakdown {
    /// Crée un breakdown vide
    pub fn new() -> Self {
        ConfidenceBreakdown {
            atr_score: 0.0,
            atr_points: 0.0,
            atr_reasoning: String::new(),
            body_range_score: 0.0,
            body_range_points: 0.0,
            body_range_reasoning: String::new(),
            volatility_score: 0.0,
            volatility_points: 0.0,
            volatility_reasoning: String::new(),
            noise_ratio_score: 0.0,
            noise_ratio_points: 0.0,
            noise_ratio_reasoning: String::new(),
            breakout_score: 0.0,
            breakout_points: 0.0,
            breakout_reasoning: String::new(),
            bonus_points: 0.0,
            bonus_reasoning: String::new(),
            penalties: Vec::new(),
            raw_total: 0.0,
            total: 0.0,
            interpretation: String::new(),
        }
    }

    /// Ajoute une pénalité
    pub fn add_penalty(&mut self, points: f64, reason: String) {
        self.penalties.push((points, reason));
    }

    /// Finalise le breakdown (calcul total, interpretation)
    pub fn finalize(&mut self) {
        let mut total = 0.0;
        total += self.atr_points;
        total += self.body_range_points;
        total += self.volatility_points;
        total += self.noise_ratio_points;
        total += self.breakout_points;
        total += self.bonus_points;

        for (penalty, _) in &self.penalties {
            total += penalty; // penalty is negative
        }

        self.raw_total = total;
        self.total = total.clamp(0.0, 100.0);

        // Déterminer l'interprétation
        self.interpretation = match self.total {
            x if x >= 80.0 => "🟢 EXCELLENT - Scalpe agressivement".to_string(),
            x if x >= 65.0 => "🔵 BON - Scalpe normalement".to_string(),
            x if x >= 50.0 => "🟡 PRUDENT - Scalpe avec stops serrés".to_string(),
            x if x >= 35.0 => "🟠 RISKY - Très prudent, breakouts only".to_string(),
            _ => "❌ MAUVAIS - Ne pas trader".to_string(),
        };
    }

    /// Génère un rapport lisible
    pub fn report(&self) -> String {
        let mut lines = vec![
            "═══ CONFIDENCE SCORE BREAKDOWN ═══".to_string(),
            String::new(),
            "📊 Composants:".to_string(),
            format!("  • ATR ({:.2} pips): {} pts → {}", 
                self.atr_score, self.atr_points, self.atr_reasoning),
            format!("  • Body Range ({:.1}%): {} pts → {}", 
                self.body_range_score, self.body_range_points, self.body_range_reasoning),
            format!("  • Volatility ({:.1}%): {} pts → {}", 
                self.volatility_score, self.volatility_points, self.volatility_reasoning),
            format!("  • Noise Ratio ({:.2}x): {} pts → {}", 
                self.noise_ratio_score, self.noise_ratio_points, self.noise_ratio_reasoning),
            format!("  • Breakout ({:.1}%): {} pts → {}", 
                self.breakout_score, self.breakout_points, self.breakout_reasoning),
            format!("  • Bonus: {} pts → {}", 
                self.bonus_points, self.bonus_reasoning),
        ];

        if !self.penalties.is_empty() {
            lines.push(String::new());
            lines.push("⚠️ Pénalités:".to_string());
            for (penalty, reason) in &self.penalties {
                lines.push(format!("  • {} pts → {}", penalty, reason));
            }
        }

        lines.push(String::new());
        lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
        lines.push(format!("Sous-total: {:.1} pts", self.raw_total));
        lines.push(format!("Total (clamped [0, 100]): {:.1} pts", self.total));
        lines.push(format!("Interprétation: {}", self.interpretation));

        lines.join("\n")
    }
}

impl Default for ConfidenceBreakdown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakdown_finalize() {
        let mut bd = ConfidenceBreakdown::new();
        bd.atr_score = 2.5;
        bd.atr_points = 30.0;
        bd.atr_reasoning = "Excellent".to_string();

        bd.body_range_score = 52.0;
        bd.body_range_points = 25.0;
        bd.body_range_reasoning = "Strong directional".to_string();

        bd.finalize();

        assert_eq!(bd.total, 55.0);
        assert!(bd.interpretation.contains("BON"));
    }

    #[test]
    fn test_breakdown_with_penalties() {
        let mut bd = ConfidenceBreakdown::new();
        bd.atr_points = 30.0;
        bd.body_range_points = 25.0;
        bd.add_penalty(-15.0, "Chaos detected".to_string());

        bd.finalize();

        assert_eq!(bd.total, 40.0);
    }

    #[test]
    fn test_breakdown_clamp() {
        let mut bd = ConfidenceBreakdown::new();
        bd.atr_points = 100.0; // Over limit
        bd.finalize();

        assert_eq!(bd.total, 100.0); // Should be clamped
    }
}
