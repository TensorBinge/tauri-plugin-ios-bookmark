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
  assert.equal(typeof api.createFolderByFolderBookmark, 'function')
  assert.equal(typeof api.createMarkdownFileByFolderBookmark, 'function')
  assert.equal(typeof api.renameByFolderBookmark, 'function')
  assert.equal(typeof api.deleteByFolderBookmark, 'function')
  assert.equal(typeof api.readByBookmark, 'function')
  assert.equal(typeof api.writeByBookmark, 'function')
  assert.equal(typeof api.readByFolderBookmark, 'function')
  assert.equal(typeof api.readBinaryByFolderBookmark, 'function')
  assert.equal(typeof api.writeByFolderBookmark, 'function')
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

test('pickAndBookmark returns null when the native picker is cancelled', async () => {
  invokeResult = null

  const result = await api.pickAndBookmark()

  assert.equal(result, null)
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

test('pickFolderAndBookmark returns null when the native picker is cancelled', async () => {
  invokeResult = null

  const result = await api.pickFolderAndBookmark()

  assert.equal(result, null)
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

test('readBinaryByFolderBookmark forwards the folder bookmark id and target path', async () => {
  invokeResult = {
    fileName: 'diagram.png',
    filePath: '/docs/assets/diagram.png',
    mimeType: 'image/png',
    base64Content: 'ZmFrZS1pbWFnZQ==',
  }

  const result = await api.readBinaryByFolderBookmark('folder-123', '/docs/assets/diagram.png')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|read_binary_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs/assets/diagram.png',
      },
    },
    undefined,
  ]])
})

test('listByFolderBookmark forwards the folder bookmark id and target path', async () => {
  invokeResult = [
    {
      name: 'notes',
      path: '/docs/notes',
      isDir: true,
      size: 0,
      mtime: 42,
    },
  ]

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

test('createFolderByFolderBookmark forwards the folder bookmark id, parent path, and name', async () => {
  invokeResult = {
    name: 'notes',
    path: '/docs/notes',
    isDir: true,
    size: 0,
    mtime: 42,
  }

  const result = await api.createFolderByFolderBookmark('folder-123', '/docs', 'notes')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|create_folder_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        parentPath: '/docs',
        name: 'notes',
      },
    },
    undefined,
  ]])
})

test('createMarkdownFileByFolderBookmark forwards the folder bookmark id, parent path, file name, and content', async () => {
  invokeResult = {
    name: 'notes.md',
    path: '/docs/notes.md',
    isDir: false,
    size: 12,
    mtime: 42,
  }

  const result = await api.createMarkdownFileByFolderBookmark('folder-123', '/docs', 'notes.md', '# Notes')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|create_markdown_file_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        parentPath: '/docs',
        name: 'notes.md',
        content: '# Notes',
      },
    },
    undefined,
  ]])
})

test('renameByFolderBookmark forwards the folder bookmark id, target path, and next name', async () => {
  invokeResult = {
    name: 'ideas.md',
    path: '/docs/ideas.md',
    isDir: false,
    size: 8,
    mtime: 42,
  }

  const result = await api.renameByFolderBookmark('folder-123', '/docs/notes.md', 'ideas.md')

  assert.deepEqual(result, invokeResult)
  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|rename_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs/notes.md',
        name: 'ideas.md',
      },
    },
    undefined,
  ]])
})

test('deleteByFolderBookmark forwards the folder bookmark id and target path', async () => {
  await api.deleteByFolderBookmark('folder-123', '/docs/notes.md')

  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|delete_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs/notes.md',
      },
    },
    undefined,
  ]])
})

test('writeByBookmark forwards the bookmark id and updated content', async () => {
  await api.writeByBookmark('bookmark-123', '# Updated')

  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|write_by_bookmark',
    {
      args: {
        id: 'bookmark-123',
        content: '# Updated',
      },
    },
    undefined,
  ]])
})

test('writeByFolderBookmark forwards the folder bookmark id, target path, and content', async () => {
  await api.writeByFolderBookmark('folder-123', '/docs/related.md', '# Updated')

  assert.deepEqual(invokeCalls, [[
    'plugin:ios-bookmark|write_by_folder_bookmark',
    {
      args: {
        id: 'folder-123',
        targetPath: '/docs/related.md',
        content: '# Updated',
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