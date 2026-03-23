# tauri-plugin-ios-bookmark

iOS security-scoped bookmark plugin for Tauri 2.

This plugin provides a native bridge for opening files from the iOS Files app,
creating persistent security-scoped bookmarks, reading bookmarked files later,
and forgetting saved bookmarks.

## Features

- `pickAndBookmark`: presents `UIDocumentPickerViewController`, reads the file,
  and returns a bookmark identifier plus file metadata
- `readByBookmark`: resolves a previously saved bookmark and reads the file again
- `forgetBookmark`: removes a stored bookmark

## Package Layout

- `src/`: Rust plugin entrypoints and mobile bridge
- `ios/`: Swift implementation for the iOS native plugin
- `guest-js/`: TypeScript guest API entrypoint
- `permissions/`: default Tauri permission definitions
