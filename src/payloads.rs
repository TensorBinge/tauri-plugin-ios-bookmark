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
