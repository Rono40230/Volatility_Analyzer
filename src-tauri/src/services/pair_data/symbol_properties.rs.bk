// services/pair_data/symbol_properties.rs
// Gestion des propriétés des symboles (valeur du point, pip, etc.)

/// Retourne la valeur d'un point (Tick Size) pour un symbole donné
/// C'est la plus petite variation de prix possible (ou l'unité de base pour les calculs)
pub fn get_point_value(symbol: &str) -> f64 {
    let s = symbol.to_uppercase();
    
    // JPY pairs (3 décimales): 1 point = 0.001
    if s.contains("JPY") {
        return 0.001;
    }
    
    // Commodités or (XAUUSD, XAUJPY): 2 décimales -> 1 point = 0.01
    if s.contains("XAU") {
        return 0.01;
    }
    
    // Commodités argent (XAGUSD): 3 décimales -> 1 point = 0.001
    if s.contains("XAG") {
        return 0.001;
    }
    
    // Indices (US30, DE30, NAS100, SPX500): 1 point = 1.0
    // Note: Certains courtiers ont des décimales, mais on standardise souvent à 1.0 ou 0.1
    if s.contains("US30") 
        || s.contains("DE30")
        || s.contains("NAS100")
        || s.contains("SPX500")
        || s.contains("USA500")
        || s.contains("GER30")
        || s.contains("FRA40")
        || s.contains("UK100")
    {
        return 1.0;
    }
    
    // Crypto (BTCUSD, ETHUSD): 1 point = 1.0 (souvent)
    // Pour les petites cryptos, ça peut être moins, mais pour les majeures c'est souvent 1 ou 0.01
    // On assume 1.0 pour BTC/ETH pour l'instant
    if s.contains("BTC") || s.contains("ETH") {
        return 1.0;
    }
    
    // Forex standard 5 décimales (EURUSD, GBPUSD, etc.): 1 point = 0.00001
    // C'est le "point" standard MetaTrader
    0.00001
}

/// Retourne la valeur d'un Pip standard
/// - Forex: 10 points
/// - Indices/Crypto: 1 point (souvent)
#[allow(dead_code)]
pub fn get_pip_value(symbol: &str) -> f64 {
    let point = get_point_value(symbol);
    let s = symbol.to_uppercase();
    
    // Forex (JPY ou Standard) : 1 pip = 10 points
    // Ex: EURUSD point=0.00001, pip=0.0001
    // Ex: USDJPY point=0.001, pip=0.01
    if is_forex_pair(&s) {
        return point * 10.0;
    }
    
    // Pour le reste (Indices, Crypto, Gold), souvent 1 pip = 1 point ou 10 points selon convention
    // On garde 1 pip = 1 point pour simplifier sauf si spécifié autrement
    if s.contains("XAU") {
        return point * 10.0; // Gold: 1 pip = 0.1 (10 ticks de 0.01)
    }
    
    point
}

/// Détermine si c'est une paire Forex
#[allow(dead_code)]
fn is_forex_pair(symbol: &str) -> bool {
    let s = symbol.to_uppercase();
    // Liste non exhaustive mais couvre les majeurs
    let currencies = ["EUR", "USD", "GBP", "JPY", "CAD", "AUD", "NZD", "CHF"];
    
    // Si contient 2 devises de la liste
    let count = currencies.iter().filter(|c| s.contains(*c)).count();
    count >= 2
}
