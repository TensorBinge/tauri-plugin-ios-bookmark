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
mod models;

pub use models::*;

#[cfg(desktop)]
pub(crate) use desktop::IosBookmark;
#[cfg(mobile)]
pub(crate) use mobile::IosBookmark;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ios-bookmark")
        .invoke_handler(tauri::generate_handler![
            commands::pick_and_bookmark,
            commands::read_by_bookmark,
            commands::forget_bookmark,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let bookmark = mobile::init(app, api)?;
            #[cfg(desktop)]
            let bookmark = desktop::init(app, api)?;
            app.manage(bookmark);
            Ok(())
        })
        .build()
}
