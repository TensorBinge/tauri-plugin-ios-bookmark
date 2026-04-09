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
  assert.equal(typeof api.exportPdf, 'function')
  assert.equal(typeof api.exportFile, 'function')
  assert.equal(typeof api.pickAndBookmark, 'function')
  assert.equal(typeof api.pickFolderAndBookmark, 'function')
  assert.equal(typeof api.listByFolderBookmark, 'function')
  assert.equal(typeof api.readByBookmark, 'function')
  assert.equal(typeof api.readByFolderBookmark, 'function')
  assert.equal(typeof api.writeByBookmark, 'function')
  assert.equal(typeof api.writeByFolderBookmark, 'function')
  assert.equal(typeof api.createDirectoryByFolderBookmark, 'function')
  assert.equal(typeof api.forgetBookmark, 'function')
})

test('exportPdf forwards the file name and serialized html to the plugin command', async () => {
  await api.exportPdf('theory.pdf', '<!DOCTYPE html>')

  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|export_pdf',
    {
      fileName: 'theory.pdf',
      html: '<!DOCTYPE html>',
    },
    undefined,
  ]])
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

test('pickFolderAndBookmark invokes the plugin command without a request payload by default', async () => {
  invokeResult = {
    bookmarkId: 'folder-456',
    folderName: 'docs',
    folderPath: '/docs',
  }

  const result = await api.pickFolderAndBookmark()

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|pick_folder_and_bookmark',
    {},
    undefined,
  ]])
})

test('pickFolderAndBookmark forwards an explicit target-path request payload', async () => {
  invokeResult = {
    bookmarkId: 'folder-123',
    folderName: 'docs',
    folderPath: '/docs',
  }

  const result = await api.pickFolderAndBookmark({
    targetPath: '/docs',
  })

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|pick_folder_and_bookmark',
    {
      request: {
        targetPath: '/docs',
      },
    },
    undefined,
  ]])
})

test('readByFolderBookmark forwards the folder bookmark id and target path', async () => {
  invokeResult = {
    fileName: 'related.md',
    filePath: '/docs/related.md',
    content: '# Related',
  }

  const result = await api.readByFolderBookmark('folder-123', '/docs/related.md')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|read_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs/related.md',
      },
    },
    undefined,
  ]])
})

test('listByFolderBookmark forwards the folder bookmark id and target path', async () => {
  invokeResult = [{
    name: 'notes.md',
    path: '/docs/notes.md',
    isDir: false,
    size: 18,
    mtime: 42,
  }]

  const result = await api.listByFolderBookmark('folder-123', '/docs')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|list_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs',
      },
    },
    undefined,
  ]])
})

test('writeByBookmark forwards the bookmark id and markdown contents', async () => {
  invokeResult = {
    fileName: 'notes.md',
    filePath: '/docs/notes.md',
    content: '# Updated',
  }

  const result = await api.writeByBookmark('bookmark-123', '# Updated')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|write_by_bookmark',
    {
      args: {
        id: 'bookmark-123',
        contents: '# Updated',
      },
    },
    undefined,
  ]])
})

test('writeByFolderBookmark forwards the folder bookmark id, target path, and markdown contents', async () => {
  invokeResult = {
    fileName: 'notes.md',
    filePath: '/docs/notes.md',
    content: '# Updated',
  }

  const result = await api.writeByFolderBookmark('folder-123', '/docs/notes.md', '# Updated')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|write_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs/notes.md',
        contents: '# Updated',
      },
    },
    undefined,
  ]])
})

test('createDirectoryByFolderBookmark forwards the folder bookmark id and target path', async () => {
  await api.createDirectoryByFolderBookmark('folder-123', '/docs/research')

  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|create_directory_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs/research',
      },
    },
    undefined,
  ]])
})

test('exportFile forwards the temporary file path to the plugin command', async () => {
  await api.exportFile('/tmp/markscope-exports/theory.epub')

  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|export_file',
    {
      path: '/tmp/markscope-exports/theory.epub',
    },
    undefined,
  ]])
})