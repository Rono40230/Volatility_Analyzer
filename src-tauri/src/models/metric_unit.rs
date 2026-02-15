/// MetricUnit: Énumération des unités de mesure pour les métriques
/// Garantit que chaque valeur métrique a une dimension explicite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricUnit {
    /// Pourcentage (0-100)
    Percentage,
    /// Points/Pips (normalisés)
    Pips,
    /// Points (pour indices)
    Points,
    /// Ratio sans unité (1.0 - 10.0, par exemple)
    Ratio,
    /// Unitless (valeurs mathématiques sans sens physique)
    Unitless,
}

impl MetricUnit {
    /// Retourne le suffixe d'affichage pour cette unité
    pub fn display_suffix(&self) -> &'static str {
        match self {
            MetricUnit::Percentage => "%",
            MetricUnit::Pips => " pips",
            MetricUnit::Points => " pts",
            MetricUnit::Ratio => "x",
            MetricUnit::Unitless => "",
        }
    }

    /// Retourne la description humaine
    pub fn description(&self) -> &'static str {
        match self {
            MetricUnit::Percentage => "Pourcentage (0-100)",
            MetricUnit::Pips => "Points/Pips normalisés",
            MetricUnit::Points => "Points (indices)",
            MetricUnit::Ratio => "Ratio sans unité",
            MetricUnit::Unitless => "Valeur mathématique sans dimension",
        }
    }
}

/// Metric: Wrapper pour une valeur métrique avec sa dimension
#[derive(Debug, Clone)]
pub struct Metric {
    pub value: f64,
    pub unit: MetricUnit,
    pub name: &'static str,
}

impl Metric {
    /// Crée une nouvelle métrique
    pub fn new(value: f64, unit: MetricUnit, name: &'static str) -> Self {
        Metric { value, unit, name }
    }

    /// Retourne la valeur formatée avec suffixe
    pub fn formatted(&self) -> String {
        format!("{:.2}{}", self.value, self.unit.display_suffix())
    }

    /// Retourne le symbole d'unité pour la sérialisation
    pub fn unit_symbol(&self) -> &'static str {
        self.unit.display_suffix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_formatting() {
        let body_range = Metric::new(45.5, MetricUnit::Percentage, "Body Range");
        assert_eq!(body_range.formatted(), "45.50%");

        let noise = Metric::new(2.5, MetricUnit::Ratio, "Noise Ratio");
        assert_eq!(noise.formatted(), "2.50x");

        let atr = Metric::new(1.5, MetricUnit::Pips, "ATR");
        assert_eq!(atr.formatted(), "1.50 pips");
    }
}
