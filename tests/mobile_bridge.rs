use serde_json::json;
use tauri_plugin_ios_bookmark::{
    normalize_ios_bookmark_error, pick_and_bookmark_payload, pick_folder_and_bookmark_payload,
    BookmarkError, PickBookmarkRequest, PickFolderBookmarkRequest,
};

#[test]
fn pick_and_bookmark_payload_uses_null_for_no_request() {
    assert_eq!(pick_and_bookmark_payload(None), serde_json::Value::Null);
}

#[test]
fn pick_and_bookmark_payload_wraps_request_when_target_path_is_present() {
    assert_eq!(
        pick_and_bookmark_payload(Some(PickBookmarkRequest {
            target_path: Some("/docs/related.md".into()),
            suggested_file_name: None,
        })),
        json!({
            "request": {
                "targetPath": "/docs/related.md"
            }
        })
    );
}

#[test]
fn pick_folder_and_bookmark_payload_uses_null_for_no_request() {
    assert_eq!(pick_folder_and_bookmark_payload(None), serde_json::Value::Null);
}

#[test]
fn pick_folder_and_bookmark_payload_wraps_request_when_target_path_is_present() {
    assert_eq!(
        pick_folder_and_bookmark_payload(Some(PickFolderBookmarkRequest {
            target_path: Some("/docs".into()),
        })),
        json!({
            "request": {
                "targetPath": "/docs"
            }
        })
    );
}

#[test]
fn normalize_ios_bookmark_error_preserves_target_mismatch() {
    assert!(matches!(
        normalize_ios_bookmark_error(
            "BOOKMARK_ERROR:TARGET_MISMATCH:Selected file does not match requested target".into()
        ),
        BookmarkError::TargetMismatch
    ));
}
