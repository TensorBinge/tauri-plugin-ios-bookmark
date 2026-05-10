//! Tauri command handlers for the iOS bookmark plugin.
//!
//! These functions are intentionally thin wrappers around the managed
//! `IosBookmark` state so the public command surface stays declarative and the
//! platform-specific work remains inside the mobile/desktop implementations.

use crate::{models::*, IosBookmark};
use serde::Deserialize;
use std::future::Future;
use tauri::{AppHandle, Manager, Runtime};

const COMMAND_SCOPE: &str = "ios-bookmark.command";

fn field<T: serde::Serialize>(key: &'static str, value: &T) -> (&'static str, String) {
    (key, crate::logging::serialize_log_value(value))
}

async fn run_command_with_logging<T, F>(
    started_event: &str,
    succeeded_event: &str,
    failed_event: &str,
    fields: &[(&'static str, String)],
    future: F,
) -> Result<T, BookmarkError>
where
    F: Future<Output = Result<T, BookmarkError>>,
{
    crate::logging::info(COMMAND_SCOPE, started_event, fields);
    let result = future.await;
    match &result {
        Ok(_) => crate::logging::info(COMMAND_SCOPE, succeeded_event, fields),
        Err(error) => {
            let mut failed_fields = fields.to_vec();
            failed_fields.push((
                "error",
                crate::logging::serialize_log_value(&error.to_string()),
            ));
            crate::logging::warn(COMMAND_SCOPE, failed_event, &failed_fields);
        }
    }
    result
}

/// Opens the native file picker and returns a bookmark-backed file result.
#[tauri::command]
pub async fn pick_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickBookmarkRequest>,
) -> Result<Option<PickResult>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("has_request", &request.is_some())];
    run_command_with_logging(
        "pick-and-bookmark-started",
        "pick-and-bookmark-succeeded",
        "pick-and-bookmark-failed",
        &fields,
        bookmark.pick_and_bookmark(request),
    )
    .await
}

/// Opens the native folder picker and returns a bookmark-backed folder result.
#[tauri::command]
pub async fn pick_folder_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickFolderBookmarkRequest>,
) -> Result<Option<PickFolderResult>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("has_request", &request.is_some())];
    run_command_with_logging(
        "pick-folder-and-bookmark-started",
        "pick-folder-and-bookmark-succeeded",
        "pick-folder-and-bookmark-failed",
        &fields,
        bookmark.pick_folder_and_bookmark(request),
    )
    .await
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
    let fields = [
        field("bookmark_id", &args.id),
        field("target_path", &args.target_path),
    ];
    run_command_with_logging(
        "list-by-folder-bookmark-started",
        "list-by-folder-bookmark-succeeded",
        "list-by-folder-bookmark-failed",
        &fields,
        bookmark.list_by_folder_bookmark(args.id, args.target_path),
    )
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
    let fields = [
        field("bookmark_id", &args.id),
        field("parent_path", &args.parent_path),
        field("name", &args.name),
    ];
    run_command_with_logging(
        "create-folder-by-folder-bookmark-started",
        "create-folder-by-folder-bookmark-succeeded",
        "create-folder-by-folder-bookmark-failed",
        &fields,
        bookmark.create_folder_by_folder_bookmark(args.id, args.parent_path, args.name),
    )
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
    let fields = [
        field("bookmark_id", &args.id),
        field("parent_path", &args.parent_path),
        field("name", &args.name),
        field("content_length", &args.content.len()),
    ];
    run_command_with_logging(
        "create-markdown-file-by-folder-bookmark-started",
        "create-markdown-file-by-folder-bookmark-succeeded",
        "create-markdown-file-by-folder-bookmark-failed",
        &fields,
        bookmark.create_markdown_file_by_folder_bookmark(
            args.id,
            args.parent_path,
            args.name,
            args.content,
        ),
    )
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
    let fields = [
        field("bookmark_id", &args.id),
        field("target_path", &args.target_path),
        field("name", &args.name),
    ];
    run_command_with_logging(
        "rename-by-folder-bookmark-started",
        "rename-by-folder-bookmark-succeeded",
        "rename-by-folder-bookmark-failed",
        &fields,
        bookmark.rename_by_folder_bookmark(args.id, args.target_path, args.name),
    )
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
    let fields = [
        field("bookmark_id", &args.id),
        field("source_path", &args.source_path),
        field("destination_parent_path", &args.destination_parent_path),
        field("name", &args.name),
    ];
    run_command_with_logging(
        "move-by-folder-bookmark-started",
        "move-by-folder-bookmark-succeeded",
        "move-by-folder-bookmark-failed",
        &fields,
        bookmark.move_by_folder_bookmark(
            args.id,
            args.source_path,
            args.destination_parent_path,
            args.name,
        ),
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
    let fields = [
        field("bookmark_id", &args.id),
        field("target_path", &args.target_path),
    ];
    run_command_with_logging(
        "delete-by-folder-bookmark-started",
        "delete-by-folder-bookmark-succeeded",
        "delete-by-folder-bookmark-failed",
        &fields,
        bookmark.delete_by_folder_bookmark(args.id, args.target_path),
    )
    .await
}

