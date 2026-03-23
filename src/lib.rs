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
