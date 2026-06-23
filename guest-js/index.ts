import { invoke } from '@tauri-apps/api/core'

// ── Request types ───────────────────────────────────────────────────

/** Options for the native file picker. */
export interface PickFileBookmarkRequest {
  /** Hint for the native picker's initial directory. */
  suggestedName?: string
  /** When true, the returned {@link FileBookmarkResult.content} is empty. */
  skipContent?: boolean
}

/** Options for the native folder picker. */
export interface PickFolderBookmarkRequest {
  /** When true, rejects folders that are not empty. Default false. */
  requireEmpty?: boolean
}

// ── Result types ────────────────────────────────────────────────────

/** Returned when a file bookmark is successfully created. */
export interface FileBookmarkResult {
  /** Opaque bookmark identifier (UUID string). */
  id: string
  /** Display filename (e.g. "notes.md"). */
  name: string
  /** Full file path for display in recent/resume UI. */
  path: string
  /** File content as UTF-8 text. Empty string when `skipContent` was set. */
  content: string
  /** Detected MIME type. May be absent for plain-text or unknown types. */
  mimeType?: string
}

/** Returned when a folder bookmark is successfully created. */
export interface FolderBookmarkResult {
  /** Opaque bookmark identifier (UUID string). */
  id: string
  /** Display folder name (e.g. "Documents"). */
  name: string
  /** Full folder path for display and matching. */
  path: string
}

/** Returned when reading the text content of a bookmarked file. */
export interface ReadResult {
  /** Display filename. */
  name: string
  /** Full file path. */
  path: string
  /** File content as UTF-8 text. */
  content: string
}

/** Returned when reading the binary content of a bookmarked file. */
export interface DataResult {
  /** Display filename. */
  name: string
  /** Full file path. */
  path: string
  /** Detected MIME type. */
  mimeType: string
  /** File content as base64-encoded string. */
  base64Data: string
}

/** A file or directory entry within a bookmarked folder scope. */
export interface Entry {
  /** Display name of the file or directory. */
  name: string
  /** Full path within the bookmark scope. */
  path: string
  /** Whether this entry is a directory. */
  isDir: boolean
  /** File size in bytes. Absent for directories. */
  size?: number
  /** Last modification time as Unix timestamp in milliseconds. */
  mtime?: number
}

// ── File bookmark commands ──────────────────────────────────────────

/**
 * Opens the native file picker, creates a security-scoped bookmark,
 * and returns the file's metadata and optional content.
 *
 * @param request - Optional picker configuration.
 * @returns The bookmark result, or `null` when the user cancels.
 */
export async function pickFileBookmark(request?: PickFileBookmarkRequest): Promise<FileBookmarkResult | null> {
  return request === undefined
    ? invoke<FileBookmarkResult | null>('plugin:ios-bookmark|pick_file_bookmark')
    : invoke<FileBookmarkResult | null>('plugin:ios-bookmark|pick_file_bookmark', { request })
}

/**
 * Reads the text content of a file previously authorized by a file bookmark.
 *
 * @param id - Bookmark identifier from {@link pickFileBookmark}.
 */
export async function readFileBookmark(id: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_file_bookmark', { id })
}

/**
 * Reads the binary content of a file previously authorized by a file bookmark.
 *
 * @param id - Bookmark identifier from {@link pickFileBookmark}.
 */
export async function readFileBookmarkData(id: string): Promise<DataResult> {
  return invoke<DataResult>('plugin:ios-bookmark|read_file_bookmark_data', { id })
}

/**
 * Writes text content to a file previously authorized by a file bookmark.
 *
 * @param id - Bookmark identifier.
 * @param content - UTF-8 text to write.
 */
export async function writeFileBookmark(id: string, content: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|write_file_bookmark', { args: { id, content } })
}

/**
 * Writes binary content to a file previously authorized by a file bookmark.
 *
 * @param id - Bookmark identifier.
 * @param data - Raw bytes to write.
 */
export async function writeFileBookmarkData(id: string, data: number[]): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|write_file_bookmark_data', { args: { id, data } })
}

// ── Folder bookmark commands ────────────────────────────────────────

/**
 * Opens the native folder picker, creates a security-scoped bookmark,
 * and returns the folder's metadata.
 *
 * @param request - Optional picker configuration.
 * @returns The bookmark result, or `null` when the user cancels.
 */
