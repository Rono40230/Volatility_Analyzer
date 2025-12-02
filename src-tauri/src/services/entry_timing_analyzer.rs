// services/entry_timing_analyzer.rs - Analyse profitabilité par offset d'entrée
// PHASE 7a-3: Entry Timing Analyzer
// Stratifier résultats backtesting par T-10,-5,0,+3 minutes

use crate::models::VolatilityError;
use serde::{Deserialize, Serialize};

/// Ligne d'analyse: profitabilité pour UN offset d'entrée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryTimingRow {
    pub entry_offset_minutes: i8,  // -10, -5, 0, 3
    pub win_rate: f64,             // 0-100 (%)
    pub whipsaw_rate: f64,         // 0-100 (%)
    pub avg_profit_pips: f64,      // pips (peut être négatif)
    pub sample_size: usize,        // nombre de trades
}

impl EntryTimingRow {
    /// Créer une nouvelle ligne avec données
    pub fn new(
        offset: i8,
        win_rate: f64,
        whipsaw_rate: f64,
        avg_profit: f64,
        sample_size: usize,
    ) -> Self {
        Self {
            entry_offset_minutes: offset,
            win_rate,
            whipsaw_rate,
            avg_profit_pips: avg_profit,
            sample_size,
        }
    }

    /// Évaluer la qualité de cet offset (0-100)
    pub fn quality_score(&self) -> f64 {
        if self.sample_size == 0 {
            return 0.0;
        }

        // Score basé sur win_rate (50%) + avg_profit (40%) + whipsaw_rate (10%)
        let win_score = (self.win_rate / 100.0) * 50.0;
        let profit_score = if self.avg_profit_pips > 0.0 {
            (self.avg_profit_pips.min(100.0) / 100.0) * 40.0
        } else {
            0.0
        };
        let whipsaw_score = ((100.0 - self.whipsaw_rate) / 100.0) * 10.0;

        win_score + profit_score + whipsaw_score
    }
}

/// Matrice complète: 4 offsets analysés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryTimingMatrix {
    pub rows: Vec<EntryTimingRow>,
    pub best_offset: i8,
    pub best_score: f64,
}

impl EntryTimingMatrix {
    /// Créer une matrice vide
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            best_offset: 0,
            best_score: 0.0,
        }
    }

    /// Ajouter une ligne
    pub fn add_row(&mut self, row: EntryTimingRow) {
        let score = row.quality_score();
        if score > self.best_score {
            self.best_score = score;
            self.best_offset = row.entry_offset_minutes;
        }
        self.rows.push(row);
    }

    /// Sortir les 4 offsets standard (même vides)
    pub fn with_standard_offsets() -> Self {
        let mut matrix = Self::new();
        for offset in &[-10i8, -5i8, 0i8, 3i8] {
            matrix.add_row(EntryTimingRow::new(*offset, 0.0, 0.0, 0.0, 0));
        }
        matrix
    }
}

impl Default for EntryTimingMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyser profitabilité par offset d'entrée
///
/// Input: backtests (Vec de résultats trade avec entry_offset, win/loss, profit)
/// Output: EntryTimingMatrix avec 4 offsets stratifiés
pub struct EntryTimingAnalyzer;

