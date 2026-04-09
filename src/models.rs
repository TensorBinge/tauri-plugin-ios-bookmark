use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickBookmarkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickFolderBookmarkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_path: Option<String>,
}

/// Returned when the user picks a file and a bookmark is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickResult {
    /// Opaque bookmark identifier (UUID string).
    pub bookmark_id: String,
    /// Display filename (e.g. "notes.md").
    pub file_name: String,
    /// Original file path for display in recent/resume UI.
    pub file_path: String,
    /// Full UTF-8 content of the file.
    pub content: String,
}

/// Returned when the user picks a folder and a bookmark is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickFolderResult {
    /// Opaque bookmark identifier (UUID string).
    pub bookmark_id: String,
    /// Display folder name (e.g. "Docs").
    pub folder_name: String,
    /// Original folder path for display and matching.
    pub folder_path: String,
}

/// Returned when resolving a bookmark and reading its content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResult {
    pub file_name: String,
    pub file_path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("security-scoped bookmarks are not supported on this platform")]
    Unsupported,
    #[error("bookmark not found: {0}")]
    NotFound(String),
    #[error("bookmark is stale — user must re-pick the file")]
    Stale,
    #[error("access denied")]
    PermissionDenied,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("cancelled")]
    Cancelled,
    #[error("selected file does not match requested target")]
    TargetMismatch,
    #[error("native error: {0}")]
    Native(String),
}

impl serde::Serialize for BookmarkError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
