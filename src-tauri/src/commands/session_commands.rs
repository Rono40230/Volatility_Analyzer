use crate::commands::calendar_commands::CalendarState;
use crate::services::session_analyzer::{
    OverlapStats, SessionAnalysisResult, SessionAnalyzer,
    SessionStats, TradingSession,
};
use crate::services::{CsvLoader, CalendarCorrelator};
use chrono::{NaiveDateTime, Timelike};
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn analyze_sessions(
    pair_symbol: String,
    _state: State<'_, CalendarState>,
) -> Result<SessionAnalysisResult, String> {
    // Charger les données depuis le fichier CSV au lieu de la DB
    let loader = CsvLoader::new();
    let all_candles = loader.load_candles(&pair_symbol)
        .map_err(|e| format!("Erreur lors du chargement des données: {}", e))?;

    if all_candles.is_empty() {
        return Err(format!(
            "Aucune donnée trouvée pour {}. Vérifiez que le fichier CSV existe dans data/csv/",
            pair_symbol
        ));
    }

    // Convertir les bougies en format (timestamp, high, low)
    let candles: Vec<(i64, f64, f64)> = all_candles.iter()
        .map(|c| (c.datetime.timestamp(), c.high, c.low))
        .collect();

    // Analyser par session
    let sessions = SessionAnalyzer::get_sessions();
    let mut session_volatilities: HashMap<String, Vec<f64>> = HashMap::new();

    for session in &sessions {
        session_volatilities.insert(session.name.clone(), Vec::new());
    }

    let mut total_volatility = 0.0;
    let mut first_date: Option<NaiveDateTime> = None;
    let mut last_date: Option<NaiveDateTime> = None;

    // Parcourir toutes les bougies
    for (ts, h, l) in &candles {
        let datetime = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|dt| dt.naive_utc())
            .ok_or_else(|| "Timestamp invalide".to_string())?;

        if first_date.is_none() {
            first_date = Some(datetime);
        }
        last_date = Some(datetime);

        let hour = datetime.hour();
        let volatility = h - l;
        total_volatility += volatility;

        // Assigner à la ou les sessions correspondantes
        for session in &sessions {
            if SessionAnalyzer::is_in_session(hour, session) {
                session_volatilities
                    .get_mut(&session.name)
                    .ok_or_else(|| "Session not found in volatilities map".to_string())?
                    .push(volatility);
            }
        }
    }

    // Calculer les statistiques par session
    let total_candles = candles.len();
    let avg_daily_volatility = total_volatility / (total_candles as f64 / 24.0);

    let mut session_stats: Vec<SessionStats> = Vec::new();

    // On utilise l'heure d'hiver par défaut pour l'affichage
    let is_winter = true;

    for session in &sessions {
        let vols = session_volatilities.get(&session.name)
            .ok_or_else(|| "Session not found in volatilities map".to_string())?;
        if vols.is_empty() {
            continue;
        }

        let avg_vol: f64 = vols.iter().sum::<f64>() / vols.len() as f64;
        let percentage = (vols.len() as f64 / total_candles as f64) * 100.0;

        session_stats.push(SessionStats {
            name: session.name.clone(),
            icon: session.icon.clone(),
            paris_hours: SessionAnalyzer::format_paris_hours(session, is_winter),
            avg_volatility: (avg_vol * 10000.0 * 100.0).round() / 100.0, // Convertir en pips avec 2 décimales
            percentage: (percentage * 100.0).round() / 100.0,
            candle_count: vols.len(),
        });
    }

    // Trier par volatilité décroissante
    session_stats.sort_by(|a, b| {
        b.avg_volatility
            .partial_cmp(&a.avg_volatility)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Calculer les chevauchements
    let overlaps = calculate_overlaps(&sessions, &candles, is_winter, avg_daily_volatility)?;

    // Corrélation avec calendrier (vraies données DB)
    let pool_guard = _state.pool.lock()
        .map_err(|e| format!("Erreur lock pool: {}", e))?;
    let pool = pool_guard.as_ref()
        .ok_or("Pool DB non initialisé")?;
    let calendar_correlation = CalendarCorrelator::calculate_correlation(&sessions, pool)?;

    // Générer les recommandations
    let recommendations =
        SessionAnalyzer::generate_recommendations(&session_stats, avg_daily_volatility);

    // Formater la période
    let period = if let (Some(first), Some(last)) = (first_date, last_date) {
        format!(
            "{} → {}",
            first.format("%Y-%m-%d"),
            last.format("%Y-%m-%d")
        )
    } else {
        "N/A".to_string()
    };

    Ok(SessionAnalysisResult {
        period,
        total_candles,
        avg_daily_volatility: (avg_daily_volatility * 10000.0 * 100.0).round() / 100.0,
        sessions: session_stats,
        overlaps,
        calendar_correlation,
        recommendations,
    })
}

fn calculate_overlaps(
    _sessions: &[TradingSession],
    candles: &[(i64, f64, f64)],
    is_winter: bool,
    avg_daily_vol: f64,
) -> Result<Vec<OverlapStats>, String> {
    let mut overlaps = Vec::new();

    // Définir les chevauchements connus
    let overlap_pairs = vec![
        ("Tokyo", "Londres", 8, 9),    // Tokyo+Londres: 8-9 UTC
        ("Londres", "New York", 13, 17), // Londres+NY: 13-17 UTC
    ];

    for (sess1_name, sess2_name, start_hour, end_hour) in overlap_pairs {
        let mut overlap_vols = Vec::new();

        for (ts, h, l) in candles {
            let datetime = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|dt| dt.naive_utc())
                .ok_or_else(|| "Timestamp invalide".to_string())?;

            let hour = datetime.hour();

            if hour >= start_hour && hour < end_hour {
                overlap_vols.push(h - l);
            }
        }

        if !overlap_vols.is_empty() {
            let avg_vol: f64 = overlap_vols.iter().sum::<f64>() / overlap_vols.len() as f64;
            let avg_vol_pips = (avg_vol * 10000.0 * 100.0).round() / 100.0;
            let multiplier = (avg_vol / (avg_daily_vol / 24.0) * 10.0).round() / 10.0;

            let offset = if is_winter { 1 } else { 2 };
            let paris_start = (start_hour + offset) % 24;
            let paris_end = (end_hour + offset) % 24;

            overlaps.push(OverlapStats {
                name: format!("{} + {}", sess1_name, sess2_name),
                paris_hours: format!("{:02}h00-{:02}h00", paris_start, paris_end),
                avg_volatility: avg_vol_pips,
                volatility_multiplier: multiplier,
            });
        }
    }

    Ok(overlaps)
}
