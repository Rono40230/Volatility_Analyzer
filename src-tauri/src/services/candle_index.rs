// services/candle_index.rs
// Index en mémoire pour recherche rapide de candles par date
// Utilise BTreeMap pour requêtes range O(log n) au lieu de O(n) linéaire

use std::collections::{HashMap, BTreeMap};
use chrono::{NaiveDate, DateTime, Utc, Timelike};
use crate::models::Candle;
use crate::services::CsvLoader;

/// Structure pour stocker les candles indexées par paire et par date
/// Permet des recherches O(log n) au lieu de O(n)
pub struct CandleIndex {
    /// HashMap<pair_symbol, BTreeMap<NaiveDate, Vec<Candle>>>
    data: HashMap<String, BTreeMap<NaiveDate, Vec<Candle>>>,
}

impl CandleIndex {
    /// Crée un nouvel index vide
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Crée un index vide SANS charger les CSV (lazy loading)
    /// Les paires sont chargées à la demande avec load_pair_candles()
    pub fn new_lazy() -> Result<Self, String> {
        // Juste vérifie que les symboles existent
        let loader = CsvLoader::new();
        let _symbols = loader
            .list_available_symbols()
            .map_err(|e| format!("Failed to list symbols: {}", e))?;
        
        Ok(CandleIndex::new())
    }

    /// Charge et indexe ALL CSV files au démarrage
    /// Appelé UNE SEULE FOIS au startup de l'app
    /// NOTE: Cette fonction est conservée pour usage futur
    #[allow(dead_code)]
    pub fn load_all_pairs() -> Result<Self, String> {
        let mut index = CandleIndex::new();
        let loader = CsvLoader::new();
        
        // Lister toutes les paires disponibles
        let symbols = loader
            .list_available_symbols()
            .map_err(|e| format!("Failed to list symbols: {}", e))?;

        // Charger et indexer chaque paire
        for symbol in symbols {
            let candles = loader
                .load_candles(&symbol)
                .map_err(|e| format!("Failed to load candles for {}: {}", symbol, e))?;

            if !candles.is_empty() {
                index.add_candles(&symbol, candles);
            }
        }

        Ok(index)
    }

    /// Charge une paire spécifique à la demande (lazy loading)
    /// Retourne true si la paire a été chargée, false si elle l'était déjà
    pub fn load_pair_candles(&mut self, symbol: &str) -> Result<bool, String> {
        // Vérifier si déjà chargée
        if self.data.contains_key(symbol) {
            return Ok(false); // Déjà en cache
        }

        let loader = CsvLoader::new();
        let candles = loader
            .load_candles(symbol)
            .map_err(|e| format!("Failed to load candles for {}: {}", symbol, e))?;

        if !candles.is_empty() {
            self.add_candles(symbol, candles);
            Ok(true) // Nouvelle paire chargée
        } else {
            Err(format!("No candles found for symbol: {}", symbol))
        }
    }

    /// Ajoute des candles indexées par pair et date
    fn add_candles(&mut self, symbol: &str, candles: Vec<Candle>) {
        let mut date_map = BTreeMap::new();

        for candle in candles {
            let date = candle.datetime.date_naive();
            date_map
                .entry(date)
                .or_insert_with(Vec::new)
                .push(candle);
        }

        self.data.insert(symbol.to_string(), date_map);
    }

    /// Récupère ALL candles pour une paire
    /// NOTE: Cette fonction est conservée pour usage futur
    #[allow(dead_code)]
    pub fn get_all_candles(&self, symbol: &str) -> Option<Vec<Candle>> {
        self.data.get(symbol).map(|date_map| {
            date_map
                .values()
                .flat_map(|v| v.clone())
                .collect()
        })
    }

    /// Récupère les candles DANS UNE PLAGE DE DATES (optimisé O(log n))
    /// Important: retourne les candles comme Vec<(DateTime, high, low)> pour compatibilité
    pub fn get_candles_in_range(
        &self,
        symbol: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Option<Vec<(DateTime<Utc>, f64, f64)>> {
        self.data.get(symbol).map(|date_map| {
            date_map
                .range(start_date..=end_date)
                .flat_map(|(_, candles)| candles.iter())
                .map(|c| (c.datetime, c.high, c.low))
                .collect()
        })
    }

    /// Récupère les candles pour UN JOUR SPÉCIFIQUE
    /// NOTE: Cette fonction est conservée pour usage futur
    #[allow(dead_code)]
    pub fn get_candles_for_date(
        &self,
        symbol: &str,
        date: NaiveDate,
    ) -> Option<Vec<(DateTime<Utc>, f64, f64)>> {
        self.data
            .get(symbol)
            .and_then(|date_map| date_map.get(&date))
            .map(|candles| {
                candles
                    .iter()
                    .map(|c| (c.datetime, c.high, c.low))
                    .collect()
            })
    }

    /// Récupère les candles AVANT une datetime donnée (pour baseline)
    /// Returns candles avec same heure, N jours précédents, excluant event_date
    pub fn get_baseline_candles(
        &self,
        symbol: &str,
        event_dt: DateTime<Utc>,
        baseline_days_back: i64,
    ) -> Option<Vec<(DateTime<Utc>, f64, f64)>> {
        let event_date = event_dt.date_naive();
        let baseline_start = event_date - chrono::Duration::days(baseline_days_back);
        let event_hour = event_dt.hour();

        self.data.get(symbol).map(|date_map| {
            date_map
                .range(baseline_start..event_date)
                .flat_map(|(_, candles)| candles.iter())
                .filter(|c| c.datetime.hour() == event_hour && c.datetime.date_naive() != event_date)
                .map(|c| (c.datetime, c.high, c.low))
                .collect()
        })
    }

    /// Retourne la liste des paires chargées
    pub fn get_available_pairs(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Stats pour debugging
    pub fn get_stats(&self) -> HashMap<String, usize> {
        self.data
            .iter()
            .map(|(pair, date_map)| {
                let total_candles = date_map.values().map(|v| v.len()).sum();
                (pair.clone(), total_candles)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_creation() {
        let index = CandleIndex::new();
        assert!(index.get_available_pairs().is_empty());
    }
}
