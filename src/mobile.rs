//! Mobile runtime implementation for the iOS bookmark plugin.
//!
//! Tauri exposes mobile plugins through a `PluginHandle`; this module wraps that
//! handle in a small Rust API so command handlers can stay platform-agnostic.

use crate::{
    models::*,
    normalize_ios_bookmark_error,
    payloads::{
        check_bookmark_payload, create_dir_payload, create_file_payload, list_folder_bookmark_payload,
        move_payload, pick_file_bookmark_payload, pick_folder_bookmark_payload,
        read_file_bookmark_data_payload, read_file_bookmark_payload,
        read_folder_bookmark_data_payload, read_folder_bookmark_payload,
        release_bookmark_payload, remove_payload, rename_payload,
        write_file_bookmark_data_payload, write_file_bookmark_payload,
        write_folder_bookmark_data_payload, write_folder_bookmark_payload,
    },
};
use serde::de::DeserializeOwned;
use std::future::Future;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

const MOBILE_SCOPE: &str = "ios-bookmark.mobile";

fn field<T: serde::Serialize>(key: &'static str, value: &T) -> (&'static str, String) {
    (key, crate::logging::serialize_log_value(value))
}

async fn run_mobile_call<T, F>(
    started_event: &str,
    succeeded_event: &str,
    failed_event: &str,
    fields: &[(&'static str, String)],
    future: F,
) -> Result<T, BookmarkError>
where
    F: Future<Output = Result<T, BookmarkError>>,
{
    crate::logging::info(MOBILE_SCOPE, started_event, fields);
    let result = future.await;
    match &result {
        Ok(_) => crate::logging::info(MOBILE_SCOPE, succeeded_event, fields),
        Err(error) => {
            let mut failed_fields = fields.to_vec();
            failed_fields.push((
                "error",
                crate::logging::serialize_log_value(&error.to_string()),
            ));
            crate::logging::warn(MOBILE_SCOPE, failed_event, &failed_fields);
        }
    }
    result
}

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
    // ── File bookmark operations ────────────────────────────────────

    /// Presents the file picker, creates a security-scoped bookmark, and returns
    /// the file metadata and optional content.
    pub async fn pick_file_bookmark(
        &self,
        request: Option<PickFileBookmarkRequest>,
    ) -> Result<Option<FileBookmarkResult>, BookmarkError> {
        let fields = [field("has_request", &request.is_some())];
        run_mobile_call(
            "pick-file-bookmark-started",
            "pick-file-bookmark-succeeded",
            "pick-file-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "pickFileBookmark",
                        pick_file_bookmark_payload(request),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Reads the text content of a file previously authorized by a file bookmark.
    pub async fn read_file_bookmark(&self, id: String) -> Result<ReadResult, BookmarkError> {
        let fields = [field("id", &id)];
        run_mobile_call(
            "read-file-bookmark-started",
            "read-file-bookmark-succeeded",
            "read-file-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "readFileBookmark",
                        read_file_bookmark_payload(id),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Reads the binary content of a file previously authorized by a file bookmark.
    pub async fn read_file_bookmark_data(
        &self,
        id: String,
    ) -> Result<DataResult, BookmarkError> {
        let fields = [field("id", &id)];
        run_mobile_call(
            "read-file-bookmark-data-started",
            "read-file-bookmark-data-succeeded",
            "read-file-bookmark-data-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "readFileBookmarkData",
                        read_file_bookmark_data_payload(id),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Writes text content to a file previously authorized by a file bookmark.
    pub async fn write_file_bookmark(
        &self,
        id: String,
        content: String,
    ) -> Result<(), BookmarkError> {
        let fields = [field("id", &id), field("content_length", &content.len())];
        run_mobile_call(
            "write-file-bookmark-started",
            "write-file-bookmark-succeeded",
            "write-file-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "writeFileBookmark",
                        write_file_bookmark_payload(id, content),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Writes binary content to a file previously authorized by a file bookmark.
    pub async fn write_file_bookmark_data(
        &self,
        id: String,
        data: Vec<u8>,
    ) -> Result<(), BookmarkError> {
        let fields = [field("id", &id), field("data_length", &data.len())];
        run_mobile_call(
            "write-file-bookmark-data-started",
            "write-file-bookmark-data-succeeded",
            "write-file-bookmark-data-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "writeFileBookmarkData",
                        write_file_bookmark_data_payload(id, data),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    // ── Folder bookmark operations ──────────────────────────────────

    /// Presents the folder picker and stores a bookmark that can authorize
    /// descendant reads.
    pub async fn pick_folder_bookmark(
        &self,
        request: Option<PickFolderBookmarkRequest>,
    ) -> Result<Option<FolderBookmarkResult>, BookmarkError> {
        let fields = [field("has_request", &request.is_some())];
        run_mobile_call(
            "pick-folder-bookmark-started",
            "pick-folder-bookmark-succeeded",
            "pick-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "pickFolderBookmark",
                        pick_folder_bookmark_payload(request),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Lists directory entries inside a previously authorized folder bookmark scope.
    pub async fn list_folder_bookmark(
        &self,
        id: String,
        path: String,
    ) -> Result<Vec<Entry>, BookmarkError> {
        let fields = [field("id", &id), field("path", &path)];
        run_mobile_call(
            "list-folder-bookmark-started",
            "list-folder-bookmark-succeeded",
            "list-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "listFolderBookmark",
                        list_folder_bookmark_payload(id, path),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Reads the text content of a file within a bookmarked folder scope.
    pub async fn read_folder_bookmark(
        &self,
        id: String,
        path: String,
    ) -> Result<ReadResult, BookmarkError> {
        let fields = [field("id", &id), field("path", &path)];
        run_mobile_call(
            "read-folder-bookmark-started",
            "read-folder-bookmark-succeeded",
            "read-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "readFolderBookmark",
                        read_folder_bookmark_payload(id, path),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Reads the binary content of a file within a bookmarked folder scope.
    pub async fn read_folder_bookmark_data(
        &self,
        id: String,
        path: String,
    ) -> Result<DataResult, BookmarkError> {
        let fields = [field("id", &id), field("path", &path)];
        run_mobile_call(
            "read-folder-bookmark-data-started",
            "read-folder-bookmark-data-succeeded",
            "read-folder-bookmark-data-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "readFolderBookmarkData",
                        read_folder_bookmark_data_payload(id, path),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Writes text content to a file within a bookmarked folder scope.
    pub async fn write_folder_bookmark(
        &self,
        id: String,
        path: String,
        content: String,
    ) -> Result<(), BookmarkError> {
        let fields = [
            field("id", &id),
            field("path", &path),
            field("content_length", &content.len()),
        ];
        run_mobile_call(
            "write-folder-bookmark-started",
            "write-folder-bookmark-succeeded",
            "write-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "writeFolderBookmark",
                        write_folder_bookmark_payload(id, path, content),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Writes binary content to a file within a bookmarked folder scope.
    pub async fn write_folder_bookmark_data(
        &self,
        id: String,
        path: String,
        data: Vec<u8>,
    ) -> Result<(), BookmarkError> {
        let fields = [
            field("id", &id),
            field("path", &path),
            field("data_length", &data.len()),
        ];
        run_mobile_call(
            "write-folder-bookmark-data-started",
            "write-folder-bookmark-data-succeeded",
            "write-folder-bookmark-data-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "writeFolderBookmarkData",
                        write_folder_bookmark_data_payload(id, path, data),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    // ── Folder-scoped mutations ─────────────────────────────────────

    /// Creates a subdirectory within a bookmarked folder scope.
    pub async fn create_dir(
        &self,
        id: String,
        parent_path: String,
        name: String,
    ) -> Result<Entry, BookmarkError> {
        let fields = [field("id", &id), field("parent_path", &parent_path), field("name", &name)];
        run_mobile_call(
            "create-dir-started",
            "create-dir-succeeded",
            "create-dir-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "createDir",
                        create_dir_payload(id, parent_path, name),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Creates a new file within a bookmarked folder scope.
    pub async fn create_file(
        &self,
        id: String,
        parent_path: String,
        name: String,
        content: Option<String>,
    ) -> Result<Entry, BookmarkError> {
        let content_len = content.as_ref().map(|c| c.len()).unwrap_or(0);
        let fields = [
            field("id", &id),
            field("parent_path", &parent_path),
            field("name", &name),
            field("content_length", &content_len),
        ];
        run_mobile_call(
            "create-file-started",
            "create-file-succeeded",
            "create-file-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "createFile",
                        create_file_payload(id, parent_path, name, content),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Renames a file or directory within a bookmarked folder scope.
    pub async fn rename(
        &self,
        id: String,
        path: String,
        new_name: String,
    ) -> Result<Entry, BookmarkError> {
        let fields = [field("id", &id), field("path", &path), field("new_name", &new_name)];
        run_mobile_call(
            "rename-started",
            "rename-succeeded",
            "rename-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async("rename", rename_payload(id, path, new_name))
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Moves a file or directory to a new parent within the same bookmarked folder scope.
    pub async fn move_entry(
        &self,
        id: String,
        src_path: String,
        dest_parent_path: String,
        name: Option<String>,
    ) -> Result<Entry, BookmarkError> {
        let fields = [
            field("id", &id),
            field("src_path", &src_path),
            field("dest_parent_path", &dest_parent_path),
            field("name", &name),
        ];
        run_mobile_call(
            "move-started",
            "move-succeeded",
            "move-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "move",
                        move_payload(id, src_path, dest_parent_path, name),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Deletes a file or directory within a bookmarked folder scope.
    pub async fn remove(
        &self,
        id: String,
        path: String,
    ) -> Result<(), BookmarkError> {
        let fields = [field("id", &id), field("path", &path)];
        run_mobile_call(
            "remove-started",
            "remove-succeeded",
            "remove-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async("remove", remove_payload(id, path))
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    // ── Lifecycle ───────────────────────────────────────────────────

    /// Validates that a stored bookmark is still usable without performing I/O.
    pub async fn check_bookmark(&self, id: String) -> Result<bool, BookmarkError> {
        let fields = [field("id", &id)];
        run_mobile_call(
            "check-bookmark-started",
            "check-bookmark-succeeded",
            "check-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async("checkBookmark", check_bookmark_payload(id))
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Releases the native security-scoped grant and removes the bookmark from
    /// persistent storage.
    pub async fn release_bookmark(&self, id: String) -> Result<(), BookmarkError> {
        let fields = [field("id", &id)];
        run_mobile_call(
            "release-bookmark-started",
            "release-bookmark-succeeded",
            "release-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "releaseBookmark",
                        release_bookmark_payload(id),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }
}
