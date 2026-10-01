import { computed, reactive, ref } from 'vue'
import { mediaApi, listenToMediaEvents } from '../services/mediaApi.js'

let refreshPromise = null

const state = reactive({
  tracks: [],
  albums: [],
  artists: [],
  roots: [],
  history: [],
  playlists: [],
  tags: [],
  sortRules: [],
  total: 0,
  loading: false,
  scanning: false,
  scanJob: null,
  scanProgress: null,
  error: null,
  unlisten: null
})

const durationText = (durationMs = 0) => {
  const totalSeconds = Math.floor(Number(durationMs) / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
    : `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
}

const toSong = (track) => ({
  ...track,
  duration: durationText(track.durationMs),
  cover: mediaApi.coverUrl(track.coverId),
  url: track.path || track.url || '',
  sourcePath: track.path || null
})

const toAlbum = (album) => ({
  ...album,
  year: album.year == null ? '' : String(album.year),
  cover: mediaApi.coverUrl(album.coverId),
  addedDate: ''
})

const toArtist = (artist) => ({
  ...artist,
  cover: mediaApi.coverUrl(artist.coverId),
  avatar: mediaApi.coverUrl(artist.coverId),
  followers: artist.followers || 0,
  isFollowing: Boolean(artist.isFollowing)
})

export function useLibraryStore() {
  const initialized = ref(false)

  const refreshLibrary = async () => {
    state.loading = true
    state.error = null
    try {
      const [trackResult, albumResult, artistResult, rootsResult, historyResult, playlistResult, tagResult, sortRuleResult] = await Promise.all([
        mediaApi.tracks(),
        mediaApi.albums(),
        mediaApi.artists(),
        mediaApi.roots(),
        mediaApi.history(),
        mediaApi.playlistList(),
        mediaApi.tagList(),
        mediaApi.sortRuleList()
      ])
      state.tracks = trackResult.tracks || []
      state.total = trackResult.total || state.tracks.length
      state.albums = albumResult.albums || []
      state.artists = artistResult.artists || []
      state.roots = rootsResult.roots || []
      state.history = historyResult.history || []
      state.playlists = playlistResult.playlists || []
      state.tags = tagResult.tags || []
      state.sortRules = sortRuleResult.rules || []
      initialized.value = true
    } catch (error) {
      state.error = error
    } finally {
      state.loading = false
    }
  }

  const refresh = () => {
    if (!refreshPromise) refreshPromise = refreshLibrary().finally(() => { refreshPromise = null })
    return refreshPromise
  }

  const addRootAndScan = async (path) => {
    const result = await mediaApi.addRoot(path)
    await refresh()
    const scan = await mediaApi.scan([result.root.id])
    state.scanning = true
    state.scanJob = scan.job
    return scan
  }

  const scan = async (rootIds = null) => {
    const result = await mediaApi.scan(rootIds)
    state.scanning = true
    state.scanJob = result.job
    return result
  }

  const openPlayback = async (track) => {
    const result = await mediaApi.openPlayback(track.id)
    await mediaApi.record(track.id, 0)
    return result
  }

  const createPlaylist = async (name, description = null) => {
    const result = await mediaApi.playlistCreate(name, description)
    await refresh()
    return result
  }

  const removePlaylist = async (playlistId) => {
    const result = await mediaApi.playlistRemove(playlistId)
    await refresh()
    return result
  }

  const renamePlaylist = async (playlistId, name, description = null) => {
    const result = await mediaApi.playlistRename(playlistId, name, description)
    await refresh()
    return result
  }

  const addToPlaylist = async (playlistId, trackId) => {
    const result = await mediaApi.playlistAddTrack(playlistId, trackId)
    await refresh()
    return result
  }

  const removeFromPlaylist = async (playlistId, trackId) => {
    const result = await mediaApi.playlistRemoveTrack(playlistId, trackId)
    await refresh()
    return result
  }

  const playlistTracks = async (playlistId) => {
    const result = await mediaApi.playlistOrderGet(playlistId)
    const tracks = result.tracks || []
    return tracks.map(toSong)
  }

  const playlistOrder = async (playlistId) => {
    const result = await mediaApi.playlistOrderGet(playlistId)
    const tracks = result.tracks || []
    return {
      ...result,
      tracks: tracks.map(toSong)
    }
  }

  const previewPlaylistOrder = async (playlistId, sortRuleId = null, rule = null) => {
    const result = await mediaApi.playlistOrderPreview(playlistId, sortRuleId, rule)
    const tracks = result.tracks || []
    return {
      ...result,
      tracks: tracks.map(toSong)
    }
  }

  const savePlaylistOrder = async (playlistId, trackIds, sortRuleId = null) => {
    const result = await mediaApi.playlistOrderSave(playlistId, trackIds, sortRuleId)
    await refresh()
    return result
  }

  const clonePlaylist = async (playlistId, name, description = null, trackIds = null) => {
    const result = await mediaApi.playlistClone(playlistId, name, description, trackIds)
    await refresh()
    return result
  }

  const saveSortRule = async (sortRuleId, name, rule) => {
    const result = await mediaApi.sortRuleSave(sortRuleId, name, rule)
    await refresh()
    return result
  }

  const removeSortRule = async (sortRuleId) => {
    const result = await mediaApi.sortRuleRemove(sortRuleId)
    await refresh()
    return result
  }

  const getPlaylistRule = async (playlistId) => mediaApi.playlistRuleGet(playlistId)

  const getSortRule = async (sortRuleId) => mediaApi.sortRuleGet(sortRuleId)

  const evaluatePlaylist = async (playlistId, rule = null) => {
    const result = await mediaApi.playlistRuleEvaluate(playlistId, rule)
    const tracks = result.tracks || []
    const contributionsByTrackId = new Map(
      (result.contributions || []).map((item) => [
        item.trackId,
        (item.sources || []).map((source) => `${source.kind || ''}:${source.id || ''}`)
      ])
    )
    return {
      ...result,
      tracks: tracks.map((track) => ({
        ...toSong(track),
        sourceKeys: contributionsByTrackId.get(track.id) || []
      }))
    }
  }

  const savePlaylistRule = async (playlistId, rule) => {
    const result = await mediaApi.playlistRuleSave(playlistId, rule)
    await refresh()
    return result
  }

  const materializePlaylist = async (playlistId, trackIds) => {
    const result = await mediaApi.playlistRuleMaterialize(playlistId, trackIds)
    await refresh()
    return result
  }

  const sourceTracks = async (source) => {
    if (source?.kind === 'library') return state.tracks.map(toSong)
    if (source?.kind === 'tag') return tracksByTag(source.id)
    if (source?.kind === 'playlist') {
      const playlist = state.playlists.find((item) => item.id === source.id)
      return playlist?.type === 'dynamic'
        ? (await evaluatePlaylist(source.id)).tracks
        : playlistTracks(source.id)
    }
    return []
  }

  const createTag = async (label, providerKey = null, wiki = null) => {
    const result = await mediaApi.tagCreate(label, providerKey, wiki)
    await refresh()
    return result
  }

  const removeTag = async (tagId) => {
    const result = await mediaApi.tagRemove(tagId)
    await refresh()
    return result
  }

  const renameTag = async (tagId, name) => {
    const result = await mediaApi.tagRename(tagId, name)
    await refresh()
    return result
  }

  const trackTags = async (trackId) => {
    const result = await mediaApi.trackTags(trackId)
    return result?.tags || []
  }

  const tagTrack = async (trackId, label) => {
    const result = await mediaApi.trackTag(trackId, label)
    await refresh()
    return result
  }

  const untagTrack = async (trackId, tagId) => {
    const result = await mediaApi.trackUntag(trackId, tagId)
    await refresh()
    return result
  }

  const tracksByTag = async (tagId) => {
    const result = await mediaApi.tracks({ tagId })
    const tracks = result.tracks || []
    return tracks.map(toSong)
  }

  const installListeners = async (onEvent = null) => {
    if (state.unlisten) return state.unlisten
    state.unlisten = await listenToMediaEvents(async (name, payload) => {
      if (name === 'media/scan-progress') {
        state.scanProgress = payload
      } else if (name === 'media/scan-finished') {
        state.scanning = false
        state.scanJob = null
        await refresh()
      } else if (name === 'media/track-updated' || name === 'media/metadata-updated') {
        await refresh()
      } else if (name === 'media/error') {
        state.error = payload?.error || payload
      }
      if (onEvent) onEvent(name, payload)
    })
    return state.unlisten
  }

  const dispose = () => {
    state.unlisten?.()
    state.unlisten = null
  }

  return {
    state,
    initialized,
    tracks: computed(() => state.tracks.map(toSong)),
    albums: computed(() => state.albums.map(toAlbum)),
    artists: computed(() => state.artists.map(toArtist)),
    roots: computed(() => state.roots),
    playlists: computed(() => state.playlists),
    tags: computed(() => state.tags),
    sortRules: computed(() => state.sortRules),
    refresh,
    addRootAndScan,
    scan,
    openPlayback,
    createPlaylist,
    removePlaylist,
    renamePlaylist,
    addToPlaylist,
    removeFromPlaylist,
    playlistTracks,
    playlistOrder,
    previewPlaylistOrder,
    savePlaylistOrder,
    clonePlaylist,
    getPlaylistRule,
    getSortRule,
    evaluatePlaylist,
    savePlaylistRule,
    materializePlaylist,
    sourceTracks,
    createTag,
    removeTag,
    renameTag,
    tagTrack,
    untagTrack,
    trackTags,
    tracksByTag,
    saveSortRule,
    removeSortRule,
    installListeners,
    dispose,
    mediaApi
  }
}
