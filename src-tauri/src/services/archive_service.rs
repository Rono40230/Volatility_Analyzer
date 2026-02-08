use crate::db::DbPool;
use crate::models::archive::{Archive, ArchiveLight, NewArchive};
use crate::schema::archives;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use tracing::error;

#[derive(Clone)]
pub struct ArchiveService {
    pool: DbPool,
}

impl ArchiveService {
    pub fn new(pool: DbPool) -> Self {
        ArchiveService { pool }
    }

    pub fn create_archive(&self, new_archive: NewArchive) -> Result<Archive, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::insert_into(archives::table)
            .values(&new_archive)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Error creating archive: {}", e);
                e.to_string()
            })?;

        archives::table
            .order(archives::id.desc())
            .first(&mut conn)
            .map_err(|e| {
                error!("Error getting created archive: {}", e);
                e.to_string()
            })
    }

    pub fn list_archives(&self) -> Result<Vec<Archive>, String> {
        tracing::debug!("🔍 ArchiveService.list_archives: getting connection");
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("❌ Pool error: {}", e);
            e.to_string()
        })?;

        tracing::debug!("🔍 ArchiveService.list_archives: executing query");
        let result = archives::table
            .order(archives::created_at.desc())
            .load::<Archive>(&mut conn);

        match &result {
            Ok(archives) => tracing::info!(
                "✅ ArchiveService.list_archives: {} archives loaded",
                archives.len()
            ),
            Err(e) => tracing::error!("❌ ArchiveService.list_archives SQL error: {}", e),
        }

        result.map_err(|e| {
            error!("Error listing archives: {}", e);
            e.to_string()
        })
    }

    /// Liste légère : exclut data_json, extrait pair + event_label via json_extract().
    pub fn list_archives_light(&self) -> Result<Vec<ArchiveLight>, String> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("❌ Pool error (light): {}", e);
            e.to_string()
        })?;

        diesel::sql_query(
            "SELECT id, title, archive_type, period_start, period_end, comment, created_at, \
             COALESCE(json_extract(data_json, '$.pair'), json_extract(data_json, '$.analysisResult.symbol'), '') as pair, \
             COALESCE(json_extract(data_json, '$.eventLabel'), json_extract(data_json, '$.eventType'), json_extract(data_json, '$.pair'), '') as event_label \
             FROM archives ORDER BY created_at DESC"
        )
        .load::<ArchiveLight>(&mut conn)
        .map_err(|e| {
            error!("Error listing archives light: {}", e);
            e.to_string()
        })
    }

    pub fn get_archive(&self, archive_id: i32) -> Result<Archive, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        archives::table
            .find(archive_id)
            .first::<Archive>(&mut conn)
            .map_err(|e| {
                error!("Error getting archive {}: {}", archive_id, e);
                e.to_string()
            })
    }

    pub fn delete_archive(&self, archive_id: i32) -> Result<usize, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::delete(archives::table.find(archive_id))
            .execute(&mut conn)
            .map_err(|e| {
                error!("Error deleting archive {}: {}", archive_id, e);
                e.to_string()
            })
    }

    pub fn delete_all_archives(&self) -> Result<usize, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::delete(archives::table)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Error deleting all archives: {}", e);
                e.to_string()
            })
    }
}
