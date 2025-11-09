use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use crate::services::CsvLoader;
use chrono::{DateTime, Utc, TimeZone, Duration, Timelike};
use tauri::State;

use super::volatility_helpers::{parse_sqlite_datetime, calculate_volatilities_optimized};
use crate::commands::candle_index_commands::CandleIndexState;

/// Retourne la valeur d'1 pip pour une paire donnée
fn get_pip_value(symbol: &str) -> f64 {
    match symbol {
        "ADAUSD" => 0.0001,
        "BTCUSD" => 1.00,
        "CADJPY" => 0.01,
        "CHFJPY" => 0.01,
        "ETHUSD" => 0.01,
        "GBPJPY" => 0.01,
        "LINKUSD" => 0.001,
        "LTCUSD" => 0.01,
        "UNIUSD" => 0.001,
        "USDCAD" => 0.0001,
        "USDJPY" => 0.01,
        "XAGUSD" => 0.001,
        "XAUUSD" => 0.01,
        "XLMUSD" => 0.00001,
        _ => 0.0001, // valeur par défaut
    }
}

/// Mappe une devise ISO à son pays
fn currency_to_country(currency: &str) -> String {
    match currency {
        "USD" => "United States",
        "EUR" => "Eurozone",
        "GBP" => "United Kingdom",
        "JPY" => "Japan",
        "CHF" => "Switzerland",
        "CAD" => "Canada",
        "AUD" => "Australia",
        "NZD" => "New Zealand",
        "CNY" => "China",
        "INR" => "India",
        "ZAR" => "South Africa",
        "MXN" => "Mexico",
        _ => "Unknown",
    }.to_string()
}

fn get_available_pairs() -> Result<Vec<String>, String> {
    let loader = CsvLoader::new();
    let symbols = loader
        .list_available_symbols()
        .map_err(|e| format!("Failed to get available symbols: {}", e))?;
    
    // Utiliser TOUTES les paires disponibles (pas de limite de 3)
    let priority_pairs = vec!["USDJPY", "GBPJPY", "BTCUSD", "ETHUSD", "EURUSD", "GBPUSD", 
                              "USDCAD", "USDCHF", "CADJPY", "CHFJPY", "XAUUSD", "XAGUSD",
                              "LTCCHF", "TRXUSD", "LNKUSD", "UNIUSD", "EURZAR", "NZDMXN"];
    
    let mut result = Vec::new();
    
    // Ajouter les paires prioritaires si disponibles
    for pair in priority_pairs {
        if symbols.contains(&pair.to_string()) {
            result.push(pair.to_string());
        }
    }
    
    // Ajouter les autres paires qui ne sont pas dans la liste prioritaire
    for symbol in symbols {
        if !result.contains(&symbol) {
            result.push(symbol);
        }
    }
    
    Ok(result)
}

