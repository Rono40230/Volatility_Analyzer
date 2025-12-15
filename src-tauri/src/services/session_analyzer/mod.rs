mod helpers;

pub use helpers::{format_paris_hours, is_in_session};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub name: String,
    pub icon: String,
    pub paris_hours: String,
    pub avg_volatility: f64,
    pub percentage: f64,
    pub candle_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlapStats {
    pub name: String,
    pub paris_hours: String,
    pub avg_volatility: f64,
    pub volatility_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarCorrelation {
    pub session: String,
    pub high_impact_events: usize,
    pub event_volatility: f64,
    pub impact_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub icon: String,
    #[serde(rename = "type")]
    pub rec_type: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalysisResult {
    pub period: String,
    pub total_candles: usize,
    pub avg_daily_volatility: f64,
    pub sessions: Vec<SessionStats>,
    pub overlaps: Vec<OverlapStats>,
    pub calendar_correlation: Vec<CalendarCorrelation>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone)]
pub struct TradingSession {
    pub name: String,
    pub icon: String,
    pub utc_start_hour: u32,
    pub utc_end_hour: u32,
}

pub struct SessionAnalyzer;

impl SessionAnalyzer {
    /// Définit les 4 sessions Forex en UTC
    pub fn obtenir_sessions() -> Vec<TradingSession> {
        vec![
            TradingSession {
                name: "Sydney".to_string(),
                icon: "🇦🇺".to_string(),
                utc_start_hour: 22,
                utc_end_hour: 7,
            },
            TradingSession {
                name: "Tokyo".to_string(),
                icon: "🇯🇵".to_string(),
                utc_start_hour: 0,
                utc_end_hour: 9,
            },
            TradingSession {
                name: "Londres".to_string(),
                icon: "🇬🇧".to_string(),
                utc_start_hour: 8,
                utc_end_hour: 17,
            },
            TradingSession {
                name: "New York".to_string(),
                icon: "🇺🇸".to_string(),
                utc_start_hour: 13,
                utc_end_hour: 22,
            },
        ]
    }

    /// Formate les horaires Paris d'une session
    pub fn formater_heures_paris(session: &TradingSession, is_winter: bool) -> String {
        format_paris_hours(session.utc_start_hour, session.utc_end_hour, is_winter)
    }

    /// Détermine si une heure UTC appartient à une session donnée
    pub fn est_dans_session(hour: u32, session: &TradingSession) -> bool {
        is_in_session(hour, session.utc_start_hour, session.utc_end_hour)
    }

    /// Génère des recommandations basées sur les statistiques
    pub fn generer_recommandations(
        sessions: &[SessionStats],
        avg_daily_vol: f64,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        if let Some(best_session) = sessions.first() {
            let multiplier = best_session.avg_volatility / (avg_daily_vol / 4.0);
            recommendations.push(Recommendation {
                icon: "✅".to_string(),
                rec_type: "positive".to_string(),
                title: format!(
                    "Session {} recommandée pour le scalping",
                    best_session.name
                ),
                description: format!(
                    "Historiquement, {} présente la volatilité la plus élevée ({:.1} pips en moyenne). \
                    Volatilité {:.1}x supérieure à la moyenne par session. \
                    Horaires Paris : {}",
                    best_session.name,
                    best_session.avg_volatility,
                    multiplier,
                    best_session.paris_hours
                ),
            });
        }

        if let Some(worst_session) = sessions.last() {
            recommendations.push(Recommendation {
                icon: "⚠️".to_string(),
                rec_type: "warning".to_string(),
                title: format!("Éviter la session {}", worst_session.name),
                description: format!(
                    "La session {} présente la volatilité la plus faible ({:.1} pips). \
                    Représente seulement {:.1}% de la volatilité totale. \
                    Moins d'opportunités de scalping pendant cette période ({})",
                    worst_session.name,
                    worst_session.avg_volatility,
                    worst_session.percentage,
                    worst_session.paris_hours
                ),
            });
        }

        recommendations.push(Recommendation {
            icon: "💡".to_string(),
            rec_type: "info".to_string(),
            title: "Zones de chevauchement = Opportunités maximales".to_string(),
            description: "Les périodes de chevauchement entre deux sessions (ex: Londres + New York) \
                génèrent historiquement la plus forte volatilité. Ces zones sont idéales pour le scalping \
                mais nécessitent une gestion du risque accrue."
                .to_string(),
        });

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sessions() {
        let sessions = SessionAnalyzer::get_sessions();
        assert_eq!(sessions.len(), 4);
        assert_eq!(sessions[0].name, "Sydney");
        assert_eq!(sessions[1].name, "Tokyo");
        assert_eq!(sessions[2].name, "Londres");
        assert_eq!(sessions[3].name, "New York");
    }

    #[test]
    fn test_format_paris_hours() {
        let sydney = &SessionAnalyzer::get_sessions()[0];
        let hours = SessionAnalyzer::format_paris_hours(sydney, false);
        assert!(!hours.is_empty());
        assert!(hours.contains("h"));
    }

    #[test]
    fn test_is_in_session() {
        let london = &SessionAnalyzer::get_sessions()[2];
        assert!(SessionAnalyzer::is_in_session(10, london)); // 8-17 UTC
        assert!(!SessionAnalyzer::is_in_session(20, london));
    }

    #[test]
    fn test_generate_recommendations_normal() {
        let sessions = vec![
            SessionStats {
                name: "Best".to_string(),
                icon: "🔝".to_string(),
                paris_hours: "10h00-17h00".to_string(),
                avg_volatility: 100.0,
                percentage: 40.0,
                candle_count: 1000,
            },
            SessionStats {
                name: "Worst".to_string(),
                icon: "🔻".to_string(),
                paris_hours: "22h00-07h00".to_string(),
                avg_volatility: 50.0,
                percentage: 20.0,
                candle_count: 500,
            },
        ];
        let recommendations = SessionAnalyzer::generate_recommendations(&sessions, 75.0);
        assert!(recommendations.len() >= 3);
        assert!(recommendations[0].title.contains("recommandée"));
    }

    #[test]
    fn test_generate_recommendations_empty() {
        let recommendations = SessionAnalyzer::generate_recommendations(&[], 75.0);
        assert!(recommendations.len() > 0); // Au moins la recommandation info
    }

    #[test]
    fn test_recommendation_structure() {
        let sessions = vec![SessionStats {
            name: "Test".to_string(),
            icon: "✅".to_string(),
            paris_hours: "10h00-17h00".to_string(),
            avg_volatility: 100.0,
            percentage: 50.0,
            candle_count: 1000,
        }];
        let recs = SessionAnalyzer::generate_recommendations(&sessions, 75.0);
        for rec in recs {
            assert!(!rec.icon.is_empty());
            assert!(!rec.title.is_empty());
            assert!(!rec.description.is_empty());
        }
    }
}
