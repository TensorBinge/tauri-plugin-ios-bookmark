import { invoke } from '@tauri-apps/api/core'

export interface PickBookmarkRequest {
  targetPath?: string
  suggestedFileName?: string
}

export interface PickFolderBookmarkRequest {
  targetPath?: string
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

export interface DirectoryEntry {
  name: string
  path: string
  isDir: boolean
  size: number
  mtime: number
}

export interface PickFolderResult {
  bookmarkId: string
  folderName: string
  folderPath: string
}

export async function exportPdf(fileName: string, html: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|export_pdf', { fileName, html })
}

export async function exportFile(path: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|export_file', { path })
}

export async function pickAndBookmark(request?: PickBookmarkRequest): Promise<PickResult> {
  return request === undefined
    ? invoke<PickResult>('plugin:ios-bookmark|pick_and_bookmark')
    : invoke<PickResult>('plugin:ios-bookmark|pick_and_bookmark', { request })
}

export async function readByBookmark(id: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_bookmark', { id })
}

export async function pickFolderAndBookmark(request?: PickFolderBookmarkRequest): Promise<PickFolderResult> {
  return request === undefined
    ? invoke<PickFolderResult>('plugin:ios-bookmark|pick_folder_and_bookmark')
    : invoke<PickFolderResult>('plugin:ios-bookmark|pick_folder_and_bookmark', { request })
}

export async function readByFolderBookmark(id: string, targetPath: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_folder_bookmark', { args: { id, targetPath } })
}

export async function listByFolderBookmark(id: string, targetPath: string): Promise<DirectoryEntry[]> {
  return invoke<DirectoryEntry[]>('plugin:ios-bookmark|list_by_folder_bookmark', { args: { id, targetPath } })
}

export async function writeByBookmark(id: string, contents: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|write_by_bookmark', { args: { id, contents } })
}

export async function writeByFolderBookmark(id: string, targetPath: string, contents: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|write_by_folder_bookmark', { args: { id, targetPath, contents } })
}

export async function createDirectoryByFolderBookmark(id: string, targetPath: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|create_directory_by_folder_bookmark', { args: { id, targetPath } })
}

export async function forgetBookmark(id: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|forget_bookmark', { id })
}
