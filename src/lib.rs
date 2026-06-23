//! iOS security-scoped bookmark plugin for Tauri 2.
//!
//! This crate exposes a Tauri plugin that bridges to native iOS code for:
//! - presenting the Files picker for files and folders
//! - creating and storing security-scoped bookmarks
//! - reading and writing bookmarked files (text and binary)
//! - listing, creating, renaming, moving, and removing items within a folder scope
//! - validating and releasing stored bookmarks
//!
//! The plugin is intended for Tauri mobile apps that target iOS. On unsupported
//! platforms, every operation returns `BookmarkError::Unsupported`.
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
//! Then call the guest API from JavaScript or TypeScript to work with bookmarked
//! files and folders.

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error_bridge;
mod logging;
mod models;
mod payloads;

pub use error_bridge::normalize_ios_bookmark_error;
pub use models::*;
pub use payloads::{
    check_bookmark_payload, create_dir_payload, create_file_payload,
    list_folder_bookmark_payload, move_payload, pick_file_bookmark_payload,
    pick_folder_bookmark_payload, read_file_bookmark_data_payload,
    read_file_bookmark_payload, read_folder_bookmark_data_payload,
    read_folder_bookmark_payload, release_bookmark_payload, remove_payload,
    rename_payload, write_file_bookmark_data_payload, write_file_bookmark_payload,
    write_folder_bookmark_data_payload, write_folder_bookmark_payload,
};

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
            // File bookmarks
            commands::pick_file_bookmark,
            commands::read_file_bookmark,
            commands::read_file_bookmark_data,
            commands::write_file_bookmark,
            commands::write_file_bookmark_data,
            // Folder bookmarks
            commands::pick_folder_bookmark,
            commands::list_folder_bookmark,
            commands::read_folder_bookmark,
            commands::read_folder_bookmark_data,
            commands::write_folder_bookmark,
            commands::write_folder_bookmark_data,
            // Folder-scoped mutations
            commands::create_dir,
            commands::create_file,
            commands::rename,
            commands::move_entry,
            commands::remove,
            // Lifecycle
            commands::check_bookmark,
            commands::release_bookmark,
        ])
        .setup(|app, api| {
            crate::plugin_log_info!("ios-bookmark.plugin", "setup-started");
            #[cfg(mobile)]
            let bookmark = mobile::init(app, api)?;
            #[cfg(desktop)]
            let bookmark = desktop::init(app, api)?;

            // Expose a single managed handle so every command shares the same plugin instance.
            app.manage(bookmark);
            crate::plugin_log_info!("ios-bookmark.plugin", "setup-succeeded");
            Ok(())
        })
        .build()
}
