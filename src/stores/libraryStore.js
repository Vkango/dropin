import { computed, reactive, ref } from 'vue'
import { mediaApi, listenToMediaEvents, coverDataUrl } from '../services/mediaApi.js'

const state = reactive({
  tracks: [],
  albums: [],
  artists: [],
  roots: [],
  history: [],
  playlists: [],
  tags: [],
  total: 0,
  loading: false,
  scanning: false,
  scanJob: null,
  scanProgress: null,
  error: null,
  covers: new Map(),
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
  cover: track.coverId ? state.covers.get(track.coverId) || '/assets/cover.jpg' : '/assets/cover.jpg',
  url: track.path || track.url || '',
  sourcePath: track.path || null
})

const toAlbum = (album) => ({
  ...album,
  year: album.year == null ? '' : String(album.year),
  cover: album.coverId ? state.covers.get(album.coverId) || '/assets/cover.jpg' : '/assets/cover.jpg',
  addedDate: ''
})

const toArtist = (artist) => ({
  ...artist,
  cover: artist.coverId ? state.covers.get(artist.coverId) || '/assets/cover.jpg' : '/assets/cover.jpg',
  avatar: artist.coverId ? state.covers.get(artist.coverId) || '/assets/cover.jpg' : '/assets/cover.jpg',
  followers: artist.followers || 0,
  isFollowing: Boolean(artist.isFollowing)
})

export function useLibraryStore() {
  const initialized = ref(false)

  const refresh = async () => {
    state.loading = true
    state.error = null
    try {
      const [trackResult, albumResult, artistResult, rootsResult, historyResult, playlistResult, tagResult] = await Promise.all([
        mediaApi.tracks(),
        mediaApi.albums(),
        mediaApi.artists(),
        mediaApi.roots(),
        mediaApi.history(),
        mediaApi.playlistList(),
        mediaApi.tagList()
      ])
      state.tracks = trackResult.tracks || []
      state.total = trackResult.total || state.tracks.length
      state.albums = albumResult.albums || []
      state.artists = artistResult.artists || []
      state.roots = rootsResult.roots || []
      state.history = historyResult.history || []
      state.playlists = playlistResult.playlists || []
      state.tags = tagResult.tags || []
      initialized.value = true
    } catch (error) {
      state.error = error
    } finally {
      state.loading = false
    }
  }

  const loadCover = async (coverId) => {
    if (!coverId || state.covers.has(coverId)) return state.covers.get(coverId) || ''
    try {
      const payload = await mediaApi.cover(coverId)
      const url = coverDataUrl(payload)
      state.covers.set(coverId, url)
      return url
    } catch (error) {
      state.error = error
      return ''
    }
  }

  const hydrateCovers = async (tracks = state.tracks) => {
    await Promise.all([...new Set(tracks.map((track) => track.coverId).filter(Boolean))].map(loadCover))
    state.tracks = state.tracks.slice()
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
    const result = await mediaApi.tracks({ playlistId })
    return result.tracks || []
  }

  const createTag = async (label) => {
    const result = await mediaApi.tagCreate(label)
    await refresh()
    return result
  }

  const removeTag = async (tagId) => {
    const result = await mediaApi.tagRemove(tagId)
    await refresh()
    return result
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
    return result.tracks || []
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
        await hydrateCovers()
      } else if (name === 'media/track-updated' || name === 'media/metadata-updated') {
        await refresh()
        await hydrateCovers()
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
    refresh,
    hydrateCovers,
    addRootAndScan,
    scan,
    openPlayback,
    createPlaylist,
    removePlaylist,
    renamePlaylist,
    addToPlaylist,
    removeFromPlaylist,
    playlistTracks,
    createTag,
    removeTag,
    tagTrack,
    untagTrack,
    tracksByTag,
    installListeners,
    dispose,
    mediaApi
  }
}