/// Reads the content of a file previously authorized by a direct bookmark.
#[tauri::command]
pub async fn read_by_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<ReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("bookmark_id", &id)];
    run_command_with_logging(
        "read-by-bookmark-started",
        "read-by-bookmark-succeeded",
        "read-by-bookmark-failed",
        &fields,
        bookmark.read_by_bookmark(id),
    )
    .await
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
    let fields = [
        field("bookmark_id", &args.id),
        field("content_length", &args.content.len()),
    ];
    run_command_with_logging(
        "write-by-bookmark-started",
        "write-by-bookmark-succeeded",
        "write-by-bookmark-failed",
        &fields,
        bookmark.write_by_bookmark(args.id, args.content),
    )
    .await
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
    let fields = [
        field("bookmark_id", &args.id),
        field("target_path", &args.target_path),
    ];
    run_command_with_logging(
        "read-by-folder-bookmark-started",
        "read-by-folder-bookmark-succeeded",
        "read-by-folder-bookmark-failed",
        &fields,
        bookmark.read_by_folder_bookmark(args.id, args.target_path),
    )
    .await
}

/// Reads binary file content within a previously authorized folder scope.
#[tauri::command]
pub async fn read_binary_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: ReadByFolderBookmarkArgs,
) -> Result<BinaryReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("bookmark_id", &args.id),
        field("target_path", &args.target_path),
    ];
    run_command_with_logging(
        "read-binary-by-folder-bookmark-started",
        "read-binary-by-folder-bookmark-succeeded",
        "read-binary-by-folder-bookmark-failed",
        &fields,
        bookmark.read_binary_by_folder_bookmark(args.id, args.target_path),
    )
    .await
}

/// Writes a file within a previously authorized folder scope.
#[tauri::command]
pub async fn write_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: WriteByFolderBookmarkArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("bookmark_id", &args.id),
        field("target_path", &args.target_path),
        field("content_length", &args.content.len()),
    ];
    run_command_with_logging(
        "write-by-folder-bookmark-started",
        "write-by-folder-bookmark-succeeded",
        "write-by-folder-bookmark-failed",
        &fields,
        bookmark.write_by_folder_bookmark(args.id, args.target_path, args.content),
    )
    .await
}

/// Forgets a stored bookmark and releases the corresponding native grant.
#[tauri::command]
pub async fn forget_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("bookmark_id", &id)];
    run_command_with_logging(
        "forget-bookmark-started",
        "forget-bookmark-succeeded",
        "forget-bookmark-failed",
        &fields,
        bookmark.forget_bookmark(id),
    )
    .await
}

/// Presents the native export flow for a file that already exists in the app sandbox.
#[tauri::command]
pub async fn export_file<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("path", &path)];
    run_command_with_logging(
        "export-file-started",
        "export-file-succeeded",
        "export-file-failed",
        &fields,
        bookmark.export_file(path),
    )
    .await
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
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("file_name", &args.file_name),
        field("html_length", &args.html.len()),
        field("toc_length", &args.toc.len()),
    ];
    run_command_with_logging(
        "export-pdf-started",
        "export-pdf-succeeded",
        "export-pdf-failed",
        &fields,
        bookmark.export_pdf(args.file_name, args.html, args.toc),
    )
    .await
}