export async function pickFolderBookmark(request?: PickFolderBookmarkRequest): Promise<FolderBookmarkResult | null> {
  return request === undefined
    ? invoke<FolderBookmarkResult | null>('plugin:ios-bookmark|pick_folder_bookmark')
    : invoke<FolderBookmarkResult | null>('plugin:ios-bookmark|pick_folder_bookmark', { request })
}

/**
 * Lists directory entries within a bookmarked folder at the given path.
 *
 * @param id - Folder bookmark identifier.
 * @param path - Directory path within the bookmark scope.
 */
export async function listFolderBookmark(id: string, path: string): Promise<Entry[]> {
  return invoke<Entry[]>('plugin:ios-bookmark|list_folder_bookmark', { args: { id, path } })
}

/**
 * Reads the text content of a file within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param path - File path within the bookmark scope.
 */
export async function readFolderBookmark(id: string, path: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_folder_bookmark', { args: { id, path } })
}

/**
 * Reads the binary content of a file within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param path - File path within the bookmark scope.
 */
export async function readFolderBookmarkData(id: string, path: string): Promise<DataResult> {
  return invoke<DataResult>('plugin:ios-bookmark|read_folder_bookmark_data', { args: { id, path } })
}

/**
 * Writes text content to a file within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param path - File path within the bookmark scope.
 * @param content - UTF-8 text to write.
 */
export async function writeFolderBookmark(id: string, path: string, content: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|write_folder_bookmark', { args: { id, path, content } })
}

/**
 * Writes binary content to a file within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param path - File path within the bookmark scope.
 * @param data - Raw bytes to write.
 */
export async function writeFolderBookmarkData(id: string, path: string, data: number[]): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|write_folder_bookmark_data', { args: { id, path, data } })
}

// ── Folder-scoped mutations ─────────────────────────────────────────

/**
 * Creates a subdirectory within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param parentPath - Parent directory path within the scope.
 * @param name - New directory name.
 */
export async function createDir(id: string, parentPath: string, name: string): Promise<Entry> {
  return invoke<Entry>('plugin:ios-bookmark|create_dir', { args: { id, parentPath, name } })
}

/**
 * Creates a new file within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param parentPath - Parent directory path within the scope.
 * @param name - New file name.
 * @param content - Optional initial text content (empty file when omitted).
 */
export async function createFile(
  id: string,
  parentPath: string,
  name: string,
  content?: string,
): Promise<Entry> {
  return invoke<Entry>('plugin:ios-bookmark|create_file', { args: { id, parentPath, name, content } })
}

/**
 * Renames a file or directory within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param path - Path of the item to rename, within the scope.
 * @param newName - New name (not a full path — just the filename).
 */
export async function rename(id: string, path: string, newName: string): Promise<Entry> {
  return invoke<Entry>('plugin:ios-bookmark|rename', { args: { id, path, newName } })
}

/**
 * Moves a file or directory to a new parent within the same bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param srcPath - Source path within the scope.
 * @param destParentPath - Destination parent directory within the scope.
 * @param name - Optional new name at destination (keeps source name when omitted).
 */
export async function move(
  id: string,
  srcPath: string,
  destParentPath: string,
  name?: string,
): Promise<Entry> {
  return invoke<Entry>('plugin:ios-bookmark|move', { args: { id, srcPath, destParentPath, name } })
}

/**
 * Deletes a file or directory within a bookmarked folder scope.
 *
 * @param id - Folder bookmark identifier.
 * @param path - Path of the item to remove, within the scope.
 */
export async function remove(id: string, path: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|remove', { args: { id, path } })
}

// ── Lifecycle ───────────────────────────────────────────────────────

/**
 * Validates that a stored bookmark is still usable without performing I/O.
 * Resolves the security-scoped URL and verifies the resource is accessible.
 *
 * @param id - Bookmark identifier.
 * @returns `true` if the bookmark is valid and the resource exists.
 */
export async function checkBookmark(id: string): Promise<boolean> {
  return invoke<boolean>('plugin:ios-bookmark|check_bookmark', { id })
}

/**
 * Releases the native security-scoped grant and removes the bookmark from
 * persistent storage.
 *
 * @param id - Bookmark identifier.
 */
export async function releaseBookmark(id: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|release_bookmark', { id })
}
