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
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_folder_bookmark', { id, targetPath })
}

export async function forgetBookmark(id: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|forget_bookmark', { id })
}
