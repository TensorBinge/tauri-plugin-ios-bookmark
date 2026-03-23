import test from 'node:test'
import assert from 'node:assert/strict'

import * as api from '../dist/index.js'

test('exports the guest API functions', () => {
  assert.equal(typeof api.pickAndBookmark, 'function')
  assert.equal(typeof api.readByBookmark, 'function')
  assert.equal(typeof api.forgetBookmark, 'function')
})