use crate::models::archive::{Archive, NewArchive};
use crate::services::ArchiveService;
use tauri::State;

#[tauri::command]
pub async fn save_archive(
    archive_service: State<'_, ArchiveService>,
    title: String,
    archive_type: String,
    period_start: String,
    period_end: String,
    comment: Option<String>,
    data_json: String,
) -> Result<Archive, String> {
    let new_archive = NewArchive {
        title,
        archive_type,
        period_start,
        period_end,
        comment,
        data_json,
    };

    archive_service.create_archive(new_archive)
}

#[tauri::command]
pub async fn list_archives(
    archive_service: State<'_, ArchiveService>,
) -> Result<Vec<Archive>, String> {
    archive_service.list_archives()
}

#[tauri::command]
pub async fn list_all_archives(
    archive_service: State<'_, crate::services::ArchiveService>,
) -> Result<Vec<crate::models::Archive>, String> {
    archive_service.list_archives()
}

#[tauri::command]
pub async fn get_archive(
    archive_service: State<'_, ArchiveService>,
    archive_id: i32,
) -> Result<Archive, String> {
    archive_service.get_archive(archive_id)
}

#[tauri::command]
pub async fn delete_archive(
    archive_service: State<'_, ArchiveService>,
    archive_id: i32,
) -> Result<usize, String> {
    archive_service.delete_archive(archive_id)
}
