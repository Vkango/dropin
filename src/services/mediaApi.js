import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const commands = {
  metadataFile: 'media_metadata_read_file',
  metadataUrl: 'media_metadata_read_url',
  lyrics: 'media_lyrics_read',
  addRoot: 'media_library_add_root',
  removeRoot: 'media_library_remove_root',
  roots: 'media_library_roots',
  scan: 'media_library_scan',
  cancelScan: 'media_library_cancel_scan',
  tracks: 'media_library_tracks',
  albums: 'media_library_albums',
  artists: 'media_library_artists',
  refreshTrack: 'media_library_refresh_track',
  removeTrack: 'media_library_remove_track',
  cover: 'media_cover_get',
  history: 'media_playback_history',
  record: 'media_playback_record',
  pickFolder: 'media_pick_folder',
  playbackOpen: 'media_playback_open'
}

export const mediaApi = {
  readFile: (path) => invoke(commands.metadataFile, { path }),
  readUrl: (url) => invoke(commands.metadataUrl, { url }),
  lyrics: (trackId) => invoke(commands.lyrics, { trackId }),
  addRoot: (path) => invoke(commands.addRoot, { path }),
  removeRoot: (rootId) => invoke(commands.removeRoot, { rootId }),
  roots: () => invoke(commands.roots),
  scan: (rootIds = null) => invoke(commands.scan, { rootIds }),
  cancelScan: (jobId) => invoke(commands.cancelScan, { jobId }),
  tracks: ({ search = '', limit = 500, offset = 0 } = {}) => invoke(commands.tracks, { search, limit, offset }),
  albums: (search = '') => invoke(commands.albums, { search }),
  artists: (search = '') => invoke(commands.artists, { search }),
  refreshTrack: (trackId) => invoke(commands.refreshTrack, { trackId }),
  removeTrack: (trackId) => invoke(commands.removeTrack, { trackId }),
  cover: (coverId) => invoke(commands.cover, { coverId }),
  history: (limit = 50) => invoke(commands.history, { limit }),
  record: (trackId, positionMs = 0) => invoke(commands.record, { trackId, positionMs }),
  pickFolder: () => invoke(commands.pickFolder),
  openPlayback: (trackId) => invoke(commands.playbackOpen, { trackId })
}

export async function listenToMediaEvents(handler) {
  const eventNames = [
    'media/scan-progress',
    'media/track-added',
    'media/track-updated',
    'media/metadata-updated',
    'media/scan-finished',
    'media/error'
  ]
  const unlisteners = await Promise.all(eventNames.map((eventName) =>
    listen(eventName, (event) => handler(eventName, event.payload))
  ))
  return () => unlisteners.forEach((unlisten) => unlisten())
}

export function coverDataUrl(payload) {
  if (!payload?.dataBase64 || !payload?.mimeType) return ''
  return `data:${payload.mimeType};base64,${payload.dataBase64}`
}
