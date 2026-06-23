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

// ── File bookmark commands ───────────────────────────────────────────

/// Opens the native file picker, creates a security-scoped bookmark, and returns
/// the file's metadata and optional content.
#[tauri::command]
pub async fn pick_file_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickFileBookmarkRequest>,
) -> Result<Option<FileBookmarkResult>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("has_request", &request.is_some())];
    run_command_with_logging(
        "pick-file-bookmark-started",
        "pick-file-bookmark-succeeded",
        "pick-file-bookmark-failed",
        &fields,
        bookmark.pick_file_bookmark(request),
    )
    .await
}

/// Reads the text content of a file previously authorized by a file bookmark.
#[tauri::command]
pub async fn read_file_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<ReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &id)];
    run_command_with_logging(
        "read-file-bookmark-started",
        "read-file-bookmark-succeeded",
        "read-file-bookmark-failed",
        &fields,
        bookmark.read_file_bookmark(id),
    )
    .await
}

/// Reads the binary content of a file previously authorized by a file bookmark.
#[tauri::command]
pub async fn read_file_bookmark_data<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<DataResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &id)];
    run_command_with_logging(
        "read-file-bookmark-data-started",
        "read-file-bookmark-data-succeeded",
        "read-file-bookmark-data-failed",
        &fields,
        bookmark.read_file_bookmark_data(id),
    )
    .await
}

/// Argument struct for the file bookmark write command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileBookmarkArgs {
    pub id: String,
    pub content: String,
}

/// Writes text content to a file previously authorized by a file bookmark.
#[tauri::command]
pub async fn write_file_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: WriteFileBookmarkArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("id", &args.id),
        field("content_length", &args.content.len()),
    ];
    run_command_with_logging(
        "write-file-bookmark-started",
        "write-file-bookmark-succeeded",
        "write-file-bookmark-failed",
        &fields,
        bookmark.write_file_bookmark(args.id, args.content),
    )
    .await
}

/// Argument struct for the file bookmark binary write command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileBookmarkDataArgs {
    pub id: String,
    pub data: Vec<u8>,
}

/// Writes binary content to a file previously authorized by a file bookmark.
#[tauri::command]
pub async fn write_file_bookmark_data<R: Runtime>(
    app: AppHandle<R>,
    args: WriteFileBookmarkDataArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("id", &args.id),
        field("data_length", &args.data.len()),
    ];
    run_command_with_logging(
        "write-file-bookmark-data-started",
        "write-file-bookmark-data-succeeded",
        "write-file-bookmark-data-failed",
        &fields,
        bookmark.write_file_bookmark_data(args.id, args.data),
    )
    .await
}

// ── Folder bookmark commands ─────────────────────────────────────────

/// Opens the native folder picker, creates a security-scoped bookmark, and returns
/// the folder's metadata.
#[tauri::command]
pub async fn pick_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickFolderBookmarkRequest>,
) -> Result<Option<FolderBookmarkResult>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("has_request", &request.is_some())];
    run_command_with_logging(
        "pick-folder-bookmark-started",
        "pick-folder-bookmark-succeeded",
        "pick-folder-bookmark-failed",
        &fields,
        bookmark.pick_folder_bookmark(request),
    )
    .await
}

/// Argument struct for folder-scoped list/read/remove operations.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderBookmarkPathArgs {
    pub id: String,
    pub path: String,
}

/// Lists directory entries within a bookmarked folder at the given path.
#[tauri::command]
pub async fn list_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: FolderBookmarkPathArgs,
) -> Result<Vec<Entry>, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &args.id), field("path", &args.path)];
    run_command_with_logging(
        "list-folder-bookmark-started",
        "list-folder-bookmark-succeeded",
        "list-folder-bookmark-failed",
        &fields,
        bookmark.list_folder_bookmark(args.id, args.path),
    )
    .await
}

/// Reads the text content of a file within a bookmarked folder scope.
#[tauri::command]
pub async fn read_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: FolderBookmarkPathArgs,
) -> Result<ReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &args.id), field("path", &args.path)];
    run_command_with_logging(
        "read-folder-bookmark-started",
        "read-folder-bookmark-succeeded",
        "read-folder-bookmark-failed",
        &fields,
        bookmark.read_folder_bookmark(args.id, args.path),
    )
    .await
}

/// Reads the binary content of a file within a bookmarked folder scope.
#[tauri::command]
pub async fn read_folder_bookmark_data<R: Runtime>(
    app: AppHandle<R>,
    args: FolderBookmarkPathArgs,
) -> Result<DataResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &args.id), field("path", &args.path)];
    run_command_with_logging(
        "read-folder-bookmark-data-started",
        "read-folder-bookmark-data-succeeded",
        "read-folder-bookmark-data-failed",
        &fields,
        bookmark.read_folder_bookmark_data(args.id, args.path),
    )
    .await
}

/// Argument struct for folder-scoped text writes.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFolderBookmarkArgs {
    pub id: String,
    pub path: String,
    pub content: String,
}

