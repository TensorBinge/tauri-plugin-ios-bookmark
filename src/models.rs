use serde::{Deserialize, Serialize};

/// Options for picking a single file through the native file picker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickFileBookmarkRequest {
    /// Hint for the native picker's initial directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_name: Option<String>,
    /// When true, the returned `FileBookmarkResult.content` is empty. Use this
    /// to avoid loading large files into memory during the pick step.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_content: Option<bool>,
}

/// Options for picking a folder through the native folder picker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickFolderBookmarkRequest {
    /// Hint for the native picker's initial directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_name: Option<String>,
    /// When true, rejects folders that are not empty. Default false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_empty: Option<bool>,
}

/// Returned when the user picks a file and a security-scoped bookmark is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileBookmarkResult {
    /// Opaque bookmark identifier (UUID string).
    pub id: String,
    /// Display filename (e.g. "notes.md").
    pub name: String,
    /// Full file path for display in recent/resume UI.
    pub path: String,
    /// File content as UTF-8 text. Empty string when `skip_content` was set.
    pub content: String,
    /// Detected MIME type. May be absent for plain-text or unknown types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Returned when the user picks a folder and a security-scoped bookmark is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderBookmarkResult {
    /// Opaque bookmark identifier (UUID string).
    pub id: String,
    /// Display folder name (e.g. "Documents").
    pub name: String,
    /// Full folder path for display and matching.
    pub path: String,
}

/// Returned when reading the text content of a bookmarked file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResult {
    /// Display filename.
    pub name: String,
    /// Full file path.
    pub path: String,
    /// File content as UTF-8 text.
    pub content: String,
}

/// Returned when reading the binary content of a bookmarked file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataResult {
    /// Display filename.
    pub name: String,
    /// Full file path.
    pub path: String,
    /// Detected MIME type.
    pub mime_type: String,
    /// File content as base64-encoded string.
    pub base64_data: String,
}

/// A file or directory entry within a bookmarked folder scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Display name of the file or directory.
    pub name: String,
    /// Full path within the bookmark scope.
    pub path: String,
    /// Whether this entry is a directory.
    pub is_dir: bool,
    /// File size in bytes. Absent for directories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Last modification time as Unix timestamp in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<u64>,
}

/// Argument struct for folder-scoped write operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFolderBookmarkArgs {
    pub id: String,
    pub path: String,
    pub content: String,
}

/// Errors returned by bookmark operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BookmarkError {
    #[error("security-scoped bookmarks are not supported on this platform")]
    Unsupported,
    #[error("bookmark not found: {0}")]
    NotFound(String),
    #[error("bookmark is stale — user must re-pick the resource")]
    Stale,
    #[error("access denied")]
    PermissionDenied,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("cancelled")]
    Cancelled,
    #[error("selected resource does not match requested target")]
    TargetMismatch,
    #[error("selected folder must be empty")]
    FolderNotEmpty,
    #[error("native error: {0}")]
    Native(String),
}

impl serde::Serialize for BookmarkError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
