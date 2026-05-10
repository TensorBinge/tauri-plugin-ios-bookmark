//! Mobile runtime implementation for the iOS bookmark plugin.
//!
//! Tauri exposes mobile plugins through a `PluginHandle`; this module wraps that
//! handle in a small Rust API so command handlers can stay platform-agnostic.

use crate::{
    create_folder_by_folder_bookmark_payload, create_markdown_file_by_folder_bookmark_payload,
    list_by_folder_bookmark_payload, models::*, move_by_folder_bookmark_payload,
    normalize_ios_bookmark_error, pick_and_bookmark_payload, pick_folder_and_bookmark_payload,
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
    /// Presents the file picker, creates a security-scoped bookmark, and returns the first file.
    pub async fn pick_and_bookmark(
        &self,
        request: Option<PickBookmarkRequest>,
    ) -> Result<Option<PickResult>, BookmarkError> {
        let fields = [field("has_request", &request.is_some())];
        run_mobile_call(
            "pick-and-bookmark-started",
            "pick-and-bookmark-succeeded",
            "pick-and-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async("pickAndBookmark", pick_and_bookmark_payload(request))
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Presents the folder picker and stores a bookmark that can authorize descendant reads.
    pub async fn pick_folder_and_bookmark(
        &self,
        request: Option<PickFolderBookmarkRequest>,
    ) -> Result<Option<PickFolderResult>, BookmarkError> {
        let fields = [field("has_request", &request.is_some())];
        run_mobile_call(
            "pick-folder-and-bookmark-started",
            "pick-folder-and-bookmark-succeeded",
            "pick-folder-and-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "pickFolderAndBookmark",
                        pick_folder_and_bookmark_payload(request),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Lists directory entries inside a previously authorized folder bookmark scope.
    pub async fn list_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
    ) -> Result<Vec<FolderBookmarkEntry>, BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("target_path", &target_path),
        ];
        run_mobile_call(
            "list-by-folder-bookmark-started",
            "list-by-folder-bookmark-succeeded",
            "list-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "listByFolderBookmark",
                        list_by_folder_bookmark_payload(id, target_path),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Creates a child folder inside a previously authorized folder bookmark scope.
    pub async fn create_folder_by_folder_bookmark(
        &self,
        id: String,
        parent_path: String,
        name: String,
    ) -> Result<FolderBookmarkEntry, BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("parent_path", &parent_path),
            field("name", &name),
        ];
        run_mobile_call(
            "create-folder-by-folder-bookmark-started",
            "create-folder-by-folder-bookmark-succeeded",
            "create-folder-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "createFolderByFolderBookmark",
                        create_folder_by_folder_bookmark_payload(id, parent_path, name),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Creates a markdown file inside a previously authorized folder bookmark scope.
    pub async fn create_markdown_file_by_folder_bookmark(
        &self,
        id: String,
        parent_path: String,
        name: String,
        content: String,
    ) -> Result<FolderBookmarkEntry, BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("parent_path", &parent_path),
            field("name", &name),
            field("content_length", &content.len()),
        ];
        run_mobile_call(
            "create-markdown-file-by-folder-bookmark-started",
            "create-markdown-file-by-folder-bookmark-succeeded",
            "create-markdown-file-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "createMarkdownFileByFolderBookmark",
                        create_markdown_file_by_folder_bookmark_payload(
                            id,
                            parent_path,
                            name,
                            content,
                        ),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Renames a file or folder inside a previously authorized folder bookmark scope.
    pub async fn rename_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
        name: String,
    ) -> Result<FolderBookmarkEntry, BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("target_path", &target_path),
            field("name", &name),
        ];
        run_mobile_call(
            "rename-by-folder-bookmark-started",
            "rename-by-folder-bookmark-succeeded",
            "rename-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "renameByFolderBookmark",
                        serde_json::json!({ "id": id, "targetPath": target_path, "name": name }),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Moves a file or folder inside a previously authorized folder bookmark scope.
    pub async fn move_by_folder_bookmark(
        &self,
        id: String,
        source_path: String,
        destination_parent_path: String,
        name: String,
    ) -> Result<FolderBookmarkEntry, BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("source_path", &source_path),
            field("destination_parent_path", &destination_parent_path),
            field("name", &name),
        ];
        run_mobile_call(
            "move-by-folder-bookmark-started",
            "move-by-folder-bookmark-succeeded",
            "move-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "moveByFolderBookmark",
                        move_by_folder_bookmark_payload(
                            id,
                            source_path,
                            destination_parent_path,
                            name,
                        ),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Deletes a file or folder inside a previously authorized folder bookmark scope.
    pub async fn delete_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
    ) -> Result<(), BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("target_path", &target_path),
        ];
        run_mobile_call(
            "delete-by-folder-bookmark-started",
            "delete-by-folder-bookmark-succeeded",
            "delete-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "deleteByFolderBookmark",
                        serde_json::json!({ "id": id, "targetPath": target_path }),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Reads a previously bookmarked file directly by bookmark id.
    pub async fn read_by_bookmark(&self, id: String) -> Result<ReadResult, BookmarkError> {
        let fields = [field("bookmark_id", &id)];
        run_mobile_call(
            "read-by-bookmark-started",
            "read-by-bookmark-succeeded",
            "read-by-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async("readByBookmark", serde_json::json!({ "id": id }))
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Writes a previously bookmarked file directly by bookmark id.
    pub async fn write_by_bookmark(
        &self,
        id: String,
        content: String,
    ) -> Result<(), BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("content_length", &content.len()),
        ];
        run_mobile_call(
            "write-by-bookmark-started",
            "write-by-bookmark-succeeded",
            "write-by-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "writeByBookmark",
                        serde_json::json!({ "id": id, "content": content }),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Reads a descendant file by combining a folder bookmark id with the target path.
    pub async fn read_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
    ) -> Result<ReadResult, BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("target_path", &target_path),
        ];
        run_mobile_call(
            "read-by-folder-bookmark-started",
            "read-by-folder-bookmark-succeeded",
            "read-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "readByFolderBookmark",
                        serde_json::json!({ "id": id, "targetPath": target_path }),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Reads a descendant binary file by combining a folder bookmark id with the target path.
    pub async fn read_binary_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
    ) -> Result<BinaryReadResult, BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("target_path", &target_path),
        ];
        run_mobile_call(
            "read-binary-by-folder-bookmark-started",
            "read-binary-by-folder-bookmark-succeeded",
            "read-binary-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "readBinaryByFolderBookmark",
                        serde_json::json!({ "id": id, "targetPath": target_path }),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Writes a descendant file by combining a folder bookmark id with the target path.
    pub async fn write_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
        content: String,
    ) -> Result<(), BookmarkError> {
        let fields = [
            field("bookmark_id", &id),
            field("target_path", &target_path),
            field("content_length", &content.len()),
        ];
        run_mobile_call(
            "write-by-folder-bookmark-started",
            "write-by-folder-bookmark-succeeded",
            "write-by-folder-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "writeByFolderBookmark",
                        serde_json::json!({ "id": id, "targetPath": target_path, "content": content }),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Forgets a stored bookmark so the native side can release the security-scoped grant.
    pub async fn forget_bookmark(&self, id: String) -> Result<(), BookmarkError> {
        let fields = [field("bookmark_id", &id)];
        run_mobile_call(
            "forget-bookmark-started",
            "forget-bookmark-succeeded",
            "forget-bookmark-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async("forgetBookmark", serde_json::json!({ "id": id }))
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Presents the native export picker for a file that already exists in the app sandbox.
    pub async fn export_file(&self, path: String) -> Result<(), BookmarkError> {
        let fields = [field("path", &path)];
        run_mobile_call(
            "export-file-started",
            "export-file-succeeded",
            "export-file-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async("exportFile", serde_json::json!({ "path": path }))
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }

    /// Renders HTML to a temporary PDF and presents the native export picker for it.
    pub async fn export_pdf(
        &self,
        file_name: String,
        html: String,
        toc: Vec<ExportTocEntry>,
    ) -> Result<(), BookmarkError> {
        let fields = [
            field("file_name", &file_name),
            field("html_length", &html.len()),
            field("toc_length", &toc.len()),
        ];
        run_mobile_call(
            "export-pdf-started",
            "export-pdf-succeeded",
            "export-pdf-failed",
            &fields,
            async {
                self.0
                    .run_mobile_plugin_async(
                        "exportPdf",
                        serde_json::json!({ "fileName": file_name, "html": html, "toc": toc }),
                    )
                    .await
                    .map_err(|e| normalize_ios_bookmark_error(e.to_string()))
            },
        )
        .await
    }
}
