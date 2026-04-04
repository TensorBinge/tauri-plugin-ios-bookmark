use crate::{
    models::*, normalize_ios_bookmark_error, pick_and_bookmark_payload,
    pick_folder_and_bookmark_payload,
};
use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_ios_bookmark);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<IosBookmark<R>, BookmarkError> {
    #[cfg(target_os = "ios")]
    let handle = api
        .register_ios_plugin(init_plugin_ios_bookmark)
        .map_err(|e| BookmarkError::Native(e.to_string()))?;

    // Android is not supported; this plugin is iOS-only.
    #[cfg(not(target_os = "ios"))]
    let handle = {
        let _ = api;
        return Err(BookmarkError::Unsupported);
    };

    Ok(IosBookmark(handle))
}

pub struct IosBookmark<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> IosBookmark<R> {
    pub async fn pick_and_bookmark(
        &self,
        request: Option<PickBookmarkRequest>,
    ) -> Result<PickResult, BookmarkError> {
        println!("[ios-bookmark] rust mobile bridge: pickAndBookmark -> start");
        self.0
            .run_mobile_plugin_async("pickAndBookmark", pick_and_bookmark_payload(request))
            .await
            .map(|result| {
                println!("[ios-bookmark] rust mobile bridge: pickAndBookmark -> resolved");
                result
            })
            .map_err(|e| {
                println!("[ios-bookmark] rust mobile bridge: pickAndBookmark -> error: {e}");
                normalize_ios_bookmark_error(e.to_string())
            })
    }

    pub async fn pick_folder_and_bookmark(
        &self,
        request: Option<PickFolderBookmarkRequest>,
    ) -> Result<PickFolderResult, BookmarkError> {
        println!("[ios-bookmark] rust mobile bridge: pickFolderAndBookmark -> start");
        self.0
            .run_mobile_plugin_async("pickFolderAndBookmark", pick_folder_and_bookmark_payload(request))
            .await
            .map(|result| {
                println!("[ios-bookmark] rust mobile bridge: pickFolderAndBookmark -> resolved");
                result
            })
            .map_err(|e| {
                println!("[ios-bookmark] rust mobile bridge: pickFolderAndBookmark -> error: {e}");
                normalize_ios_bookmark_error(e.to_string())
            })
    }

    pub async fn read_by_bookmark(&self, id: String) -> Result<ReadResult, BookmarkError> {
        println!("[ios-bookmark] rust mobile bridge: readByBookmark({id}) -> start");
        self.0
            .run_mobile_plugin_async("readByBookmark", serde_json::json!({ "id": id }))
            .await
            .map(|result| {
                println!("[ios-bookmark] rust mobile bridge: readByBookmark -> resolved");
                result
            })
            .map_err(|e| {
                println!("[ios-bookmark] rust mobile bridge: readByBookmark -> error: {e}");
                normalize_ios_bookmark_error(e.to_string())
            })
    }

    pub async fn read_by_folder_bookmark(
        &self,
        id: String,
        target_path: String,
    ) -> Result<ReadResult, BookmarkError> {
        println!("[ios-bookmark] rust mobile bridge: readByFolderBookmark({id}, {target_path}) -> start");
        self.0
            .run_mobile_plugin_async(
                "readByFolderBookmark",
                serde_json::json!({ "id": id, "targetPath": target_path }),
            )
            .await
            .map(|result| {
                println!("[ios-bookmark] rust mobile bridge: readByFolderBookmark -> resolved");
                result
            })
            .map_err(|e| {
                println!("[ios-bookmark] rust mobile bridge: readByFolderBookmark -> error: {e}");
                normalize_ios_bookmark_error(e.to_string())
            })
    }

    pub async fn forget_bookmark(&self, id: String) -> Result<(), BookmarkError> {
        println!("[ios-bookmark] rust mobile bridge: forgetBookmark({id}) -> start");
        self.0
            .run_mobile_plugin_async("forgetBookmark", serde_json::json!({ "id": id }))
            .await
            .map(|result| {
                println!("[ios-bookmark] rust mobile bridge: forgetBookmark -> resolved");
                result
            })
            .map_err(|e| {
                println!("[ios-bookmark] rust mobile bridge: forgetBookmark -> error: {e}");
                normalize_ios_bookmark_error(e.to_string())
            })
    }
}
