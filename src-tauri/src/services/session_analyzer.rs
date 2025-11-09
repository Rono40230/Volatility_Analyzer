use chrono::{Datelike, NaiveDateTime};
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
    pub fn get_sessions() -> Vec<TradingSession> {
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

    /// Convertit une heure UTC en heure de Paris (avec gestion hiver/été)
    /// NOTE: Fonction conservée pour usage futur
    #[allow(dead_code)]
    pub fn utc_to_paris(utc_hour: u32, date: &NaiveDateTime) -> u32 {
        let is_dst = Self::is_paris_dst(date);
        let offset = if is_dst { 2 } else { 1 };
        (utc_hour + offset) % 24
    }

    /// Détermine si Paris est en heure d'été (UTC+2) ou d'hiver (UTC+1)
    /// Heure d'été : dernier dimanche de mars à dernier dimanche d'octobre
    /// NOTE: Fonction conservée pour usage futur
    #[allow(dead_code)]
    pub fn is_paris_dst(date: &NaiveDateTime) -> bool {
        let month = date.month();
        let day = date.day();
        let _weekday = date.weekday().num_days_from_sunday();

        // Entre avril et septembre : toujours heure d'été
        if month > 3 && month < 10 {
            return true;
        }

        // Janvier, février, novembre, décembre : toujours heure d'hiver
        if !(3..=10).contains(&month) {
            return false;
        }

        // Mars : heure d'été à partir du dernier dimanche
        if month == 3 {
            let last_sunday = Self::last_sunday_of_month(date.year(), 3);
            return day >= last_sunday;
        }

        // Octobre : heure d'été jusqu'au dernier dimanche
        if month == 10 {
            let last_sunday = Self::last_sunday_of_month(date.year(), 10);
            return day < last_sunday;
        }

        false
    }

    /// Calcule le jour du dernier dimanche d'un mois donné
    /// NOTE: Fonction conservée pour usage futur
    #[allow(dead_code)]
    fn last_sunday_of_month(year: i32, month: u32) -> u32 {
        let day = 31;

        // Cherche le dernier dimanche en reculant depuis la fin du mois
        for d in (1..=day).rev() {
            if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, d)
                .and_then(|d| d.and_hms_opt(12, 0, 0))
            {
                if date.weekday().num_days_from_sunday() == 0 {
                    return d;
                }
            }
        }
        day
    }

    /// Formate les horaires Paris d'une session
    pub fn format_paris_hours(session: &TradingSession, is_winter: bool) -> String {
        let offset = if is_winter { 1 } else { 2 };
        let start = (session.utc_start_hour + offset) % 24;
        let end = (session.utc_end_hour + offset) % 24;
        format!("{:02}h{:02}-{:02}h{:02}", start, 0, end, 0)
    }

    /// Détermine si une heure UTC appartient à une session donnée
    pub fn is_in_session(hour: u32, session: &TradingSession) -> bool {
        if session.utc_start_hour < session.utc_end_hour {
            // Session normale (ex: Londres 8-17)
            hour >= session.utc_start_hour && hour < session.utc_end_hour
        } else {
            // Session qui traverse minuit (ex: Sydney 22-7)
            hour >= session.utc_start_hour || hour < session.utc_end_hour
        }
    }

    /// Génère des recommandations basées sur les statistiques
    pub fn generate_recommendations(
        sessions: &[SessionStats],
        avg_daily_vol: f64,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Session la plus volatile
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

        // Session la moins volatile (à éviter)
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

        // Info générale sur les chevauchements
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
    fn test_is_paris_dst() {
        // 15 juin (été)
        let summer = NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        assert!(SessionAnalyzer::is_paris_dst(&summer));

        // 15 janvier (hiver)
        let winter = NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        assert!(!SessionAnalyzer::is_paris_dst(&winter));
    }

    #[test]
    fn test_is_in_session() {
        let london = TradingSession {
            name: "Londres".to_string(),
            icon: "🇬🇧".to_string(),
            utc_start_hour: 8,
            utc_end_hour: 17,
        };

        assert!(SessionAnalyzer::is_in_session(10, &london));
        assert!(!SessionAnalyzer::is_in_session(20, &london));

        let sydney = TradingSession {
            name: "Sydney".to_string(),
            icon: "🇦🇺".to_string(),
            utc_start_hour: 22,
            utc_end_hour: 7,
        };

        assert!(SessionAnalyzer::is_in_session(23, &sydney));
        assert!(SessionAnalyzer::is_in_session(5, &sydney));
        assert!(!SessionAnalyzer::is_in_session(12, &sydney));
    }
}
