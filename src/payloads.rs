use crate::models::{PickFileBookmarkRequest, PickFolderBookmarkRequest};

/// Payload constructors for every native bridge call.
///
/// These keep JSON key conventions in one place and ensure every `mobile.rs`
/// call site uses the same serialization contract.

// ── File bookmark payloads ──────────────────────────────────────────

#[doc(hidden)]
pub fn pick_file_bookmark_payload(request: Option<PickFileBookmarkRequest>) -> serde_json::Value {
    match request {
        Some(request) => serde_json::json!({ "request": request }),
        None => serde_json::Value::Null,
    }
}

#[doc(hidden)]
pub fn read_file_bookmark_payload(id: String) -> serde_json::Value {
    serde_json::json!({ "id": id })
}

#[doc(hidden)]
pub fn read_file_bookmark_data_payload(id: String) -> serde_json::Value {
    serde_json::json!({ "id": id })
}

#[doc(hidden)]
pub fn write_file_bookmark_payload(id: String, content: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "content": content })
}

#[doc(hidden)]
pub fn write_file_bookmark_data_payload(id: String, data: Vec<u8>) -> serde_json::Value {
    serde_json::json!({ "id": id, "data": data })
}

// ── Folder bookmark payloads ────────────────────────────────────────

#[doc(hidden)]
pub fn pick_folder_bookmark_payload(
    request: Option<PickFolderBookmarkRequest>,
) -> serde_json::Value {
    match request {
        Some(request) => serde_json::json!({ "request": request }),
        None => serde_json::Value::Null,
    }
}

#[doc(hidden)]
pub fn list_folder_bookmark_payload(id: String, path: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "path": path })
}

#[doc(hidden)]
pub fn read_folder_bookmark_payload(id: String, path: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "path": path })
}

#[doc(hidden)]
pub fn read_folder_bookmark_data_payload(id: String, path: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "path": path })
}

#[doc(hidden)]
pub fn write_folder_bookmark_payload(id: String, path: String, content: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "path": path, "content": content })
}

#[doc(hidden)]
pub fn write_folder_bookmark_data_payload(
    id: String,
    path: String,
    data: Vec<u8>,
) -> serde_json::Value {
    serde_json::json!({ "id": id, "path": path, "data": data })
}

// ── Folder-scoped mutation payloads ─────────────────────────────────

#[doc(hidden)]
pub fn create_dir_payload(id: String, parent_path: String, name: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "parentPath": parent_path, "name": name })
}

#[doc(hidden)]
pub fn create_file_payload(
    id: String,
    parent_path: String,
    name: String,
    content: Option<String>,
) -> serde_json::Value {
    match content {
        Some(text) => serde_json::json!({ "id": id, "parentPath": parent_path, "name": name, "content": text }),
        None => serde_json::json!({ "id": id, "parentPath": parent_path, "name": name }),
    }
}

#[doc(hidden)]
pub fn rename_payload(id: String, path: String, new_name: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "path": path, "newName": new_name })
}

#[doc(hidden)]
pub fn move_payload(
    id: String,
    src_path: String,
    dest_parent_path: String,
    name: Option<String>,
) -> serde_json::Value {
    serde_json::json!({ "id": id, "srcPath": src_path, "destParentPath": dest_parent_path, "name": name })
}

#[doc(hidden)]
pub fn remove_payload(id: String, path: String) -> serde_json::Value {
    serde_json::json!({ "id": id, "path": path })
}

// ── Lifecycle payloads ──────────────────────────────────────────────

#[doc(hidden)]
pub fn check_bookmark_payload(id: String) -> serde_json::Value {
    serde_json::json!({ "id": id })
}

#[doc(hidden)]
pub fn release_bookmark_payload(id: String) -> serde_json::Value {
    serde_json::json!({ "id": id })
}
