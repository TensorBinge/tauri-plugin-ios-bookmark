//! iOS security-scoped bookmark plugin for Tauri 2.
//!
//! This crate exposes a Tauri plugin that bridges to native iOS code for:
//! - presenting the Files picker
//! - creating and storing security-scoped bookmarks
//! - reading a bookmarked file later
//! - forgetting a stored bookmark
//!
//! The plugin is intended for Tauri mobile apps that target iOS. On unsupported
//! platforms, initialization falls back to an unsupported implementation.
//!
//! # Setup
//!
//! Register the plugin in your Tauri application:
//!
//! ```rust,ignore
//! tauri::Builder::default()
//!     .plugin(tauri_plugin_ios_bookmark::init())
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```
//!
//! Then call the guest API from JavaScript or TypeScript to pick, read, and
//! forget bookmarked files.

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error_bridge;
mod models;
mod payloads;

pub use error_bridge::normalize_ios_bookmark_error;
pub use models::*;
pub use payloads::{pick_and_bookmark_payload, pick_folder_and_bookmark_payload};

#[cfg(desktop)]
pub use desktop::IosBookmark;
#[cfg(mobile)]
pub use mobile::IosBookmark;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

/// Registers the iOS bookmark plugin and stores its runtime handle in managed state.
///
/// The resulting state is consumed by the Tauri commands in `commands.rs`, which
/// keeps the JavaScript-facing command surface small while allowing the platform-
/// specific implementation to live behind `IosBookmark`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ios-bookmark")
        .invoke_handler(tauri::generate_handler![
            commands::pick_and_bookmark,
            commands::pick_folder_and_bookmark,
            commands::read_by_bookmark,
            commands::read_by_folder_bookmark,
            commands::export_file,
            commands::export_pdf,
            commands::forget_bookmark,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let bookmark = mobile::init(app, api)?;
            #[cfg(desktop)]
            let bookmark = desktop::init(app, api)?;

            // Expose a single managed handle so every command shares the same plugin instance.
            app.manage(bookmark);
            Ok(())
        })
        .build()
}
