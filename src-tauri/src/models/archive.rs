use crate::schema::archives;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct Archive {
    pub id: i32,
    pub title: String,
    pub archive_type: String,
    pub period_start: String,
    pub period_end: String,
    pub comment: Option<String>,
    pub created_at: String,
    pub data_json: String,
}

/// Version légère sans data_json — pour lister les archives sans charger les blobs.
/// Les champs `pair` et `event_label` sont extraits via json_extract() SQLite.
#[derive(QueryableByName, Serialize, Deserialize, Debug, Clone)]
pub struct ArchiveLight {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = Text)]
    pub archive_type: String,
    #[diesel(sql_type = Text)]
    pub period_start: String,
    #[diesel(sql_type = Text)]
    pub period_end: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub comment: Option<String>,
    #[diesel(sql_type = Text)]
    pub created_at: String,
    #[diesel(sql_type = Text)]
    pub pair: String,
    #[diesel(sql_type = Text)]
    pub event_label: String,
}

#[derive(Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = archives)]
pub struct NewArchive {
    pub title: String,
    pub archive_type: String,
    pub period_start: String,
    pub period_end: String,
    pub comment: Option<String>,
    pub data_json: String,
}

/// Paires/événements archivés pour marquer les cellules heatmap
#[derive(QueryableByName, Serialize, Deserialize, Debug, Clone)]
pub struct ArchivedPairEvent {
    #[diesel(sql_type = Text)]
    pub pair: String,
    #[diesel(sql_type = Text)]
    pub event_type: String,
}
