use crate::models::filesystem::{DirListing, FileContent, FsEntry, SearchResult};
use crate::services::FileSystemService;
use crate::utils::AppResult;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn fs_list_directory(
    path: String,
    show_hidden: bool,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<DirListing> {
    service.list_directory(&path, show_hidden)
}

#[tauri::command]
pub fn fs_read_file(
    path: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<FileContent> {
    service.read_file(&path)
}

#[tauri::command]
pub fn fs_write_file(
    path: String,
    content: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<()> {
    service.write_file(&path, &content)
}

#[tauri::command]
pub fn fs_create_file(
    path: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<()> {
    service.create_file(&path)
}

#[tauri::command]
pub fn fs_create_directory(
    path: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<()> {
    service.create_directory(&path)
}

#[tauri::command]
pub fn fs_delete_entry(
    path: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<()> {
    service.delete_entry(&path)
}

#[tauri::command]
pub fn fs_rename_entry(
    old_path: String,
    new_name: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<()> {
    service.rename_entry(&old_path, &new_name)
}

#[tauri::command]
pub fn fs_copy_entry(
    src: String,
    dest_dir: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<()> {
    service.copy_entry(&src, &dest_dir)
}

#[tauri::command]
pub fn fs_move_entry(
    src: String,
    dest_dir: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<()> {
    service.move_entry(&src, &dest_dir)
}

#[tauri::command]
pub fn fs_search_files(
    root: String,
    query: String,
    max_results: usize,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<Vec<SearchResult>> {
    service.search_files(&root, &query, max_results)
}

#[tauri::command]
pub fn fs_get_entry_info(
    path: String,
    service: State<'_, Arc<FileSystemService>>,
) -> AppResult<FsEntry> {
    service.get_entry_info(&path)
}
