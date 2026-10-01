import { test } from 'node:test'
import assert from 'node:assert/strict'
import { isManualTag } from './tagProvider.js'

test('associated tags remain plugin tags despite their internal key or missing source plugin', () => {
  assert.equal(isManualTag({ key: 'manual.associated', sourceProviderKey: 'energy' }), false)
  assert.equal(isManualTag({ key: 'manual.associated', pluginId: 'energy.plugin' }), false)
  assert.equal(isManualTag({ kind: 'plugin', sourceAvailable: false }), false)
  assert.equal(isManualTag({ key: 'manual.collection', tagId: 'collection', kind: 'manual' }), true)
  assert.equal(isManualTag(null), false)
})
