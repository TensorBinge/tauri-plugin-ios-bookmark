import { invoke } from '@tauri-apps/api/core'

export interface PickBookmarkRequest {
  targetPath?: string
  suggestedFileName?: string
}

export interface PickFolderBookmarkRequest {
  targetPath?: string
  requireEmpty?: boolean
}

export interface PickResult {
  bookmarkId: string
  fileName: string
  filePath?: string
  content: string
}

export interface ReadResult {
  fileName: string
  filePath: string
  content: string
}

export interface BinaryReadResult {
  fileName: string
  filePath: string
  mimeType: string
  base64Content: string
}

export interface PickFolderResult {
  bookmarkId: string
  folderName: string
  folderPath: string
}

export interface FolderBookmarkEntry {
  name: string
  path: string
  isDir: boolean
  size?: number
  mtime?: number | string
}

export async function exportPdf(fileName: string, html: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|export_pdf', { fileName, html })
}

export async function exportFile(path: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|export_file', { path })
}

export async function pickAndBookmark(request?: PickBookmarkRequest): Promise<PickResult | null> {
  return request === undefined
    ? invoke<PickResult | null>('plugin:ios-bookmark|pick_and_bookmark')
    : invoke<PickResult | null>('plugin:ios-bookmark|pick_and_bookmark', { request })
}

export async function readByBookmark(id: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_bookmark', { id })
}

export async function writeByBookmark(id: string, content: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|write_by_bookmark', { args: { id, content } })
}

export async function pickFolderAndBookmark(request?: PickFolderBookmarkRequest): Promise<PickFolderResult | null> {
  return request === undefined
    ? invoke<PickFolderResult | null>('plugin:ios-bookmark|pick_folder_and_bookmark')
    : invoke<PickFolderResult | null>('plugin:ios-bookmark|pick_folder_and_bookmark', { request })
}

export async function listByFolderBookmark(id: string, targetPath: string): Promise<FolderBookmarkEntry[]> {
  return invoke<FolderBookmarkEntry[]>('plugin:ios-bookmark|list_by_folder_bookmark', { args: { id, targetPath } })
}

export async function createFolderByFolderBookmark(id: string, parentPath: string, name: string): Promise<FolderBookmarkEntry> {
  return invoke<FolderBookmarkEntry>('plugin:ios-bookmark|create_folder_by_folder_bookmark', { args: { id, parentPath, name } })
}

export async function createMarkdownFileByFolderBookmark(
  id: string,
  parentPath: string,
  name: string,
  content: string,
): Promise<FolderBookmarkEntry> {
  return invoke<FolderBookmarkEntry>('plugin:ios-bookmark|create_markdown_file_by_folder_bookmark', { args: { id, parentPath, name, content } })
}

export async function renameByFolderBookmark(id: string, targetPath: string, name: string): Promise<FolderBookmarkEntry> {
  return invoke<FolderBookmarkEntry>('plugin:ios-bookmark|rename_by_folder_bookmark', { args: { id, targetPath, name } })
}

export async function deleteByFolderBookmark(id: string, targetPath: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|delete_by_folder_bookmark', { args: { id, targetPath } })
}

export async function readByFolderBookmark(id: string, targetPath: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_folder_bookmark', { args: { id, targetPath } })
}

export async function readBinaryByFolderBookmark(id: string, targetPath: string): Promise<BinaryReadResult> {
  return invoke<BinaryReadResult>('plugin:ios-bookmark|read_binary_by_folder_bookmark', { args: { id, targetPath } })
}

export async function writeByFolderBookmark(id: string, targetPath: string, content: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|write_by_folder_bookmark', { args: { id, targetPath, content } })
}

export async function forgetBookmark(id: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|forget_bookmark', { id })
}
