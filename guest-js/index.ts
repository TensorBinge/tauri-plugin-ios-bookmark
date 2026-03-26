import { invoke } from '@tauri-apps/api/core'

export interface PickBookmarkRequest {
  targetPath?: string
  suggestedFileName?: string
}

export interface PickResult {
  bookmarkId: string
  fileName: string
  filePath?: string
  content: string
}

export interface ReadResult {
  fileName: string
  content: string
}

export async function pickAndBookmark(request?: PickBookmarkRequest): Promise<PickResult> {
  return request === undefined
    ? invoke<PickResult>('plugin:ios-bookmark|pick_and_bookmark')
    : invoke<PickResult>('plugin:ios-bookmark|pick_and_bookmark', { request })
}

export async function readByBookmark(id: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_bookmark', { id })
}

export async function forgetBookmark(id: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|forget_bookmark', { id })
}
