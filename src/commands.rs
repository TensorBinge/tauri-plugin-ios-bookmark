//! Tauri command handlers for the iOS bookmark plugin.
//!
//! These functions are intentionally thin wrappers around the managed
//! `IosBookmark` state so the public command surface stays declarative and the
//! platform-specific work remains inside the mobile/desktop implementations.

use crate::{models::*, IosBookmark};
use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};

/// Opens the native file picker and returns a bookmark-backed file result.
#[tauri::command]
pub async fn pick_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickBookmarkRequest>,
) -> Result<Option<PickResult>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.pick_and_bookmark(request).await
}

/// Opens the native folder picker and returns a bookmark-backed folder result.
#[tauri::command]
pub async fn pick_folder_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickFolderBookmarkRequest>,
) -> Result<Option<PickFolderResult>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.pick_folder_and_bookmark(request).await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListByFolderBookmarkArgs {
    pub id: String,
    pub target_path: String,
}

#[tauri::command]
pub async fn list_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: ListByFolderBookmarkArgs,
) -> Result<Vec<FolderBookmarkEntry>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .list_by_folder_bookmark(args.id, args.target_path)
        .await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderByFolderBookmarkArgs {
    pub id: String,
    pub parent_path: String,
    pub name: String,
}

#[tauri::command]
pub async fn create_folder_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: CreateFolderByFolderBookmarkArgs,
) -> Result<FolderBookmarkEntry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .create_folder_by_folder_bookmark(args.id, args.parent_path, args.name)
        .await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarkdownFileByFolderBookmarkArgs {
    pub id: String,
    pub parent_path: String,
    pub name: String,
    pub content: String,
}

#[tauri::command]
pub async fn create_markdown_file_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: CreateMarkdownFileByFolderBookmarkArgs,
) -> Result<FolderBookmarkEntry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .create_markdown_file_by_folder_bookmark(args.id, args.parent_path, args.name, args.content)
        .await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameByFolderBookmarkArgs {
    pub id: String,
    pub target_path: String,
    pub name: String,
}

#[tauri::command]
pub async fn rename_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: RenameByFolderBookmarkArgs,
) -> Result<FolderBookmarkEntry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .rename_by_folder_bookmark(args.id, args.target_path, args.name)
        .await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveByFolderBookmarkArgs {
    pub id: String,
    pub source_path: String,
    pub destination_parent_path: String,
    pub name: String,
}

#[tauri::command]
pub async fn move_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: MoveByFolderBookmarkArgs,
) -> Result<FolderBookmarkEntry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .move_by_folder_bookmark(
            args.id,
            args.source_path,
            args.destination_parent_path,
            args.name,
        )
        .await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteByFolderBookmarkArgs {
    pub id: String,
    pub target_path: String,
}

#[tauri::command]
pub async fn delete_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: DeleteByFolderBookmarkArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .delete_by_folder_bookmark(args.id, args.target_path)
        .await
}

/// Reads the content of a file previously authorized by a direct bookmark.
#[tauri::command]
pub async fn read_by_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<ReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.read_by_bookmark(id).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteByBookmarkArgs {
    pub id: String,
    pub content: String,
}

/// Writes text content to a file previously authorized by a direct bookmark.
#[tauri::command]
pub async fn write_by_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: WriteByBookmarkArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.write_by_bookmark(args.id, args.content).await
}

/// Argument object used for the folder-bookmark read command.
///
/// A named struct keeps the Tauri command boundary aligned with the camelCase
/// payload shape expected by the generated guest API.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadByFolderBookmarkArgs {
    pub id: String,
    pub target_path: String,
}

/// Reads a file within a previously authorized folder scope.
#[tauri::command]
pub async fn read_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: ReadByFolderBookmarkArgs,
) -> Result<ReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .read_by_folder_bookmark(args.id, args.target_path)
        .await
}

/// Reads binary file content within a previously authorized folder scope.
#[tauri::command]
pub async fn read_binary_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: ReadByFolderBookmarkArgs,
) -> Result<BinaryReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .read_binary_by_folder_bookmark(args.id, args.target_path)
        .await
}

/// Writes a file within a previously authorized folder scope.
#[tauri::command]
pub async fn write_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: WriteByFolderBookmarkArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark
        .write_by_folder_bookmark(args.id, args.target_path, args.content)
        .await
}

/// Forgets a stored bookmark and releases the corresponding native grant.
#[tauri::command]
pub async fn forget_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.forget_bookmark(id).await
}

/// Presents the native export flow for a file that already exists in the app sandbox.
#[tauri::command]
pub async fn export_file<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), BookmarkError> {
    println!("[ios-bookmark] rust command: export_file start path={path}");
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.export_file(path).await;
    println!(
        "[ios-bookmark] rust command: export_file finish success={}",
        result.is_ok()
    );
    result
}

/// Argument object used for the PDF export command.
///
/// `toc` is defaulted so older guest API versions that only send `fileName`
/// and `html` continue to deserialize successfully.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPdfArgs {
    pub file_name: String,
    pub html: String,
    #[serde(default)]
    pub toc: Vec<ExportTocEntry>,
}

/// Renders HTML to a temporary PDF and presents the native export flow for it.
#[tauri::command]
pub async fn export_pdf<R: Runtime>(
    app: AppHandle<R>,
    args: ExportPdfArgs,
) -> Result<(), BookmarkError> {
    println!(
        "[ios-bookmark] rust command: export_pdf start file_name={} html_length={} toc_length={}",
        args.file_name,
        args.html.len(),
        args.toc.len()
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark
        .export_pdf(args.file_name, args.html, args.toc)
        .await;
    println!(
        "[ios-bookmark] rust command: export_pdf finish success={}",
        result.is_ok()
    );
    result
}
