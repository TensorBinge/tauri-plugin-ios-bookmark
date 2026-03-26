import test, { beforeEach } from 'node:test'
import assert from 'node:assert/strict'

import * as api from '../dist/index.js'

let invokeCalls = []
let invokeResult = undefined

beforeEach(() => {
  invokeCalls = []
  invokeResult = undefined
  globalThis.window = {
    __TAURI_INTERNALS__: {
      invoke: async (...args) => {
        invokeCalls.push(args)
        return invokeResult
      },
    },
  }
})

test('exports the guest API functions', () => {
  assert.equal(typeof api.pickAndBookmark, 'function')
  assert.equal(typeof api.readByBookmark, 'function')
  assert.equal(typeof api.forgetBookmark, 'function')
})

test('pickAndBookmark invokes the plugin command without a request payload by default', async () => {
  invokeResult = {
    bookmarkId: 'bookmark-123',
    fileName: 'notes.md',
    filePath: '/docs/notes.md',
    content: '# Notes',
  }

  const result = await api.pickAndBookmark()

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|pick_and_bookmark',
    {},
    undefined,
  ]])
})

test('pickAndBookmark forwards an explicit target-path request payload', async () => {
  invokeResult = {
    bookmarkId: 'bookmark-456',
    fileName: 'related.md',
    filePath: '/docs/related.md',
    content: '# Related',
  }

  const result = await api.pickAndBookmark({
    targetPath: '/docs/related.md',
  })

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|pick_and_bookmark',
    {
      request: {
        targetPath: '/docs/related.md',
      },
    },
    undefined,
  ]])
})