//! Mobile runtime implementation for the iOS bookmark plugin.
//!
//! Tauri exposes mobile plugins through a `PluginHandle`; this module wraps that
//! handle in a small Rust API so command handlers can stay platform-agnostic.

use crate::{
    logging, models::*, normalize_ios_bookmark_error, pick_and_bookmark_payload,
    pick_folder_and_bookmark_payload,
};
use serde::de::DeserializeOwned;
use serde_json::json;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_ios_bookmark);

/// Initializes the native mobile plugin.
///
/// The crate is mobile-enabled, but only iOS actually provides the native
/// security-scoped bookmark implementation. Other mobile targets fail fast with
/// `BookmarkError::Unsupported` so the caller can surface a clear capability error.
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<IosBookmark<R>, BookmarkError> {
    #[cfg(target_os = "ios")]
    let handle = api
        .register_ios_plugin(init_plugin_ios_bookmark)
        .map_err(|e| BookmarkError::Native(e.to_string()))?;

    // Android is not supported; this plugin is iOS-only.
    #[cfg(not(target_os = "ios"))]
    let handle = {
        let _ = api;
        return Err(BookmarkError::Unsupported);
    };

    Ok(IosBookmark(handle))
}

/// Thin wrapper around the native mobile plugin handle.
pub struct IosBookmark<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> IosBookmark<R> {
    /// Presents the file picker, creates a security-scoped bookmark, and returns the first file.
    pub async fn pick_and_bookmark(
        &self,
        request: Option<PickBookmarkRequest>,
    ) -> Result<Option<PickResult>, BookmarkError> {
        logging::info(
            "ios-bookmark.mobile",
            "bookmark.pick",
            "start",
            json!({ "hasRequest": request.is_some() }),
        );
        self.0
            .run_mobile_plugin_async("pickAndBookmark", pick_and_bookmark_payload(request))
            .await
            .map(|result| {
                logging::info(
                    "ios-bookmark.mobile",
                    "bookmark.pick",
                    if result.is_some() {
                        "success"
                    } else {
                        "cancelled"
                    },
                    json!({ "selected": result.is_some() }),
                );
                result
            })
            .map_err(|e| {
                let error = normalize_ios_bookmark_error(e.to_string());
                logging::error(
                    "ios-bookmark.mobile",
                    "bookmark.pick",
                    "failure",
                    Some(logging::bookmark_error_kind(&error)),
                    json!({}),
                );
                error
            })
    }

    /// Presents the folder picker and stores a bookmark that can authorize descendant reads.
    pub async fn pick_folder_and_bookmark(
        &self,
        request: Option<PickFolderBookmarkRequest>,
    ) -> Result<Option<PickFolderResult>, BookmarkError> {
        logging::info(
            "ios-bookmark.mobile",
            "bookmark.pick_folder",
            "start",
            json!({ "hasRequest": request.is_some() }),
        );
        self.0
            .run_mobile_plugin_async(
                "pickFolderAndBookmark",
                pick_folder_and_bookmark_payload(request),
            )
            .await
            .map(|result| {
                logging::info(
                    "ios-bookmark.mobile",
                    "bookmark.pick_folder",
                    if result.is_some() {
                        "success"
                    } else {
                        "cancelled"
                    },
                    json!({ "selected": result.is_some() }),
                );
                result
            })
            .map_err(|e| {
                let error = normalize_ios_bookmark_error(e.to_string());
                logging::error(
                    "ios-bookmark.mobile",
                    "bookmark.pick_folder",
                    "failure",
                    Some(logging::bookmark_error_kind(&error)),
                    json!({}),
                );
                error
            })
    }

    /// Reads a previously bookmarked file directly by bookmark id.
    pub async fn read_by_bookmark(&self, id: String) -> Result<ReadResult, BookmarkError> {
        logging::info(
            "ios-bookmark.mobile",
            "bookmark.read",
            "start",
            json!({ "bookmarkIdPresent": !id.is_empty() }),
        );
        self.0
            .run_mobile_plugin_async("readByBookmark", serde_json::json!({ "id": id }))
            .await
            .map(|result| {
                logging::info(
                    "ios-bookmark.mobile",
                    "bookmark.read",
                    "success",
                    json!({ "fileName": result.file_name }),
                );
                result
            })
            .map_err(|e| {
                let error = normalize_ios_bookmark_error(e.to_string());
                logging::error(
                    "ios-bookmark.mobile",
                    "bookmark.read",
                    "failure",
                    Some(logging::bookmark_error_kind(&error)),
                    json!({}),
                );
                error
            })
    }