impl EntryTimingAnalyzer {
    /// Analyser les résultats de trading par offset
    ///
    /// Cette fonction prend une liste de résultats backtesting
    /// et les stratifie par offset d'entrée (T-10, T-5, T-0, T+3)
    pub fn analyze(
        backtest_results: &[(i8, bool, f64)], // (offset, win, profit_pips)
    ) -> Result<EntryTimingMatrix, VolatilityError> {
        if backtest_results.is_empty() {
            return Ok(EntryTimingMatrix::with_standard_offsets());
        }

        let mut matrix = EntryTimingMatrix::with_standard_offsets();

        // Grouper par offset
        for offset in &[-10i8, -5i8, 0i8, 3i8] {
            let offset_trades: Vec<_> = backtest_results
                .iter()
                .filter(|(o, _, _)| o == offset)
                .collect();

            if !offset_trades.is_empty() {
                let total = offset_trades.len() as f64;
                let wins = offset_trades.iter().filter(|(_, w, _)| *w).count() as f64;
                let whipsaws = offset_trades.iter().filter(|(_, w, _)| !*w).count() as f64;

                let win_rate = (wins / total) * 100.0;
                let whipsaw_rate = (whipsaws / total) * 100.0;
                let avg_profit = offset_trades
                    .iter()
                    .map(|(_, _, p)| p)
                    .sum::<f64>()
                    / total;

                let row = EntryTimingRow::new(*offset, win_rate, whipsaw_rate, avg_profit, offset_trades.len());

                // Remplacer la ligne vide par la ligne calculée
                if let Some(idx) = matrix.rows.iter().position(|r| r.entry_offset_minutes == *offset) {
                    matrix.rows[idx] = row;
                }
            }
        }

        // Recalculer best_offset et best_score
        matrix.best_offset = 0;
        matrix.best_score = 0.0;
        for row in &matrix.rows {
            let score = row.quality_score();
            if score > matrix.best_score {
                matrix.best_score = score;
                matrix.best_offset = row.entry_offset_minutes;
            }
        }

        Ok(matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_timing_row_quality_score() {
        let row = EntryTimingRow::new(-5, 52.0, 10.0, 18.0, 100);
        let score = row.quality_score();
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_entry_timing_row_empty() {
        let row = EntryTimingRow::new(0, 0.0, 0.0, 0.0, 0);
        assert_eq!(row.quality_score(), 0.0);
    }

    #[test]
    fn test_entry_timing_matrix_standard_offsets() {
        let matrix = EntryTimingMatrix::with_standard_offsets();
        assert_eq!(matrix.rows.len(), 4);
        assert_eq!(matrix.rows[0].entry_offset_minutes, -10);
        assert_eq!(matrix.rows[1].entry_offset_minutes, -5);
        assert_eq!(matrix.rows[2].entry_offset_minutes, 0);
        assert_eq!(matrix.rows[3].entry_offset_minutes, 3);
    }

    #[test]
    fn test_entry_timing_analyze_empty() {
        let results = vec![];
        let matrix = EntryTimingAnalyzer::analyze(&results).expect("analyze");
        assert_eq!(matrix.rows.len(), 4);
        assert_eq!(matrix.best_score, 0.0);
    }

    #[test]
    fn test_entry_timing_analyze_single_offset() {
        // Simuler 10 trades à T-5: 5 wins, 5 losses
        let results = vec![
            (-5i8, true, 20.0),
            (-5i8, true, 25.0),
            (-5i8, true, 18.0),
            (-5i8, true, 22.0),
            (-5i8, true, 30.0),
            (-5i8, false, -15.0),
            (-5i8, false, -12.0),
            (-5i8, false, -18.0),
            (-5i8, false, -20.0),
            (-5i8, false, -25.0),
        ];

        let matrix = EntryTimingAnalyzer::analyze(&results).expect("analyze");
        assert_eq!(matrix.rows.len(), 4);

        let row_m5 = &matrix.rows.iter().find(|r| r.entry_offset_minutes == -5).expect("find row");
        assert_eq!(row_m5.win_rate, 50.0);
        assert_eq!(row_m5.whipsaw_rate, 50.0);
        assert_eq!(row_m5.sample_size, 10);
        assert!(row_m5.avg_profit_pips > 0.0);
    }

    #[test]
    fn test_entry_timing_analyze_best_offset() {
        // T-5 très bon, T-0 moyen
        let results = vec![
            // T-5: 8/10 wins
            (-5i8, true, 20.0),
            (-5i8, true, 25.0),
            (-5i8, true, 18.0),
            (-5i8, true, 22.0),
            (-5i8, true, 30.0),
            (-5i8, true, 28.0),
            (-5i8, true, 24.0),
            (-5i8, true, 26.0),
            (-5i8, false, -15.0),
            (-5i8, false, -12.0),
            // T-0: 5/10 wins
            (0i8, true, 10.0),
            (0i8, true, 12.0),
            (0i8, true, 8.0),
            (0i8, true, 15.0),
            (0i8, true, 14.0),
            (0i8, false, -8.0),
            (0i8, false, -10.0),
            (0i8, false, -9.0),
            (0i8, false, -7.0),
            (0i8, false, -11.0),
        ];

        let matrix = EntryTimingAnalyzer::analyze(&results).expect("analyze");
        assert_eq!(matrix.best_offset, -5);
        assert!(matrix.best_score > 0.0);
    }
}
