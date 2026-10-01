import { test } from 'node:test'
import assert from 'node:assert/strict'
import { virtualRange } from './virtualRange.js'
import { sortByInitial } from './alphabet.js'

test('a large list mounts a bounded viewport at the beginning, middle and end', () => {
  for (const top of [0, 3200000, 6399360]) {
    const { start, end } = virtualRange(100000, 64, top, 640)
    assert.ok(end - start <= 22)
    assert.ok(start <= Math.floor(top / 64))
    assert.ok(end >= Math.min(100000, Math.ceil((top + 640) / 64)))
    assert.equal(start * 64 + (end - start) * 64 + (100000 - end) * 64, 6400000)
  }
})

test('offscreen alphabet groups and empty lists mount no rows', () => {
  assert.deepEqual(virtualRange(1000, 64, -641, 640), { start: 0, end: 0 })
  assert.deepEqual(virtualRange(1000, 64, 64000, 640), { start: 0, end: 0 })
  assert.deepEqual(virtualRange(0, 64, 0, 640), { start: 0, end: 0 })
})

test('cached sorting preserves numeric and Chinese initial ordering without mutating input', () => {
  const items = ['Zebra', '阿尔法', 'Alpha 10', '#Song', 'Alpha 2'].map(title => ({ title }))
  const original = items.slice()
  assert.deepEqual(sortByInitial(items, item => item.title).map(item => item.title), ['#Song', '阿尔法', 'Alpha 2', 'Alpha 10', 'Zebra'])
  assert.deepEqual(items, original)
})
