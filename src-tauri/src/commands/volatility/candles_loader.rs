// commands/volatility/candles_loader.rs - Charger les candles pour une heure spécifique
use serde_json::json;
use tauri::command;

/// Charge les 60 candles (1min) pour une heure donnée et une paire donnée
/// 
/// Format : Les candles sont retournées en JSON pour sérialisation simple
/// Chaque candle contient: { symbol, datetime, open, high, low, close, volume }
/// 
/// TODO: Implémenter la vraie logique de chargement depuis la DB
#[command]
pub async fn load_candles_for_hour(
    symbol: String,
    date: String, // Format: "2025-01-15"
    hour: u8,     // 0-23
) -> Result<Vec<serde_json::Value>, String> {
    // Validation de l'heure
    if hour > 23 {
        return Err(format!("Heure invalide: {} (0-23 requis)", hour));
    }

    // TODO: Remplacer par vraie requête DB
    // Pour l'instant: retourner candles vides (valeurs par défaut)
    // Raison: Le vrai chargement requiert accès à la DB avec query par datetime
    
    println!(
        "📊 load_candles_for_hour: {} {} {}:00",
        symbol, date, hour
    );

    // Placeholder: retourner array vide ou valeurs par défaut
    let candles: Vec<serde_json::Value> = Vec::new();

    Ok(candles)
}

/// Version simplifiée: charger les dernières 60 candles chargées en mémoire
/// (Utilise le state CandleIndex si disponible)
#[command]
pub async fn get_cached_candles_for_hour(
    symbol: String,
    hour: u8,
) -> Result<Vec<serde_json::Value>, String> {
    // Validation
    if hour > 23 {
        return Err(format!("Heure invalide: {}", hour));
    }

    // TODO: Accéder au CandleIndexState pour récupérer les candles cachées
    // Filtrer par heure
    
    println!(
        "🔍 get_cached_candles_for_hour: {} heure {}",
        symbol, hour
    );

    // Placeholder
    let candles: Vec<serde_json::Value> = Vec::new();

    Ok(candles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_hour() {
        let result = load_candles_for_hour("EURUSD".to_string(), "2025-01-15".to_string(), 25).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_valid_hour() {
        let result = load_candles_for_hour("EURUSD".to_string(), "2025-01-15".to_string(), 14).await;
        assert!(result.is_ok());
    }
}
