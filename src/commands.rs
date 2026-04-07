//! Tauri command handlers for the iOS bookmark plugin.
//!
//! These functions are intentionally thin wrappers around the managed
//! `IosBookmark` state so the public command surface stays declarative and the
//! platform-specific work remains inside the mobile/desktop implementations.

use crate::{models::*, IosBookmark};
use tauri::{AppHandle, Manager, Runtime};

/// Opens the native file picker and returns a bookmark-backed file result.
#[tauri::command]
pub async fn pick_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickBookmarkRequest>,
) -> Result<PickResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.pick_and_bookmark(request).await
}

/// Opens the native folder picker and returns a bookmark-backed folder result.
#[tauri::command]
pub async fn pick_folder_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickFolderBookmarkRequest>,
) -> Result<PickFolderResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.pick_folder_and_bookmark(request).await
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
pub async fn export_file<R: Runtime>(
    app: AppHandle<R>,
    path: String,
) -> Result<(), BookmarkError> {
    println!("[ios-bookmark] rust command: export_file start path={path}");
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.export_file(path).await;
    println!("[ios-bookmark] rust command: export_file finish success={}", result.is_ok());
    result
}

/// Renders HTML to a temporary PDF and presents the native export flow for it.
#[tauri::command]
pub async fn export_pdf<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
    html: String,
) -> Result<(), BookmarkError> {
    println!(
        "[ios-bookmark] rust command: export_pdf start file_name={} html_length={}",
        file_name,
        html.len()
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.export_pdf(file_name, html).await;
    println!("[ios-bookmark] rust command: export_pdf finish success={}", result.is_ok());
    result
}
