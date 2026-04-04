use crate::models::*;
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<IosBookmark<R>, BookmarkError> {
    Ok(IosBookmark(std::marker::PhantomData))
}

/// Desktop stub — all operations return Unsupported.
pub struct IosBookmark<R: Runtime>(std::marker::PhantomData<R>);

// Safety: PhantomData<R> itself holds no data; R is only a type-level marker.
unsafe impl<R: Runtime> Send for IosBookmark<R> {}
unsafe impl<R: Runtime> Sync for IosBookmark<R> {}

impl<R: Runtime> IosBookmark<R> {
    pub async fn pick_and_bookmark(
        &self,
        _request: Option<PickBookmarkRequest>,
    ) -> Result<PickResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn pick_folder_and_bookmark(
        &self,
        _request: Option<PickFolderBookmarkRequest>,
    ) -> Result<PickFolderResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn read_by_bookmark(&self, _id: String) -> Result<ReadResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn read_by_folder_bookmark(
        &self,
        _id: String,
        _target_path: String,
    ) -> Result<ReadResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn forget_bookmark(&self, _id: String) -> Result<(), BookmarkError> {
        Err(BookmarkError::Unsupported)
    }
}
