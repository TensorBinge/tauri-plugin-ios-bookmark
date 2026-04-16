use crate::models::BookmarkError;
use log::Level;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct LogRecord<'a> {
    event: &'a str,
    layer: &'static str,
    target: &'a str,
    status: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_kind: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attrs: Option<Value>,
}

fn emit(
    level: Level,
    target: &str,
    event: &str,
    status: &str,
    error_kind: Option<&str>,
    attrs: Value,
) {
    let attrs = if attrs.is_null() { None } else { Some(attrs) };
    let message = serde_json::to_string(&LogRecord {
        event,
        layer: "rust-plugin",
        target,
        status,
        error_kind,
        attrs,
    })
    .unwrap_or_else(|serialization_error| {
        format!(
            "{{\"event\":\"logging.serialization_error\",\"layer\":\"rust-plugin\",\"target\":\"{}\",\"status\":\"failure\",\"errorKind\":\"{}\"}}",
            target, serialization_error
        )
    });

    match level {
        Level::Error => log::error!(target: target, "{}", message),
        Level::Warn => log::warn!(target: target, "{}", message),
        Level::Info => log::info!(target: target, "{}", message),
        Level::Debug => log::debug!(target: target, "{}", message),
        Level::Trace => log::trace!(target: target, "{}", message),
    }
}

pub fn info(target: &str, event: &str, status: &str, attrs: Value) {
    emit(Level::Info, target, event, status, None, attrs);
}

pub fn error(target: &str, event: &str, status: &str, error_kind: Option<&str>, attrs: Value) {
    emit(Level::Error, target, event, status, error_kind, attrs);
}

pub fn bookmark_error_kind(error: &BookmarkError) -> &'static str {
    match error {
        BookmarkError::Unsupported => "Unsupported",
        BookmarkError::NotFound(_) => "NotFound",
        BookmarkError::Stale => "Stale",
        BookmarkError::PermissionDenied => "PermissionDenied",
        BookmarkError::Io(_) => "Io",
        BookmarkError::Cancelled => "Cancelled",
        BookmarkError::TargetMismatch => "TargetMismatch",
        BookmarkError::Native(_) => "Native",
    }
}
