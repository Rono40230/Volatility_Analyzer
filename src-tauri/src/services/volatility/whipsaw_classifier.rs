// services/volatility/whipsaw_classifier.rs - Classification Early/Late
use super::whipsaw_detector::{WhipsawDetail, WhipsawRiskLevel};

/// Classification Early/Late d'un whipsaw
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhipsawTiming {
    Early,    // Buy/Sell déclenché < 8 min après entry
    Late,     // Buy/Sell déclenché >= 8 min après entry
    Balanced, // Timing similaire pour les 2 côtés
}

/// Résultat complet de l'analyse Early/Late
#[derive(Debug, Clone)]
pub struct WhipsawRootCauseAnalysis {
    pub early_count: usize,
    pub late_count: usize,
    pub early_avg_loss_pips: f64,
    pub late_avg_loss_pips: f64,
    pub dominant_timing: WhipsawTiming,
}

impl WhipsawRootCauseAnalysis {
    /// Analyse les whipsaws pour déterminer si Early ou Late
    pub fn analyze(
        whipsaws: &[WhipsawDetail],
        peak_duration: u16,
        offset_pips: f64,
    ) -> Self {
        let mut early_count = 0;
        let mut late_count = 0;
        let mut early_losses = Vec::new();
        let mut late_losses = Vec::new();
        let timing_threshold = (peak_duration as f64 * 0.5) as usize;

        for whipsaw in whipsaws {
            let buy_delay = whipsaw.buy_trigger_index.saturating_sub(whipsaw.entry_index);
            let sell_delay = whipsaw.sell_trigger_index.saturating_sub(whipsaw.entry_index);
            let first_trigger = buy_delay.min(sell_delay);

            if first_trigger < timing_threshold {
                early_count += 1;
                early_losses.push(offset_pips);
            } else {
                late_count += 1;
                late_losses.push(offset_pips);
            }
        }

        let early_avg = if !early_losses.is_empty() {
            early_losses.iter().sum::<f64>() / early_losses.len() as f64
        } else {
            0.0
        };

        let late_avg = if !late_losses.is_empty() {
            late_losses.iter().sum::<f64>() / late_losses.len() as f64
        } else {
            0.0
        };

        let dominant_timing = if early_count > late_count {
            WhipsawTiming::Early
        } else if late_count > early_count {
            WhipsawTiming::Late
        } else {
            WhipsawTiming::Balanced
        };

        Self {
            early_count,
            late_count,
            early_avg_loss_pips: early_avg,
            late_avg_loss_pips: late_avg,
            dominant_timing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_whipsaw(entry_idx: usize, buy_trigger: usize, sell_trigger: usize) -> WhipsawDetail {
        WhipsawDetail {
            entry_index: entry_idx,
            entry_price: 1.0800,
            buy_stop: 1.0850,
            sell_stop: 1.0750,
            buy_trigger_index: buy_trigger,
            sell_trigger_index: sell_trigger,
        }
    }

    #[test]
    fn test_early_whipsaw() {
        let whipsaws = vec![create_test_whipsaw(0, 2, 3)];
        let analysis = WhipsawRootCauseAnalysis::analyze(&whipsaws, 100, 50.0);
        assert_eq!(analysis.early_count, 1);
        assert_eq!(analysis.late_count, 0);
    }

    #[test]
    fn test_late_whipsaw() {
        let whipsaws = vec![create_test_whipsaw(0, 50, 52)];
        let analysis = WhipsawRootCauseAnalysis::analyze(&whipsaws, 100, 50.0);
        assert_eq!(analysis.early_count, 0);
        assert_eq!(analysis.late_count, 1);
    }

    #[test]
    fn test_mixed_whipsaws() {
        let whipsaws = vec![
            create_test_whipsaw(0, 2, 3),
            create_test_whipsaw(10, 50, 52),
        ];
        let analysis = WhipsawRootCauseAnalysis::analyze(&whipsaws, 100, 50.0);
        assert_eq!(analysis.early_count, 1);
        assert_eq!(analysis.late_count, 1);
    }
}
