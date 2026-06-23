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
    // ── File bookmark stubs ─────────────────────────────────────────

    pub async fn pick_file_bookmark(
        &self,
        _request: Option<PickFileBookmarkRequest>,
    ) -> Result<Option<FileBookmarkResult>, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn read_file_bookmark(&self, _id: String) -> Result<ReadResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn read_file_bookmark_data(&self, _id: String) -> Result<DataResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn write_file_bookmark(
        &self,
        _id: String,
        _content: String,
    ) -> Result<(), BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn write_file_bookmark_data(
        &self,
        _id: String,
        _data: Vec<u8>,
    ) -> Result<(), BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    // ── Folder bookmark stubs ───────────────────────────────────────

    pub async fn pick_folder_bookmark(
        &self,
        _request: Option<PickFolderBookmarkRequest>,
    ) -> Result<Option<FolderBookmarkResult>, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn list_folder_bookmark(
        &self,
        _id: String,
        _path: String,
    ) -> Result<Vec<Entry>, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn read_folder_bookmark(
        &self,
        _id: String,
        _path: String,
    ) -> Result<ReadResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn read_folder_bookmark_data(
        &self,
        _id: String,
        _path: String,
    ) -> Result<DataResult, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn write_folder_bookmark(
        &self,
        _id: String,
        _path: String,
        _content: String,
    ) -> Result<(), BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn write_folder_bookmark_data(
        &self,
        _id: String,
        _path: String,
        _data: Vec<u8>,
    ) -> Result<(), BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    // ── Folder-scoped mutation stubs ────────────────────────────────

    pub async fn create_dir(
        &self,
        _id: String,
        _parent_path: String,
        _name: String,
    ) -> Result<Entry, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn create_file(
        &self,
        _id: String,
        _parent_path: String,
        _name: String,
        _content: Option<String>,
    ) -> Result<Entry, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn rename(
        &self,
        _id: String,
        _path: String,
        _new_name: String,
    ) -> Result<Entry, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn move_entry(
        &self,
        _id: String,
        _src_path: String,
        _dest_parent_path: String,
        _name: Option<String>,
    ) -> Result<Entry, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn remove(&self, _id: String, _path: String) -> Result<(), BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    // ── Lifecycle stubs ─────────────────────────────────────────────

    pub async fn check_bookmark(&self, _id: String) -> Result<bool, BookmarkError> {
        Err(BookmarkError::Unsupported)
    }

    pub async fn release_bookmark(&self, _id: String) -> Result<(), BookmarkError> {
        Err(BookmarkError::Unsupported)
    }
}
