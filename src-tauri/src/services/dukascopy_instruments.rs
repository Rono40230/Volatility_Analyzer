// services/dukascopy_instruments.rs
// Liste des instruments Dukascopy supportés (CDN public).

use serde::Serialize;

/// Instrument disponible sur le CDN Dukascopy
#[derive(Debug, Clone, Serialize)]
pub struct DukascopyInstrument {
    /// Identifiant CDN (ex: "EURUSD")
    pub id: &'static str,
    /// Nom affiché (ex: "EUR/USD")
    pub display: &'static str,
    /// Catégorie (ex: "Forex Major")
    pub category: &'static str,
    /// Diviseur pour décoder les prix bi5 (ex: 100_000.0 pour 5 décimales)
    pub point_value: f64,
    /// Valeur d'un pip (ex: 0.0001)
    pub pip_value: f64,
}

/// Retourne la liste complète des instruments supportés.
pub fn get_instruments() -> Vec<DukascopyInstrument> {
    vec![
        // ─── Forex Majeurs ───
        DukascopyInstrument { id: "EURUSD", display: "EUR/USD", category: "Forex Major", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "GBPUSD", display: "GBP/USD", category: "Forex Major", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "USDJPY", display: "USD/JPY", category: "Forex Major", point_value: 1_000.0, pip_value: 0.01 },
        DukascopyInstrument { id: "USDCHF", display: "USD/CHF", category: "Forex Major", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "AUDUSD", display: "AUD/USD", category: "Forex Major", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "NZDUSD", display: "NZD/USD", category: "Forex Major", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "USDCAD", display: "USD/CAD", category: "Forex Major", point_value: 100_000.0, pip_value: 0.0001 },
        // ─── Forex Cross ───
        DukascopyInstrument { id: "EURGBP", display: "EUR/GBP", category: "Forex Cross", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "EURJPY", display: "EUR/JPY", category: "Forex Cross", point_value: 1_000.0, pip_value: 0.01 },
        DukascopyInstrument { id: "GBPJPY", display: "GBP/JPY", category: "Forex Cross", point_value: 1_000.0, pip_value: 0.01 },
        DukascopyInstrument { id: "EURCHF", display: "EUR/CHF", category: "Forex Cross", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "AUDJPY", display: "AUD/JPY", category: "Forex Cross", point_value: 1_000.0, pip_value: 0.01 },
        DukascopyInstrument { id: "CHFJPY", display: "CHF/JPY", category: "Forex Cross", point_value: 1_000.0, pip_value: 0.01 },
        DukascopyInstrument { id: "GBPCHF", display: "GBP/CHF", category: "Forex Cross", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "EURAUD", display: "EUR/AUD", category: "Forex Cross", point_value: 100_000.0, pip_value: 0.0001 },
        DukascopyInstrument { id: "GBPAUD", display: "GBP/AUD", category: "Forex Cross", point_value: 100_000.0, pip_value: 0.0001 },
        // ─── Métaux ───
        DukascopyInstrument { id: "XAUUSD", display: "XAU/USD (Or)", category: "Métal", point_value: 1_000.0, pip_value: 0.01 },
        DukascopyInstrument { id: "XAGUSD", display: "XAG/USD (Argent)", category: "Métal", point_value: 100_000.0, pip_value: 0.0001 },
        // ─── Crypto ───
        DukascopyInstrument { id: "BTCUSD", display: "BTC/USD", category: "Crypto", point_value: 100.0, pip_value: 0.01 },
        DukascopyInstrument { id: "ETHUSD", display: "ETH/USD", category: "Crypto", point_value: 100.0, pip_value: 0.01 },
        DukascopyInstrument { id: "LTCUSD", display: "LTC/USD", category: "Crypto", point_value: 100.0, pip_value: 0.01 },
        DukascopyInstrument { id: "XRPUSD", display: "XRP/USD", category: "Crypto", point_value: 100_000.0, pip_value: 0.00001 },
        DukascopyInstrument { id: "SOLUSD", display: "SOL/USD", category: "Crypto", point_value: 100.0, pip_value: 0.01 },
    ]
}

/// Recherche un instrument par ID (insensible à la casse).
pub fn find_instrument(id: &str) -> Option<DukascopyInstrument> {
    get_instruments()
        .into_iter()
        .find(|i| i.id.eq_ignore_ascii_case(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_instrument_eurusd() {
        let inst = find_instrument("EURUSD").expect("EURUSD devrait exister");
        assert_eq!(inst.id, "EURUSD");
        assert!((inst.point_value - 100_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_instrument_case_insensitive() {
        assert!(find_instrument("eurusd").is_some());
        assert!(find_instrument("EurUsd").is_some());
    }

    #[test]
    fn test_find_instrument_unknown() {
        assert!(find_instrument("ZZZZZZ").is_none());
    }

    #[test]
    fn test_jpy_pairs_have_correct_point_value() {
        for id in &["USDJPY", "EURJPY", "GBPJPY", "AUDJPY", "CHFJPY"] {
            let inst = find_instrument(id).unwrap_or_else(|| panic!("{} devrait exister", id));
            assert!((inst.point_value - 1_000.0).abs() < f64::EPSILON, "{} devrait avoir point_value=1000", id);
            assert!((inst.pip_value - 0.01).abs() < f64::EPSILON, "{} devrait avoir pip_value=0.01", id);
        }
    }

    #[test]
    fn test_instruments_list_not_empty() {
        assert!(get_instruments().len() >= 10);
    }
}
