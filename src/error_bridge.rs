use crate::models::BookmarkError;

const BOOKMARK_NATIVE_ERROR_PREFIX: &str = "BOOKMARK_ERROR:";

#[doc(hidden)]
pub fn normalize_ios_bookmark_error(message: String) -> BookmarkError {
    let Some(rest) = message.strip_prefix(BOOKMARK_NATIVE_ERROR_PREFIX) else {
        return BookmarkError::Native(message);
    };

    let mut parts = rest.splitn(2, ':');
    let code = parts.next().unwrap_or_default();
    let detail = parts.next().unwrap_or_default().to_string();

    match code {
        "UNSUPPORTED" => BookmarkError::Unsupported,
        "NOT_FOUND" => BookmarkError::NotFound(detail),
        "STALE" => BookmarkError::Stale,
        "PERMISSION_DENIED" => BookmarkError::PermissionDenied,
        "IO_ERROR" => BookmarkError::Io(detail),
        "CANCELLED" => BookmarkError::Cancelled,
        "TARGET_MISMATCH" => BookmarkError::TargetMismatch,
        "NATIVE_ERROR" => BookmarkError::Native(detail),
        _ => BookmarkError::Native(message),
    }
}