    /// Reads a descendant file by combining a folder bookmark id with the target path.
    pub async fn read_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
    ) -> Result<ReadResult, BookmarkError> {
        logging::info(
            "ios-bookmark.mobile",
            "bookmark.read_folder",
            "start",
            json!({ "bookmarkIdPresent": !id.is_empty(), "targetFileName": std::path::Path::new(&target_path).file_name().and_then(|name| name.to_str()).unwrap_or(target_path.as_str()) }),
        );
        self.0
            .run_mobile_plugin_async(
                "readByFolderBookmark",
                serde_json::json!({ "id": id, "targetPath": target_path }),
            )
            .await
            .map(|result| {
                logging::info(
                    "ios-bookmark.mobile",
                    "bookmark.read_folder",
                    "success",
                    json!({ "fileName": result.file_name }),
                );
                result
            })
            .map_err(|e| {
                let error = normalize_ios_bookmark_error(e.to_string());
                logging::error(
                    "ios-bookmark.mobile",
                    "bookmark.read_folder",
                    "failure",
                    Some(logging::bookmark_error_kind(&error)),
                    json!({}),
                );
                error
            })
    }

    /// Forgets a stored bookmark so the native side can release the security-scoped grant.
    pub async fn forget_bookmark(&self, id: String) -> Result<(), BookmarkError> {
        logging::info(
            "ios-bookmark.mobile",
            "bookmark.forget",
            "start",
            json!({ "bookmarkIdPresent": !id.is_empty() }),
        );
        self.0
            .run_mobile_plugin_async("forgetBookmark", serde_json::json!({ "id": id }))
            .await
            .map(|result| {
                logging::info(
                    "ios-bookmark.mobile",
                    "bookmark.forget",
                    "success",
                    json!({}),
                );
                result
            })
            .map_err(|e| {
                let error = normalize_ios_bookmark_error(e.to_string());
                logging::error(
                    "ios-bookmark.mobile",
                    "bookmark.forget",
                    "failure",
                    Some(logging::bookmark_error_kind(&error)),
                    json!({}),
                );
                error
            })
    }

    /// Presents the native export picker for a file that already exists in the app sandbox.
    pub async fn export_file(&self, path: String) -> Result<(), BookmarkError> {
        logging::info(
            "ios-bookmark.mobile",
            "bookmark.export_file",
            "start",
            json!({ "fileName": std::path::Path::new(&path).file_name().and_then(|name| name.to_str()).unwrap_or(path.as_str()) }),
        );
        self.0
            .run_mobile_plugin_async("exportFile", serde_json::json!({ "path": path }))
            .await
            .map(|result| {
                logging::info(
                    "ios-bookmark.mobile",
                    "bookmark.export_file",
                    "success",
                    json!({}),
                );
                result
            })
            .map_err(|e| {
                let error = normalize_ios_bookmark_error(e.to_string());
                logging::error(
                    "ios-bookmark.mobile",
                    "bookmark.export_file",
                    "failure",
                    Some(logging::bookmark_error_kind(&error)),
                    json!({}),
                );
                error
            })
    }

    /// Renders HTML to a temporary PDF and presents the native export picker for it.
    pub async fn export_pdf(
        &self,
        file_name: String,
        html: String,
        toc: Vec<ExportTocEntry>,
    ) -> Result<(), BookmarkError> {
        logging::info(
            "ios-bookmark.mobile",
            "bookmark.export_pdf",
            "start",
            json!({ "fileName": file_name, "htmlLength": html.len(), "tocLength": toc.len() }),
        );
        self.0
            .run_mobile_plugin_async(
                "exportPdf",
                serde_json::json!({ "fileName": file_name, "html": html, "toc": toc }),
            )
            .await
            .map(|result| {
                logging::info(
                    "ios-bookmark.mobile",
                    "bookmark.export_pdf",
                    "success",
                    json!({}),
                );
                result
            })
            .map_err(|e| {
                let error = normalize_ios_bookmark_error(e.to_string());
                logging::error(
                    "ios-bookmark.mobile",
                    "bookmark.export_pdf",
                    "failure",
                    Some(logging::bookmark_error_kind(&error)),
                    json!({}),
                );
                error
            })
    }
}
