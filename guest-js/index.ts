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

export async function pickAndBookmark(request?: PickBookmarkRequest): Promise<PickResult | null> {
  return request === undefined
    ? invoke<PickResult | null>('plugin:ios-bookmark|pick_and_bookmark')
    : invoke<PickResult | null>('plugin:ios-bookmark|pick_and_bookmark', { request })
}

export async function readByBookmark(id: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_bookmark', { id })
}

export async function pickFolderAndBookmark(request?: PickFolderBookmarkRequest): Promise<PickFolderResult | null> {
  return request === undefined
    ? invoke<PickFolderResult | null>('plugin:ios-bookmark|pick_folder_and_bookmark')
    : invoke<PickFolderResult | null>('plugin:ios-bookmark|pick_folder_and_bookmark', { request })
}

export async function readByFolderBookmark(id: string, targetPath: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_folder_bookmark', { args: { id, targetPath } })
}

export async function forgetBookmark(id: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|forget_bookmark', { id })
}
