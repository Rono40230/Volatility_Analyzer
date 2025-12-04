use crate::schema::archives;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct Archive {
    pub id: i32,
    pub title: String,
    pub archive_type: String,
    pub period_start: String,
    pub period_end: String,
    pub comment: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub data_json: String,
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
