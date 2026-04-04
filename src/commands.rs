use crate::{models::*, IosBookmark};
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
pub async fn pick_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickBookmarkRequest>,
) -> Result<PickResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.pick_and_bookmark(request).await
}

#[tauri::command]
pub async fn pick_folder_and_bookmark<R: Runtime>(
    app: AppHandle<R>,
    request: Option<PickFolderBookmarkRequest>,
) -> Result<PickFolderResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.pick_folder_and_bookmark(request).await
}

#[tauri::command]
pub async fn read_by_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<ReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.read_by_bookmark(id).await
}

#[tauri::command]
pub async fn read_by_folder_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
    target_path: String,
) -> Result<ReadResult, BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.read_by_folder_bookmark(id, target_path).await
}

#[tauri::command]
pub async fn forget_bookmark<R: Runtime>(
    app: AppHandle<R>,
    id: String,
) -> Result<(), BookmarkError> {
    let bookmark = app.state::<IosBookmark<R>>();
    bookmark.forget_bookmark(id).await
}
