// models/entry_analysis.rs — Modèle de résultat d'analyse de point d'entrée straddle
// Remplace l'ancien scoring heuristique par des métriques basées sur le profit net réel.

use serde::{Deserialize, Serialize};

/// Configuration de l'analyse de point d'entrée
#[derive(Debug, Clone)]
pub struct EntryAnalysisConfig {
    /// Nombre de minutes à regarder après l'entrée (défaut: 30)
    pub forward_minutes: usize,
    /// Seuil de spread en pips au-delà duquel une minute est non-tradable (défaut: 10.0)
    pub spread_threshold_pips: f64,
    /// Nombre minimum d'échantillons pour considérer un offset valide (défaut: 5)
    pub min_samples: usize,
}

impl Default for EntryAnalysisConfig {
    fn default() -> Self {
        Self {
            forward_minutes: 30,
            spread_threshold_pips: 10.0,
            min_samples: 5,
        }
    }
}

/// Détail d'un offset minute (0-14) dans le quarter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteDetail {
    pub offset: u8,
    pub win_rate: f64,
    pub avg_net_profit_pips: f64,
    pub avg_spread_pips: f64,
    pub sample_size: usize,
    pub tradable: bool,
}

/// Résultat complet d'analyse d'entrée pour un symbol × event_type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryAnalysisResult {
    pub symbol: String,
    pub event_type: String,
    /// Offset optimal 0-14 dans le quarter
    pub optimal_offset_minutes: u8,
    /// Label lisible "HH:MM UTC"
    pub optimal_entry_time_label: String,
    /// Win rate réel = count(profit > 0) / count(total)
    pub real_win_rate: f64,
    /// Profit net moyen en pips (après spread)
    pub avg_net_profit_pips: f64,
    /// Spread moyen à l'entrée en pips
    pub avg_spread_at_entry_pips: f64,
    /// Mouvement brut moyen en pips (avant spread)
    pub avg_movement_pips: f64,
    /// Minute forward où le mouvement atteint son pic
    pub peak_minute: u8,
    /// Durée du mouvement au-dessus de 50% du pic
    pub movement_duration_minutes: f64,
    /// Vitesse de décroissance: FAST / MEDIUM / SLOW / UNKNOWN
    pub decay_speed: String,
    /// Consistance: 1 - (std_dev / mean), clampé [0, 1]
    pub consistency_score: f64,
    /// Nombre d'occurrences historiques analysées
    pub sample_size: usize,
    /// Minutes où le spread moyen dépasse le seuil
    pub non_tradable_minutes: Vec<u8>,
    /// Détail par minute (15 entrées, offsets 0-14)
    pub minute_details: Vec<MinuteDetail>,
    /// Unité d'affichage (pips, $, pts)
    pub unit: String,
}