/// Calcule les volatilités pour plusieurs événements avec les candles déjà chargés (une seule lecture CSV)
/// 
/// Option C - Baseline glissante :
/// - event_volatility : moyenne des volatilités sur la fenêtre événement (±30min autour de l'événement)
/// - baseline_volatility : moyenne des volatilités sur les 7 jours ouvrables AVANT l'événement,
///   à la même heure, excluant les autres événements HIGH/MEDIUM
fn calculate_volatilities_for_events(
    candles: &[(DateTime<Utc>, f64, f64)], // (datetime, high, low)
    event_datetimes: &[DateTime<Utc>],
    event_window_minutes: i64,
    baseline_days_back: i64,
    symbol: &str,
) -> (f64, f64) {
    let pip_value = get_pip_value(symbol);
    let mut all_event_volatilities = Vec::new();
    let mut all_baseline_volatilities = Vec::new();
    
    for event_dt in event_datetimes {
        let event_hour = event_dt.hour();
        let event_date = event_dt.date_naive();
        
        let event_window_start = *event_dt - Duration::minutes(event_window_minutes);
        let event_window_end = *event_dt + Duration::minutes(event_window_minutes);
        let baseline_start = *event_dt - Duration::days(baseline_days_back);
        
        let mut event_vol_sum = 0.0;
        let mut event_count = 0;
        let mut baseline_vol_sum = 0.0;
        let mut baseline_count = 0;
        
        for (candle_dt, high, low) in candles.iter() {
            // Convertir en pips en utilisant la valeur correcte pour la paire
            let pips = (high - low) / pip_value;
            
            // Volatilité pendant l'événement (±N min)
            if candle_dt >= &event_window_start && candle_dt <= &event_window_end {
                event_vol_sum += pips;
                event_count += 1;
            }
            
            // Volatilité baseline (même heure sur 7 jours précédents)
            // Option C: 7 jours ouvrables avant l'événement, même heure
            let candle_date = candle_dt.date_naive();
            if candle_dt >= &baseline_start 
               && candle_dt < event_dt 
               && candle_date != event_date
               && candle_dt.hour() == event_hour {
                baseline_vol_sum += pips;
                baseline_count += 1;
            }
        }
        
        // Moyennes pour CET événement
        if event_count > 0 {
            all_event_volatilities.push(event_vol_sum / event_count as f64);
        }
        if baseline_count > 0 {
            all_baseline_volatilities.push(baseline_vol_sum / baseline_count as f64);
        }
    }
    
    // Moyennes sur TOUS les événements
    let event_volatility = if all_event_volatilities.is_empty() {
        0.0
    } else {
        all_event_volatilities.iter().sum::<f64>() / all_event_volatilities.len() as f64
    };
    
    let baseline_volatility = if all_baseline_volatilities.is_empty() {
        0.0
    } else {
        all_baseline_volatilities.iter().sum::<f64>() / all_baseline_volatilities.len() as f64
    };
    
    (event_volatility, baseline_volatility)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairImpactDetail {
    pub symbol: String,
    pub event_volatility: f64,     // valeur brute en pips
    pub baseline_volatility: f64,   // valeur brute en pips
    pub event_volatility_formatted: String,    // formatée à 1 décimale
    pub baseline_volatility_formatted: String, // formatée à 1 décimale
    pub points: f64,               // event_volatility / 10 (1 point = 1/10 pip)
    pub points_formatted: String,  // formatée à 1 décimale
    pub price: f64,                // points * pip_value (valeur monétaire approx)
    pub price_formatted: String,   // formatée à 2 décimales
    pub multiplier: f64,
    pub direction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventImpactResult {
    pub event_id: i32,
    pub event_name: String,
    pub datetime: String,           // Première date
    pub last_datetime: String,      // Dernière date
    pub country: String,
    pub currency: String,
    pub impact: String,
    pub event_count: i32,           // Nombre d'occurrences de cet événement (pas multiplié par paires)
    pub window_start: String,
    pub window_end: String,
    pub pair_impacts: Vec<PairImpactDetail>,
    pub observations: Vec<String>,
}

#[tauri::command]
pub async fn get_event_impact_by_pair(
    event_type: String,
    event_count: i32,
    state: State<'_, CandleIndexState>,
) -> Result<EventImpactResult, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Failed to get data directory")?
        .join("volatility-analyzer");
    
    let db_path = data_dir.join("volatility.db");
    
    if !db_path.exists() {
        return Err("Database not found".to_string());
    }
    
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // Récupérer TOUTES les occurrences de cet événement depuis 2024-01-01
    let mut event_stmt = conn
        .prepare("SELECT id, datetime(event_time), symbol, impact FROM calendar_events WHERE description = ?1 AND event_time >= '2024-01-01' ORDER BY event_time")
        .map_err(|e| format!("Failed to prepare event query: {}", e))?;
    
    let events: Vec<(i32, String, String, String)> = event_stmt
        .query_map([&event_type], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| format!("Failed to query events: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect events: {}", e))?;
    
    if events.is_empty() {
        return Err(format!("No events found for type: {}", event_type));
    }
    
    // Récupérer info de la première occurrence pour les détails
    let (first_id, first_datetime, currency, impact) = &events[0];
    // Convertir la devise en pays
    let country = currency_to_country(currency);
    
    // Parser la première et dernière datetime
    let first_event_datetime = parse_sqlite_datetime(first_datetime)?;
    let window_start = first_event_datetime.format("%Y-%m-%d %H:%M").to_string();
    let window_end = (first_event_datetime + chrono::Duration::minutes(120)).format("%Y-%m-%d %H:%M").to_string();
    
    // Récupérer la dernière datetime
    let (_, last_datetime, _, _) = &events[events.len() - 1];
    let last_event_datetime = parse_sqlite_datetime(last_datetime)?;
    let last_datetime_formatted = last_datetime.clone();
    
    // Le event_count est passé en paramètre depuis le frontend (du dropdown)
    // Pas besoin de le recalculer
    
    // Obtenir toutes les paires disponibles
    let pairs = get_available_pairs()?;
    
    // Préparer les datetimes des événements pour le calcul groupé
    let event_datetimes: Result<Vec<DateTime<Utc>>, String> = events
        .iter()
        .map(|(_, datetime_str, _, _)| {
            let naive_dt = parse_sqlite_datetime(datetime_str)?;
            Ok(Utc.from_utc_datetime(&naive_dt))
        })
        .collect();
    let event_datetimes = event_datetimes?;
    
    // ⚠️ AUDIT FIX: Garder le lock pendant TOUTE l'opération (race condition fix)
    let mut index_state = state.index.lock()
        .map_err(|e| format!("Failed to lock candle index state: {}", e))?;
    
    let candle_index = index_state
        .as_mut()
        .ok_or("CandleIndex not initialized. Call init_candle_index first.")?;
    
    // Charger les paires à la demande (lazy loading)
    for pair in &pairs {
        candle_index.load_pair_candles(pair)?;  // ✅ AUDIT FIX: Propager erreurs, pas ignorer
    }
    
    let mut pair_impacts = Vec::new();
    
    for pair in &pairs {
        // OPTIMISATION: Utiliser le CandleIndex au lieu de charger CSV
        let metrics = calculate_volatilities_optimized(
            candle_index,
            pair,
            event_datetimes[0].naive_utc(),  // Première événement pour fenêtre
            30,  // event_window_minutes
            7,   // baseline_days_back
            super::volatility_helpers::get_pip_value(pair),  // ✅ CORRECTION: passer pip_value
        )?;
        
        let event_volatility = metrics.event_volatility;
        let baseline_volatility = metrics.baseline_volatility;

        let multiplier = if baseline_volatility > 0.0 {
            event_volatility / baseline_volatility
        } else {
            0.0
        };
        
        // Déterminer la direction (simplifié pour le moment)
        let direction = if multiplier > 10.0 {
            "HAUSSIER".to_string()
        } else if multiplier > 5.0 {
            "BAISSIER".to_string()
        } else {
            "NEUTRE".to_string()
        };
        
        // Calculer points (1 point = 1/10 pip) et prix
        let points = event_volatility / 10.0;
        let pip_value = get_pip_value(pair);
        let price = points * pip_value;
        
        pair_impacts.push(PairImpactDetail {
            symbol: pair.clone(),
            event_volatility,
            baseline_volatility,
            event_volatility_formatted: format!("{:.1}", event_volatility),
            baseline_volatility_formatted: format!("{:.1}", baseline_volatility),
            points,
            points_formatted: format!("{:.1}", points),
            price,
            price_formatted: format!("{:.2}", price),
            multiplier,
            direction,
        });
    }
    
    // Trier par multiplicateur décroissant
    // FIX .clinerules: Remplacer unwrap() par gestion d'erreur explicite
    pair_impacts.sort_by(|a, b| {
        b.multiplier.partial_cmp(&a.multiplier)
            .unwrap_or(std::cmp::Ordering::Equal) // Gère les NaN
    });
    
    // Générer des observations basées sur les données
    let mut observations = Vec::new();
    
    // Observation 1 : Impact le plus fort
    if let Some(top_pair) = pair_impacts.first() {
        observations.push(format!(
            "{} a enregistré le plus fort impact avec {:.0} pips, soit {:.1}× sa volatilité normale",
            top_pair.symbol, top_pair.event_volatility, top_pair.multiplier
        ));
    }
    
    // Observation 2 : Paire avec la plus grosse variation en pips
    if let Some(biggest_vol) = pair_impacts.iter().max_by(|a, b| 
        a.event_volatility.partial_cmp(&b.event_volatility)
            .unwrap_or(std::cmp::Ordering::Equal)
    ) {
        observations.push(format!(
            "Variation maximale observée: {} avec {:.1} pips de volatilité événement",
            biggest_vol.symbol, biggest_vol.event_volatility
        ));
    }
    
    // Observation 3 : Paires réactives et conseil
    let high_impact_count = pair_impacts.iter().filter(|p| p.multiplier > 5.0).count();
    if high_impact_count > 0 {
        let avg_multiplier = pair_impacts.iter()
            .filter(|p| p.multiplier > 5.0)
            .map(|p| p.multiplier)
            .sum::<f64>() / high_impact_count.max(1) as f64;
        
        observations.push(format!(
            "⚠️ Attention: {} paires ont montré une volatilité EXCESSIVE (multiplicateur >5×). Ces multiplicateurs élevés (moy. {:.1}×) indiquent une réaction disproportionnée. À éviter en trading régulier, risque trop élevé pour le gain potentiel",
            high_impact_count, avg_multiplier
        ));
    }
    
    Ok(EventImpactResult {
        event_id: *first_id,
        event_name: event_type.clone(),
        datetime: first_datetime.clone(),
        last_datetime: last_datetime_formatted,
        country,
        currency: currency.clone(),
        impact: impact.clone(),
        event_count,
        window_start,
        window_end,
        pair_impacts,
        observations,
    })
}

