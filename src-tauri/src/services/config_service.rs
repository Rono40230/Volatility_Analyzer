//! Service de gestion de la configuration de l'application
//! Gère la persistance du fichier calendrier sélectionné

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration de l'application
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Fichier calendrier actuellement sélectionné
    pub selected_calendar_file: Option<String>,
    
    /// Timestamp de dernière mise à jour
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

/// Service de configuration
pub struct ConfigService;

impl ConfigService {
    /// Obtient le chemin du fichier de configuration
    fn get_config_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .context("Impossible de trouver le dossier de données local")?;

        let app_dir = data_dir.join("volatility-analyzer");
        fs::create_dir_all(&app_dir)
            .context("Impossible de créer le dossier de l'application")?;

        Ok(app_dir.join("config.json"))
    }

    /// Charge la configuration depuis le fichier
    pub fn load_config() -> Result<AppConfig> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // Créer une config par défaut si le fichier n'existe pas
            let default_config = AppConfig::default();
            Self::save_config(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .context("Impossible de lire le fichier de configuration")?;

        let config: AppConfig = serde_json::from_str(&content)
            .context("Impossible de parser le fichier de configuration")?;

        Ok(config)
    }

    /// Sauvegarde la configuration dans le fichier
    pub fn save_config(config: &AppConfig) -> Result<()> {
        let config_path = Self::get_config_path()?;

        let content = serde_json::to_string_pretty(config)
            .context("Impossible de sérialiser la configuration")?;

        fs::write(&config_path, content)
            .context("Impossible d'écrire le fichier de configuration")?;

        Ok(())
    }

    /// Définit le fichier calendrier sélectionné
    pub fn set_selected_calendar_file(filename: String) -> Result<()> {
        let mut config = Self::load_config()?;
        config.selected_calendar_file = Some(filename);
        config.last_updated = Some(chrono::Utc::now().to_rfc3339());
        Self::save_config(&config)?;
        Ok(())
    }

    /// Obtient le fichier calendrier sélectionné
    pub fn get_selected_calendar_file() -> Result<Option<String>> {
        let config = Self::load_config()?;
        Ok(config.selected_calendar_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.selected_calendar_file.is_none());
        assert!(config.last_updated.is_none());
    }
}
