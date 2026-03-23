import { invoke } from '@tauri-apps/api/core'

export interface PickResult {
  bookmarkId: string
  fileName: string
  content: string
}

export interface ReadResult {
  fileName: string
  content: string
}

export async function pickAndBookmark(): Promise<PickResult> {
  return invoke<PickResult>('plugin:ios-bookmark|pick_and_bookmark')
}

export async function readByBookmark(id: string): Promise<ReadResult> {
  return invoke<ReadResult>('plugin:ios-bookmark|read_by_bookmark', { id })
}

export async function forgetBookmark(id: string): Promise<void> {
  return invoke<void>('plugin:ios-bookmark|forget_bookmark', { id })
}
