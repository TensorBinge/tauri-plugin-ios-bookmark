use crate::models::{PickBookmarkRequest, PickFolderBookmarkRequest};

#[doc(hidden)]
pub fn pick_and_bookmark_payload(request: Option<PickBookmarkRequest>) -> serde_json::Value {
    match request {
        Some(request) => serde_json::json!({ "request": request }),
        None => serde_json::Value::Null,
    }
}

#[doc(hidden)]
pub fn pick_folder_and_bookmark_payload(
    request: Option<PickFolderBookmarkRequest>,
) -> serde_json::Value {
    match request {
        Some(request) => serde_json::json!({ "request": request }),
        None => serde_json::Value::Null,
    }
}

#[doc(hidden)]
pub fn list_by_folder_bookmark_payload(id: String, target_path: String) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "targetPath": target_path,
    })
}

#[doc(hidden)]
pub fn create_folder_by_folder_bookmark_payload(
    id: String,
    parent_path: String,
    name: String,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "parentPath": parent_path,
        "name": name,
    })
}

#[doc(hidden)]
pub fn create_markdown_file_by_folder_bookmark_payload(
    id: String,
    parent_path: String,
    name: String,
    content: String,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "parentPath": parent_path,
        "name": name,
        "content": content,
    })
}
