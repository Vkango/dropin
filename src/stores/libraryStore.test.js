import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mediaApi } from '../services/mediaApi.js'
import { useLibraryStore } from './libraryStore.js'

test('concurrent refreshes share one query and cover URLs need no binary IPC', async () => {
  const original = { ...mediaApi }
  const originalWindow = globalThis.window
  let calls = 0
  globalThis.window = { __TAURI_INTERNALS__: { convertFileSrc: (id, scheme) => `http://${scheme}.localhost/${id}` } }
  Object.assign(mediaApi, {
    tracks: async () => { calls++; await new Promise(resolve => setTimeout(resolve, 5)); return { tracks: [{ id: 'song', coverId: 'cover' }], total: 1 } },
    albums: async () => ({ albums: [{ coverId: 'cover' }] }),
    artists: async () => ({ artists: [{ coverId: 'cover' }] }),
    roots: async () => ({}), history: async () => ({}), playlistList: async () => ({}), tagList: async () => ({}), sortRuleList: async () => ({}),
    cover: () => { throw new Error('Base64 cover request') },
    coverPath: () => { throw new Error('Eager cover request') }
  })
  try {
    const first = useLibraryStore(), second = useLibraryStore()
    await Promise.all([first.refresh(), second.refresh()])
    assert.equal(calls, 1)
    for (const result of [first.tracks, first.albums, first.artists]) assert.equal(result.value[0].cover, 'http://dropin-cover.localhost/cover')
    assert.equal(first.state.loading, false)
  } finally {
    Object.assign(mediaApi, original)
    globalThis.window = originalWindow
  }
})
