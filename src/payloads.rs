use crate::models::PickBookmarkRequest;

#[doc(hidden)]
pub fn pick_and_bookmark_payload(request: Option<PickBookmarkRequest>) -> serde_json::Value {
    match request {
        Some(request) => serde_json::json!({ "request": request }),
        None => serde_json::Value::Null,
    }
}
