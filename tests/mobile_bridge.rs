use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_ios_bookmark::{
    create_folder_by_folder_bookmark_payload, create_markdown_file_by_folder_bookmark_payload,
    list_by_folder_bookmark_payload, normalize_ios_bookmark_error, pick_and_bookmark_payload,
    pick_folder_and_bookmark_payload, BookmarkError, PickBookmarkRequest,
    PickFolderBookmarkRequest,
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
    assert_eq!(
        pick_folder_and_bookmark_payload(None),
        serde_json::Value::Null
    );
}

#[test]
fn pick_folder_and_bookmark_payload_wraps_request_when_target_path_is_present() {
    assert_eq!(
        pick_folder_and_bookmark_payload(Some(PickFolderBookmarkRequest {
            target_path: Some("/docs".into()),
            require_empty: None,
        })),
        json!({
            "request": {
                "targetPath": "/docs"
            }
        })
    );
}

#[test]
fn pick_folder_and_bookmark_payload_wraps_require_empty_when_present() {
    assert_eq!(
        pick_folder_and_bookmark_payload(Some(PickFolderBookmarkRequest {
            target_path: None,
            require_empty: Some(true),
        })),
        json!({
            "request": {
                "requireEmpty": true
            }
        })
    );
}

#[test]
fn list_by_folder_bookmark_payload_uses_root_fields() {
    assert_eq!(
        list_by_folder_bookmark_payload("folder-123".into(), "/docs".into()),
        json!({
            "id": "folder-123",
            "targetPath": "/docs"
        })
    );
}

#[test]
fn create_folder_by_folder_bookmark_payload_uses_root_fields() {
    assert_eq!(
        create_folder_by_folder_bookmark_payload(
            "folder-123".into(),
            "/docs".into(),
            "notes".into()
        ),
        json!({
            "id": "folder-123",
            "parentPath": "/docs",
            "name": "notes"
        })
    );
}

#[test]
fn create_markdown_file_by_folder_bookmark_payload_uses_root_fields() {
    assert_eq!(
        create_markdown_file_by_folder_bookmark_payload(
            "folder-123".into(),
            "/docs".into(),
            "notes.md".into(),
            "# Notes".into(),
        ),
        json!({
            "id": "folder-123",
            "parentPath": "/docs",
            "name": "notes.md",
            "content": "# Notes"
        })
    );
}

#[test]
fn plugin_metadata_includes_rename_and_delete_commands() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let build_rs = fs::read_to_string(root.join("build.rs")).expect("read build.rs");
    let default_permissions = fs::read_to_string(root.join("permissions/default.toml"))
        .expect("read permissions/default.toml");

    assert!(build_rs.contains("rename_by_folder_bookmark"));
    assert!(build_rs.contains("delete_by_folder_bookmark"));
    assert!(build_rs.contains("read_binary_by_folder_bookmark"));
    assert!(default_permissions.contains("allow-rename-by-folder-bookmark"));
    assert!(default_permissions.contains("allow-delete-by-folder-bookmark"));
    assert!(default_permissions.contains("allow-read-binary-by-folder-bookmark"));
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

#[test]
fn normalize_ios_bookmark_error_preserves_folder_not_empty() {
    assert!(matches!(
        normalize_ios_bookmark_error(
            "BOOKMARK_ERROR:FOLDER_NOT_EMPTY:Selected folder must be empty".into()
        ),
        BookmarkError::FolderNotEmpty
    ));
}
