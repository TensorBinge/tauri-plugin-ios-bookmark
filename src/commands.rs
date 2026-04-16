//! Tauri command handlers for the iOS bookmark plugin.
//!
//! These functions are intentionally thin wrappers around the managed
//! `IosBookmark` state so the public command surface stays declarative and the
//! platform-specific work remains inside the mobile/desktop implementations.

use crate::{logging, models::*, IosBookmark};
use serde_json::json;
use tauri::{AppHandle, Manager, Runtime};

/// Opens the native file picker and returns a bookmark-backed file result.
#[tauri::command]
pub async fn pick_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickBookmarkRequest>,
) -> Result<Option<PickResult>, BookmarkError> {
    logging::info(
        "ios-bookmark.command",
        "bookmark.pick",
        "start",
        json!({ "hasRequest": request.is_some() }),
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.pick_and_bookmark(request).await;
    match &result {
        Ok(Some(pick_result)) => logging::info(
            "ios-bookmark.command",
            "bookmark.pick",
            "success",
            json!({ "selected": true, "fileName": pick_result.file_name }),
        ),
        Ok(None) => logging::info(
            "ios-bookmark.command",
            "bookmark.pick",
            "cancelled",
            json!({ "selected": false }),
        ),
        Err(error) => logging::error(
            "ios-bookmark.command",
            "bookmark.pick",
            "failure",
            Some(logging::bookmark_error_kind(error)),
            json!({}),
        ),
    }
    result
}

/// Opens the native folder picker and returns a bookmark-backed folder result.
#[tauri::command]
pub async fn pick_folder_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickFolderBookmarkRequest>,
) -> Result<Option<PickFolderResult>, BookmarkError> {
    logging::info(
        "ios-bookmark.command",
        "bookmark.pick_folder",
        "start",
        json!({ "hasRequest": request.is_some() }),
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.pick_folder_and_bookmark(request).await;
    match &result {
        Ok(Some(pick_result)) => logging::info(
            "ios-bookmark.command",
            "bookmark.pick_folder",
            "success",
            json!({ "selected": true, "folderName": pick_result.folder_name }),
        ),
        Ok(None) => logging::info(
            "ios-bookmark.command",
            "bookmark.pick_folder",
            "cancelled",
            json!({ "selected": false }),
        ),
        Err(error) => logging::error(
            "ios-bookmark.command",
            "bookmark.pick_folder",
            "failure",
            Some(logging::bookmark_error_kind(error)),
            json!({}),
        ),
    }
    result
}

/// Reads the content of a file previously authorized by a direct bookmark.
#[tauri::command]
pub async fn read_by_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<ReadResult, BookmarkError> {
    logging::info(
        "ios-bookmark.command",
        "bookmark.read",
        "start",
        json!({ "bookmarkIdPresent": !id.is_empty() }),
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.read_by_bookmark(id).await;
    match &result {
        Ok(read_result) => logging::info(
            "ios-bookmark.command",
            "bookmark.read",
            "success",
            json!({ "fileName": read_result.file_name }),
        ),
        Err(error) => logging::error(
            "ios-bookmark.command",
            "bookmark.read",
            "failure",
            Some(logging::bookmark_error_kind(error)),
            json!({}),
        ),
    }
    result
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
    logging::info(
        "ios-bookmark.command",
        "bookmark.read_folder",
        "start",
        json!({ "bookmarkIdPresent": !args.id.is_empty() }),
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark
        .read_by_folder_bookmark(args.id, args.target_path)
        .await;
    match &result {
        Ok(read_result) => logging::info(
            "ios-bookmark.command",
            "bookmark.read_folder",
            "success",
            json!({ "fileName": read_result.file_name }),
        ),
        Err(error) => logging::error(
            "ios-bookmark.command",
            "bookmark.read_folder",
            "failure",
            Some(logging::bookmark_error_kind(error)),
            json!({}),
        ),
    }
    result
}

/// Forgets a stored bookmark and releases the corresponding native grant.
#[tauri::command]
pub async fn forget_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<(), BookmarkError> {
    logging::info(
        "ios-bookmark.command",
        "bookmark.forget",
        "start",
        json!({ "bookmarkIdPresent": !id.is_empty() }),
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.forget_bookmark(id).await;
    match &result {
        Ok(_) => logging::info(
            "ios-bookmark.command",
            "bookmark.forget",
            "success",
            json!({}),
        ),
        Err(error) => logging::error(
            "ios-bookmark.command",
            "bookmark.forget",
            "failure",
            Some(logging::bookmark_error_kind(error)),
            json!({}),
        ),
    }
    result
}

/// Presents the native export flow for a file that already exists in the app sandbox.
#[tauri::command]
pub async fn export_file<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), BookmarkError> {
    logging::info(
        "ios-bookmark.command",
        "bookmark.export_file",
        "start",
        json!({ "fileName": std::path::Path::new(&path).file_name().and_then(|name| name.to_str()).unwrap_or(path.as_str()) }),
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark.export_file(path).await;
    match &result {
        Ok(_) => logging::info(
            "ios-bookmark.command",
            "bookmark.export_file",
            "success",
            json!({}),
        ),
        Err(error) => logging::error(
            "ios-bookmark.command",
            "bookmark.export_file",
            "failure",
            Some(logging::bookmark_error_kind(error)),
            json!({}),
        ),
    }
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
    logging::info(
        "ios-bookmark.command",
        "bookmark.export_pdf",
        "start",
        json!({ "fileName": args.file_name, "htmlLength": args.html.len(), "tocLength": args.toc.len() }),
    );
    let bookmark = app.state::<IosBookmark<R>>();
    let result = bookmark
        .export_pdf(args.file_name, args.html, args.toc)
        .await;
    match &result {
        Ok(_) => logging::info(
            "ios-bookmark.command",
            "bookmark.export_pdf",
            "success",
            json!({}),
        ),
        Err(error) => logging::error(
            "ios-bookmark.command",
            "bookmark.export_pdf",
            "failure",
            Some(logging::bookmark_error_kind(error)),
            json!({}),
        ),
    }
    result
}
