// models/errors.rs - Gestion des erreurs avec thiserror
// Conforme .clinerules : toujours utiliser thiserror pour erreurs personnalisées

use thiserror::Error;

/// Type Result personnalisé pour ce projet
pub type Result<T> = std::result::Result<T, VolatilityError>;

/// Erreurs possibles dans l'application
#[derive(Debug, Error)]
pub enum VolatilityError {
    #[error("Erreur de chargement CSV: {0}")]
    CsvLoadError(String),
    
    #[error("Données CSV invalides: {0}")]
    InvalidCsvData(String),
    
    #[error("Validation échouée: {0}")]
    ValidationError(String),
    
    #[error("Calcul de métrique échoué: {0}")]
    #[allow(dead_code)]
    MetricCalculationError(String),
    
    #[error("Symbole non trouvé: {0}")]
    SymbolNotFound(String),
    
    #[error("Heure invalide: {0} (doit être entre 0 et 23)")]
    #[allow(dead_code)]
    InvalidHour(u8),
    
    #[error("Données insuffisantes: {0}")]
    InsufficientData(String),
    
    #[error("Erreur base de données: {0}")]
    DatabaseError(String),
    
    #[error("Erreur réseau: {0}")]
    #[allow(dead_code)]
    NetworkError(String),
    
    #[error("Erreur de parsing: {0}")]
    ParseError(String),
    
    #[error("Erreur I/O: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Erreur CSV parsing: {0}")]
    CsvError(#[from] csv::Error),
    
    #[error("Erreur de sérialisation JSON: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("Erreur chrono: {0}")]
    ChronoError(String),
}

// Conversion depuis validator::ValidationErrors
impl From<validator::ValidationErrors> for VolatilityError {
    fn from(err: validator::ValidationErrors) -> Self {
        VolatilityError::ValidationError(err.to_string())
    }
}

// Conversion depuis diesel::result::Error
impl From<diesel::result::Error> for VolatilityError {
    fn from(err: diesel::result::Error) -> Self {
        VolatilityError::DatabaseError(err.to_string())
    }
}