/// Writes text content to a file within a bookmarked folder scope.
#[tauri::command]
pub async fn write_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    args: WriteFolderBookmarkArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("id", &args.id),
        field("path", &args.path),
        field("content_length", &args.content.len()),
    ];
    run_command_with_logging(
        "write-folder-bookmark-started",
        "write-folder-bookmark-succeeded",
        "write-folder-bookmark-failed",
        &fields,
        bookmark.write_folder_bookmark(args.id, args.path, args.content),
    )
    .await
}

/// Argument struct for folder-scoped binary writes.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFolderBookmarkDataArgs {
    pub id: String,
    pub path: String,
    pub data: Vec<u8>,
}

/// Writes binary content to a file within a bookmarked folder scope.
#[tauri::command]
pub async fn write_folder_bookmark_data<R: Runtime>(
    app: AppHandle<R>,
    args: WriteFolderBookmarkDataArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("id", &args.id),
        field("path", &args.path),
        field("data_length", &args.data.len()),
    ];
    run_command_with_logging(
        "write-folder-bookmark-data-started",
        "write-folder-bookmark-data-succeeded",
        "write-folder-bookmark-data-failed",
        &fields,
        bookmark.write_folder_bookmark_data(args.id, args.path, args.data),
    )
    .await
}

// ── Folder-scoped mutation commands ──────────────────────────────────

/// Argument struct for `create_dir`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirArgs {
    pub id: String,
    pub parent_path: String,
    pub name: String,
}

/// Creates a subdirectory within a bookmarked folder scope.
#[tauri::command]
pub async fn create_dir<R: Runtime>(
    app: AppHandle<R>,
    args: CreateDirArgs,
) -> Result<Entry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("id", &args.id),
        field("parent_path", &args.parent_path),
        field("name", &args.name),
    ];
    run_command_with_logging(
        "create-dir-started",
        "create-dir-succeeded",
        "create-dir-failed",
        &fields,
        bookmark.create_dir(args.id, args.parent_path, args.name),
    )
    .await
}

/// Argument struct for `create_file`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileArgs {
    pub id: String,
    pub parent_path: String,
    pub name: String,
    #[serde(default)]
    pub content: Option<String>,
}

/// Creates a new file within a bookmarked folder scope.
#[tauri::command]
pub async fn create_file<R: Runtime>(
    app: AppHandle<R>,
    args: CreateFileArgs,
) -> Result<Entry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let content_len = args.content.as_ref().map(|c| c.len()).unwrap_or(0);
    let fields = [
        field("id", &args.id),
        field("parent_path", &args.parent_path),
        field("name", &args.name),
        field("content_length", &content_len),
    ];
    run_command_with_logging(
        "create-file-started",
        "create-file-succeeded",
        "create-file-failed",
        &fields,
        bookmark.create_file(args.id, args.parent_path, args.name, args.content),
    )
    .await
}

/// Argument struct for `rename`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameArgs {
    pub id: String,
    pub path: String,
    pub new_name: String,
}

/// Renames a file or directory within a bookmarked folder scope.
#[tauri::command]
pub async fn rename<R: Runtime>(
    app: AppHandle<R>,
    args: RenameArgs,
) -> Result<Entry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("id", &args.id),
        field("path", &args.path),
        field("new_name", &args.new_name),
    ];
    run_command_with_logging(
        "rename-started",
        "rename-succeeded",
        "rename-failed",
        &fields,
        bookmark.rename(args.id, args.path, args.new_name),
    )
    .await
}

/// Argument struct for `move`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveArgs {
    pub id: String,
    pub src_path: String,
    pub dest_parent_path: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Moves a file or directory to a new parent within the same bookmarked folder scope.
#[tauri::command]
pub async fn move_entry<R: Runtime>(
    app: AppHandle<R>,
    args: MoveArgs,
) -> Result<Entry, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [
        field("id", &args.id),
        field("src_path", &args.src_path),
        field("dest_parent_path", &args.dest_parent_path),
        field("name", &args.name),
    ];
    run_command_with_logging(
        "move-started",
        "move-succeeded",
        "move-failed",
        &fields,
        bookmark.move_entry(args.id, args.src_path, args.dest_parent_path, args.name),
    )
    .await
}

/// Argument struct for `remove`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveArgs {
    pub id: String,
    pub path: String,
}

/// Deletes a file or directory within a bookmarked folder scope.
#[tauri::command]
pub async fn remove<R: Runtime>(
    app: AppHandle<R>,
    args: RemoveArgs,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &args.id), field("path", &args.path)];
    run_command_with_logging(
        "remove-started",
        "remove-succeeded",
        "remove-failed",
        &fields,
        bookmark.remove(args.id, args.path),
    )
    .await
}

// ── Lifecycle commands ───────────────────────────────────────────────

/// Validates that a stored bookmark is still usable without performing I/O.
#[tauri::command]
pub async fn check_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<bool, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &id)];
    run_command_with_logging(
        "check-bookmark-started",
        "check-bookmark-succeeded",
        "check-bookmark-failed",
        &fields,
        bookmark.check_bookmark(id),
    )
    .await
}

/// Releases the native security-scoped grant and removes the bookmark from persistent storage.
#[tauri::command]
pub async fn release_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    let fields = [field("id", &id)];
    run_command_with_logging(
        "release-bookmark-started",
        "release-bookmark-succeeded",
        "release-bookmark-failed",
        &fields,
        bookmark.release_bookmark(id),
    )
    .await
}
