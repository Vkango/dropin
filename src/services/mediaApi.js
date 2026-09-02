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
  coverPath: 'media_cover_path',
  history: 'media_playback_history',
  record: 'media_playback_record',
  pickFolder: 'media_pick_folder',
  playbackOpen: 'media_playback_open',
  playlistCreate: 'media_playlist_create',
  playlistRemove: 'media_playlist_remove',
  playlistRename: 'media_playlist_rename',
  playlistList: 'media_playlist_list',
  playlistAddTrack: 'media_playlist_add_track',
  playlistRemoveTrack: 'media_playlist_remove_track',
  playlistRuleGet: 'media_playlist_rule_get',
  playlistRuleSave: 'media_playlist_rule_save',
  playlistRuleEvaluate: 'media_playlist_rule_evaluate',
  playlistRuleMaterialize: 'media_playlist_rule_materialize',
  playlistOrderGet: 'media_playlist_order_get',
  playlistOrderPreview: 'media_playlist_order_preview',
  playlistOrderSave: 'media_playlist_order_save',
  playlistClone: 'media_playlist_clone',
  sortRuleList: 'media_sort_rule_list',
  sortRuleGet: 'media_sort_rule_get',
  sortRuleSave: 'media_sort_rule_save',
  sortRuleRemove: 'media_sort_rule_remove',
  tagCreate: 'media_tag_create',
  tagRemove: 'media_tag_remove',
  tagList: 'media_tag_list',
  trackTag: 'media_track_tag',
  trackUntag: 'media_track_untag',
  dataDirRead: 'data_dir_read',
  dataDirSet: 'data_dir_set'
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
  tracks: ({ search = '', limit = 500, offset = 0, playlistId = null, tagId = null } = {}) =>
    invoke(commands.tracks, { search, limit, offset, playlistId, tagId }),
  albums: (search = '') => invoke(commands.albums, { search }),
  artists: (search = '') => invoke(commands.artists, { search }),
  refreshTrack: (trackId) => invoke(commands.refreshTrack, { trackId }),
  removeTrack: (trackId) => invoke(commands.removeTrack, { trackId }),
  cover: (coverId) => invoke(commands.cover, { coverId }),
  coverPath: (coverId) => invoke(commands.coverPath, { coverId }),
  history: (limit = 50) => invoke(commands.history, { limit }),
  record: (trackId, positionMs = 0) => invoke(commands.record, { trackId, positionMs }),
  pickFolder: () => invoke(commands.pickFolder),
  openPlayback: (trackId) => invoke(commands.playbackOpen, { trackId }),
  playlistCreate: (name, description = null) => invoke(commands.playlistCreate, { name, description }),
  playlistRemove: (playlistId) => invoke(commands.playlistRemove, { playlistId }),
  playlistRename: (playlistId, name, description = null) => invoke(commands.playlistRename, { playlistId, name, description }),
  playlistList: () => invoke(commands.playlistList),
  playlistAddTrack: (playlistId, trackId, position = null) => invoke(commands.playlistAddTrack, { playlistId, trackId, position }),
  playlistRemoveTrack: (playlistId, trackId) => invoke(commands.playlistRemoveTrack, { playlistId, trackId }),
  playlistRuleGet: (playlistId) => invoke(commands.playlistRuleGet, { playlistId }),
  playlistRuleSave: (playlistId, rule) => invoke(commands.playlistRuleSave, { playlistId, rule }),
  playlistRuleEvaluate: (playlistId, rule = null) => invoke(commands.playlistRuleEvaluate, { playlistId, rule }),
  playlistRuleMaterialize: (playlistId, trackIds) => invoke(commands.playlistRuleMaterialize, { playlistId, trackIds }),
  playlistOrderGet: (playlistId) => invoke(commands.playlistOrderGet, { playlistId }),
  playlistOrderPreview: (playlistId, sortRuleId = null, rule = null) =>
    invoke(commands.playlistOrderPreview, { playlistId, sortRuleId, rule }),
  playlistOrderSave: (playlistId, trackIds, sortRuleId = null) =>
    invoke(commands.playlistOrderSave, { playlistId, trackIds, sortRuleId }),
  playlistClone: (playlistId, name, description = null, trackIds = null) =>
    invoke(commands.playlistClone, { playlistId, name, description, trackIds }),
  sortRuleList: () => invoke(commands.sortRuleList),
  sortRuleGet: (sortRuleId) => invoke(commands.sortRuleGet, { sortRuleId }),
  sortRuleSave: (sortRuleId, name, rule) =>
    invoke(commands.sortRuleSave, { sortRuleId, name, rule }),
  sortRuleRemove: (sortRuleId) => invoke(commands.sortRuleRemove, { sortRuleId }),
  tagCreate: (label) => invoke(commands.tagCreate, { label }),
  tagRemove: (tagId) => invoke(commands.tagRemove, { tagId }),
  tagList: () => invoke(commands.tagList),
  trackTag: (trackId, label) => invoke(commands.trackTag, { trackId, label }),
  trackUntag: (trackId, tagId) => invoke(commands.trackUntag, { trackId, tagId }),
  dataDirRead: () => invoke(commands.dataDirRead),
  dataDirSet: (dataDir = null) => invoke(commands.dataDirSet, { dataDir })
}

export async function listenToMediaEvents(handler) {
  const eventNames = [
    'media/scan-progress',
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
