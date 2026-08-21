<script setup>
import { ref, reactive, onMounted, onBeforeUnmount, provide, computed, watch, nextTick } from 'vue'
import Sidebar from './components/Sidebar.vue'
import MusicLibrary from './components/MusicLibrary.vue'
import HomePage from './components/HomePage.vue'
import AlbumsPage from './components/AlbumsPage.vue'
import ArtistsPage from './components/ArtistsPage.vue'
import SoundEffectsPage from './components/SoundEffectsPage.vue'
import PluginsPage from './components/PluginsPage.vue'
import DetailPanel from './components/DetailPanel.vue'
import PlayerSurface from './components/PlayerSurface.vue'
import TitleBar from './components/TitleBar.vue'
import Drawer from './components/Drawer.vue'
import Playlist from './components/Playlist.vue'
import { useReducedMotion } from 'motion-v'
import { themeManager } from './utils/themeManager.js'
import { bassCall, listenToBassEvents } from './services/bassApi.js'
import { useLibraryStore } from './stores/libraryStore.js'
import { animateElement, APPLE_SPRING } from './utils/motion.js'

const libraryStore = useLibraryStore()

// 当前页面状态
const currentPage = ref('home')
const reducedMotion = useReducedMotion()
const pageHistory = ref(['home'])
const pageCache = reactive(new Map())

// 全屏播放器状态
const showFullscreenPlayer = ref(false)
const isTitlebarScrolled = ref(false)
let unbindScrollSources = () => { }

// 页面组件映射
const pageComponents = {
  home: HomePage,
  library: MusicLibrary,
  albums: AlbumsPage,
  artists: ArtistsPage,
  effects: SoundEffectsPage,
  plugins: PluginsPage
}

// 当前播放的歌曲
const currentSong = ref({
  title: '未选择歌曲',
  artist: '请先导入音乐目录',
  album: '',
  duration: '00:00',
  cover: '/assets/cover.jpg'
})

provide('currentSong', currentSong)

// 播放状态
const isPlaying = ref(false)
const currentTime = ref('00:00')
const currentTimeMs = ref(0)
const totalTime = ref('00:00')
const progress = ref(0)
const activeChannelId = ref(null)
const playbackQueue = ref(null)
const volume = ref(75)
const muted = ref(false)
const playbackMode = ref('sequential')
const listLoop = ref(false)
const shufflePlayedIds = new Set()
let completionInFlight = false
const isQueueDrawerOpen = ref(false)
const fullscreenBackgroundMode = ref('flowing')
let snapshotTimer = null
let seekTimer = null
let pendingSeekSeconds = null
let seekInFlight = false
let seekCommitRequested = false
const SEEK_END_EPSILON_SECONDS = 0.05
let bassReleaseInFlight = null
let isAppDisposing = false
let unlistenBassEvents = () => { }
let snapshotInFlight = false
let lastRecordedTrackId = null
let lastRecordedPositionMs = -1
const lyricsPayload = ref(null)
const lyricsLoading = ref(false)
let lyricsRequestId = 0

// 搜索查询
const searchQuery = ref('')

// 主题状态
const currentTheme = ref(null)
const isDarkTheme = computed(() => currentTheme.value?.isDark ?? themeManager.isDarkMode)

// 音乐库数据
const musicLibrary = reactive({
  totalSongs: 5,
  totalDuration: "18' 56\"",
  songs: [
    {
      id: 1,
      title: '僕たち、やっと行けるんだね!',
      artist: '菅野祐悟',
      album: 'TVアニメ『はたらく細胞...』',
      duration: '02:04',
      cover: '/assets/cover.jpg'
    },
    {
      id: 2,
      title: '1996 Internet Starter Kit - Velkommen (Original Mix)',
      artist: 'Stan LePard',
      album: '1996 Internet Starter Kit - Velkommen (Original Mix)',
      duration: '05:24',
      cover: '/assets/1996 Internet Starter Kit - Velkommen (Original Mix) - Stan LePard.jpg'
    },
    {
      id: 3,
      title: 'Afternoon Delight',
      artist: 'Starland Vocal Band',
      album: 'AM Gold: Mellow Hits of the \'70s',
      duration: '03:15',
      cover: '/assets/Afternoon Delight - Starland Vocal Band.jpg'
    },
    {
      id: 4,
      title: 'Alright!',
      artist: 'Juju B. Goode; HOHYUN',
      album: 'Garden',
      duration: '03:04',
      cover: '/assets/Alright! - Juju B. Goode, HOHYUN.jpg'
    },
    {
      id: 5,
      title: 'Between Worlds',
      artist: 'Roger Subirana',
      album: 'X I I',
      duration: '05:09',
      cover: '/assets/Between Worlds - Roger Subirana.jpg'
    }
  ]
})

// 侧边栏导航项
const sidebarItems = reactive([
  { id: 'home', icon: 'home.svg', label: '首页', active: true },
  { id: 'library', icon: 'library.svg', label: '库', active: false },
  { id: 'albums', icon: 'album.svg', label: '专辑', active: false },
  { id: 'artists', icon: 'artists.svg', label: '艺术家', active: false },
  { id: 'effects', icon: 'effect.svg', label: '声音效果', active: false },
  { id: 'plugins', icon: 'plugin.svg', label: '扩展插件', active: false }
])

// 数据集合
const albumsData = reactive([
  {
    id: 1,
    title: 'TVアニメ『はたらく細胞BLACK』Original Soundtrack',
    artist: '菅野祐悟',
    year: '2021',
    cover: '/assets/cover.jpg',
    trackCount: 25,
    duration: '1:23:45',
    genres: ['原声', '动画'],
    addedDate: '2024-01-15'
  },
  {
    id: 2,
    title: '1996 Internet Starter Kit',
    artist: 'Stan LePard',
    year: '2023',
    cover: '/assets/1996 Internet Starter Kit - Velkommen (Original Mix) - Stan LePard.jpg',
    trackCount: 12,
    duration: '45:32',
    genres: ['电子', '氛围'],
    addedDate: '2024-01-20'
  },
  {
    id: 3,
    title: 'AM Gold: Mellow Hits of the \'70s',
    artist: 'Starland Vocal Band',
    year: '1976',
    cover: '/assets/Afternoon Delight - Starland Vocal Band.jpg',
    trackCount: 15,
    duration: '52:18',
    genres: ['流行', '复古'],
    addedDate: '2024-01-10'
  },
  {
    id: 4,
    title: 'Garden',
    artist: 'Juju B. Goode; HOHYUN',
    year: '2023',
    cover: '/assets/Alright! - Juju B. Goode, HOHYUN.jpg',
    trackCount: 8,
    duration: '32:15',
    genres: ['独立', '实验'],
    addedDate: '2024-01-25'
  },
  {
    id: 5,
    title: 'X I I',
    artist: 'Roger Subirana',
    year: '2022',
    cover: '/assets/Between Worlds - Roger Subirana.jpg',
    trackCount: 10,
    duration: '38:42',
    genres: ['氛围', '后摇'],
    addedDate: '2024-01-18'
  }
])

const artistsData = reactive([
  {
    id: 1,
    name: '菅野祐悟',
    cover: '/assets/cover.jpg',
    albumCount: 15,
    songCount: 120,
    genres: ['原声', '动画', '电子'],
    followers: 45000,
    isFollowing: true,
    lastPlayed: '2024-01-30'
  },
  {
    id: 2,
    name: 'Stan LePard',
    cover: '/assets/1996 Internet Starter Kit - Velkommen (Original Mix) - Stan LePard.jpg',
    albumCount: 3,
    songCount: 35,
    genres: ['电子', '氛围', '实验'],
    followers: 12000,
    isFollowing: false,
    lastPlayed: '2024-01-28'
  },
  {
    id: 3,
    name: 'Starland Vocal Band',
    cover: '/assets/Afternoon Delight - Starland Vocal Band.jpg',
    albumCount: 8,
    songCount: 95,
    genres: ['流行', '复古', '摇滚'],
    followers: 28000,
    isFollowing: true,
    lastPlayed: '2024-01-25'
  },
  {
    id: 4,
    name: 'Juju B. Goode',
    cover: '/assets/Alright! - Juju B. Goode, HOHYUN.jpg',
    albumCount: 2,
    songCount: 18,
    genres: ['独立', '实验', '电子'],
    followers: 8500,
    isFollowing: false,
    lastPlayed: '2024-01-29'
  },
  {
    id: 5,
    name: 'Roger Subirana',
    cover: '/assets/Between Worlds - Roger Subirana.jpg',
    albumCount: 4,
    songCount: 42,
    genres: ['氛围', '后摇', '器乐'],
    followers: 15000,
    isFollowing: true,
    lastPlayed: '2024-01-27'
  }
])

const homePageData = reactive({
  recentlyPlayed: musicLibrary.songs.slice(0, 6),
  recommendedPlaylists: [
    {
      id: 1,
      name: '深夜氛围',
      description: '适合深夜聆听的宁静音乐',
      cover: '/assets/cover.jpg',
      trackCount: 25
    },
    {
      id: 2,
      name: '电子之境',
      description: '现代电子音乐精选',
      cover: '/assets/Between Worlds - Roger Subirana.jpg',
      trackCount: 18
    },
    {
      id: 3,
      name: '复古金曲',
      description: '70年代经典流行音乐',
      cover: '/assets/Afternoon Delight - Starland Vocal Band.jpg',
      trackCount: 32
    },
    {
      id: 4,
      name: '独立音乐',
      description: '独特声音的实验性音乐',
      cover: '/assets/Alright! - Juju B. Goode, HOHYUN.jpg',
      trackCount: 15
    }
  ]
})

const formatLibraryDuration = (tracks) => {
  const durationMs = tracks.reduce((total, track) => total + (Number(track.durationMs) || 0), 0)
  const totalSeconds = Math.floor(durationMs / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  return hours > 0
    ? `${hours}h ${minutes}m ${seconds}s`
    : `${minutes}m ${seconds}s`
}

const syncLibraryState = () => {
  const tracks = libraryStore.state.tracks
  const songs = libraryStore.tracks.value
  musicLibrary.totalSongs = libraryStore.state.total
  musicLibrary.totalDuration = formatLibraryDuration(tracks)
  musicLibrary.songs.splice(0, musicLibrary.songs.length, ...songs)
  albumsData.splice(0, albumsData.length, ...libraryStore.albums.value)
  artistsData.splice(0, artistsData.length, ...libraryStore.artists.value)
  homePageData.recentlyPlayed.splice(0, homePageData.recentlyPlayed.length, ...songs.slice(0, 6))
  homePageData.recommendedPlaylists.splice(
    0,
    homePageData.recommendedPlaylists.length,
    ...libraryStore.albums.value.slice(0, 4).map((album) => ({
      id: album.id,
      name: album.title,
      description: album.artist || '本地音乐专辑',
      cover: album.cover,
      trackCount: album.trackCount
    }))
  )
  if (songs.length && (!activeChannelId.value || currentSong.value.title === '未选择歌曲')) {
    currentSong.value = { ...songs[0] }
    totalTime.value = songs[0].duration || '00:00'
  }
}

watch(
  [libraryStore.tracks, libraryStore.albums, libraryStore.artists],
  syncLibraryState,
  { deep: true }
)

// 页面切换逻辑
const navigateToPage = (pageId) => {
  if (pageId === currentPage.value) return

  // 缓存当前页面状态
  if (currentPage.value) {
    pageCache.set(currentPage.value, {
      scrollPosition: document.querySelector('.page-layout-scroll')?.scrollTop || 0,
      timestamp: Date.now()
    })
  }

  // 更新导航状态
  sidebarItems.forEach(item => item.active = item.id === pageId)

  // 添加到历史记录
  if (pageHistory.value[pageHistory.value.length - 1] !== pageId) {
    pageHistory.value.push(pageId)
  }

  currentPage.value = pageId

  // 页面节点现在即时切换，下一帧恢复各页自己的滚动位置。
  nextTick(() => {
    restorePageScroll()
    bindScrollSources()
  })

}

const restorePageScroll = () => {
  const cachedState = pageCache.get(currentPage.value)
  if (!cachedState) return

  const mainContent = document.querySelector('.page-layout-scroll')
  if (mainContent) {
    mainContent.scrollTop = cachedState.scrollPosition
  }
}

const bindScrollSources = async () => {
  unbindScrollSources()
  await nextTick()

  const updateTitlebarScroll = () => {
    const sources = [
      document.querySelector('.sidebar-scroll'),
      document.querySelector('.page-layout-scroll')
    ].filter(Boolean)

    isTitlebarScrolled.value = sources.some((source) => source.scrollTop > 0)
  }

  // 使用捕获阶段监听所有滚动容器，页面切换替换滚动节点时无需重新依赖旧节点。
  document.addEventListener('scroll', updateTitlebarScroll, { capture: true, passive: true })
  updateTitlebarScroll()

  unbindScrollSources = () => {
    document.removeEventListener('scroll', updateTitlebarScroll, true)
    unbindScrollSources = () => { }
  }
}

// 当前页面组件
const currentPageComponent = computed(() => {
  return pageComponents[currentPage.value] || HomePage
})

const effectiveQueue = computed(() => {
  return playbackQueue.value?.length ? playbackQueue.value : libraryStore.tracks.value
})

// 事件处理函数
const handleSearchUpdate = (query) => {
  searchQuery.value = query
  console.log('搜索查询:', query)
}

const handleNavItemClick = (item) => {
  navigateToPage(item.id)
  console.log('导航点击:', item.label)
}

const handleHeaderControlClick = async (control) => {
  if (!['system', 'local', 'import'].includes(control.id)) return
  try {
    const result = await libraryStore.mediaApi.pickFolder()
    if (result?.path) await libraryStore.addRootAndScan(result.path)
  } catch (error) {
    console.error('导入音乐目录失败:', error)
  }
}

const handleSongSelect = (song) => {
  currentSong.value = { ...song }
  totalTime.value = song.duration || '00:00'
  updateThemeFromSong(song)
}

const formatSeconds = (seconds) => {
  const totalSeconds = Math.max(0, Math.floor(Number(seconds) || 0))
  const minutes = Math.floor(totalSeconds / 60)
  return `${String(minutes).padStart(2, '0')}:${String(totalSeconds % 60).padStart(2, '0')}`
}

const activeLengthSeconds = ref(0)

const loadLyricsForSong = async (song) => {
  const requestId = ++lyricsRequestId
  lyricsPayload.value = null

  if (!song?.id) {
    lyricsLoading.value = false
    return
  }

  lyricsLoading.value = true
  try {
    const payload = await libraryStore.mediaApi.lyrics(song.id)
    if (requestId === lyricsRequestId && currentSong.value.id === song.id) {
      lyricsPayload.value = payload
    }
  } catch (error) {
    if (requestId === lyricsRequestId) {
      lyricsPayload.value = {
        source: 'none',
        lines: [],
        plainLines: [],
        warnings: [error?.message || '歌词读取失败']
      }
    }
  } finally {
    if (requestId === lyricsRequestId) lyricsLoading.value = false
  }
}

watch(
  () => currentSong.value.id,
  () => loadLyricsForSong(currentSong.value)
)

const savePlaybackProgress = (trackId, positionMs, force = false) => {
  if (!trackId || (!force && trackId === lastRecordedTrackId && Math.abs(positionMs - lastRecordedPositionMs) < 2000)) return
  lastRecordedTrackId = trackId
  lastRecordedPositionMs = positionMs
  void libraryStore.mediaApi.record(trackId, positionMs).catch((error) => {
    console.debug('保存播放进度失败:', error)
  })
}

const refreshPlaybackSnapshot = async (forceRecord = false) => {
  const channelId = activeChannelId.value
  if (!channelId || seekInFlight || pendingSeekSeconds !== null || snapshotInFlight) return
  snapshotInFlight = true
  try {
    const snapshot = await bassCall('bass_channel_snapshot', { channelId })
    if (channelId !== activeChannelId.value) return
    const position = Number(snapshot.positionSeconds || 0)
    activeLengthSeconds.value = Number(snapshot.lengthSeconds || activeLengthSeconds.value || 0)
    currentTimeMs.value = Math.max(0, Math.round(position * 1000))
    currentTime.value = formatSeconds(position)
    totalTime.value = formatSeconds(activeLengthSeconds.value)
    progress.value = activeLengthSeconds.value > 0 ? (position / activeLengthSeconds.value) * 100 : 0
    const wasPlaying = isPlaying.value
    isPlaying.value = snapshot.state === 'playing'
    savePlaybackProgress(currentSong.value.id, currentTimeMs.value, forceRecord)

    if (wasPlaying && snapshot.state === 'stopped' && activeLengthSeconds.value > 0
      && position >= activeLengthSeconds.value - 0.35) {
      void handlePlaybackCompleted()
    }
  } catch (error) {
    if (channelId === activeChannelId.value) console.debug('播放状态更新失败:', error)
  } finally {
    snapshotInFlight = false
  }
}

const handleBassEvent = (eventName, payload) => {
  if (eventName !== 'bass/channel-state' || payload?.channelId !== activeChannelId.value) return
  void refreshPlaybackSnapshot(true)
}

const releaseBassResources = () => {
  if (snapshotTimer) {
    clearInterval(snapshotTimer)
    snapshotTimer = null
  }
  if (seekTimer !== null) {
    window.clearTimeout(seekTimer)
    seekTimer = null
  }
  pendingSeekSeconds = null
  seekCommitRequested = false
  activeChannelId.value = null
  playbackQueue.value = null
  shufflePlayedIds.clear()
  completionInFlight = false
  currentSong.value = {
    title: '未选择歌曲',
    artist: '请先导入音乐目录',
    album: '',
    duration: '00:00',
    cover: '/assets/cover.jpg'
  }
  isPlaying.value = false
  currentTimeMs.value = 0
  currentTime.value = '00:00'
  totalTime.value = '00:00'
  progress.value = 0
  lastRecordedTrackId = null
  lastRecordedPositionMs = -1

  if (!bassReleaseInFlight) {
    bassReleaseInFlight = bassCall('bass_unload')
      .catch((error) => console.debug('释放 BASS 资源失败:', error))
      .finally(() => {
        bassReleaseInFlight = null
      })
  }
  return bassReleaseInFlight
}

const handleWindowExit = () => {
  isAppDisposing = true
  void releaseBassResources()
}

const startPlaybackSnapshot = () => {
  if (snapshotTimer) clearInterval(snapshotTimer)
  snapshotTimer = setInterval(refreshPlaybackSnapshot, 500)
  refreshPlaybackSnapshot()
}

const playSong = async (song, queue = null) => {
  if (isAppDisposing) return
  try {
    playbackQueue.value = queue?.length ? queue : null
    shufflePlayedIds.add(song.id)
    if (seekTimer !== null) {
      window.clearTimeout(seekTimer)
      seekTimer = null
    }
    pendingSeekSeconds = null
    seekCommitRequested = false
    if (activeChannelId.value) {
      await bassCall('bass_channel_stop', { channelId: activeChannelId.value }).catch(() => { })
      await bassCall('bass_channel_close', { channelId: activeChannelId.value }).catch(() => { })
      activeChannelId.value = null
    }
    const result = await libraryStore.openPlayback(song)
    if (isAppDisposing) {
      const channelId = result?.channel?.channelId
      if (channelId) await bassCall('bass_channel_close', { channelId }).catch(() => { })
      return
    }
    currentSong.value = { ...song }
    activeChannelId.value = result?.channel?.channelId || null
    if (activeChannelId.value) {
      await bassCall('bass_channel_play', { channelId: activeChannelId.value, restart: true })
      await bassCall('bass_channel_set_volume', {
        channelId: activeChannelId.value,
        value: muted.value ? 0 : volume.value / 100
      }).catch((error) => console.debug('设置音量失败:', error))
    }
    isPlaying.value = true
    currentTimeMs.value = 0
    currentTime.value = '00:00'
    totalTime.value = song.duration || '00:00'
    lastRecordedTrackId = song.id
    lastRecordedPositionMs = 0
    updateThemeFromSong(song)
    startPlaybackSnapshot()
  } catch (error) {
    console.error('播放歌曲失败:', error)
  }
}

const handleSongPlay = (song) => {
  shufflePlayedIds.clear()
  playSong(song)
}

const handleAlbumSelect = (album) => {
  console.log('选择专辑:', album.title)
  // 可以导航到专辑详情页面
}

const handleAlbumPlay = (album) => {
  const tracks = album?.tracks?.length
    ? album.tracks
        .map((track) => libraryStore.tracks.value.find((song) => song.id === track.id || song.title === track.title))
        .filter(Boolean)
    : libraryStore.tracks.value.filter((track) => track.album === album?.title)
  if (tracks.length) {
    shufflePlayedIds.clear()
    playSong(tracks[0], tracks)
  }
}

const handleArtistSelect = (artist) => {
  console.log('选择艺术家:', artist.name)
  // 可以导航到艺术家详情页面
}

const handleArtistPlay = (artist) => {
  const song = libraryStore.tracks.value.find((track) => track.artist === artist.name)
  if (song) {
    shufflePlayedIds.clear()
    playSong(song)
  }
}

const handleArtistFollow = (artist) => {
  artist.isFollowing = !artist.isFollowing
  console.log(artist.isFollowing ? '关注' : '取消关注', artist.name)
}

const handlePlaylistPlay = (playlist) => {
  const tracks = libraryStore.tracks.value.filter((track) => track.album === playlist.name)
  if (tracks.length) {
    shufflePlayedIds.clear()
    playSong(tracks[0], tracks)
  }
}

const handleNavigate = (pageId) => {
  navigateToPage(pageId)
}

const handleEffectsChange = (effects) => {
  console.log('音效设置更改:', effects)
  // 这里可以实现实际的音频处理逻辑
}

// 从歌曲专辑封面更新主题
const updateThemeFromSong = async (song) => {
  if (song.cover) {
    try {
      const theme = await themeManager.updateThemeFromAlbum(song.cover)
      if (theme) {
        console.log('主题已更新为专辑:', song.album)
      }
    } catch (error) {
      console.error('主题更新失败:', error)
    }
  }
}

const handleTogglePlay = async () => {
  if (!activeChannelId.value) {
    if (libraryStore.tracks.value[0]) await playSong(libraryStore.tracks.value[0])
    return
  }
  try {
    if (isPlaying.value) {
      await bassCall('bass_channel_pause', { channelId: activeChannelId.value })
    } else {
      await bassCall('bass_channel_play', { channelId: activeChannelId.value, restart: false })
    }
    await refreshPlaybackSnapshot()
  } catch (error) {
    console.error('切换播放状态失败:', error)
  }
}

const handlePrevious = () => {
  const songs = playbackQueue.value || libraryStore.tracks.value
  const index = songs.findIndex((song) => song.id === currentSong.value.id)
  if (index >= 0 && songs.length) playSong(songs[(index - 1 + songs.length) % songs.length], playbackQueue.value)
}

const handleNext = () => {
  void advancePlayback(true)
}

const stopAfterPlayback = async () => {
  if (!activeChannelId.value) return
  await bassCall('bass_channel_pause', { channelId: activeChannelId.value }).catch(() => { })
  isPlaying.value = false
  await refreshPlaybackSnapshot(true)
}

const advancePlayback = async (manual = false) => {
  const songs = playbackQueue.value || libraryStore.tracks.value
  if (!songs.length || !currentSong.value?.id) return

  if (playbackMode.value === 'repeat-one') {
    if (!manual && listLoop.value) return playSong(currentSong.value, playbackQueue.value)
    if (!manual) return stopAfterPlayback()
  }

  if (playbackMode.value === 'shuffle') {
    let candidates = songs.filter((song) => !shufflePlayedIds.has(song.id))
    if (!candidates.length) {
      if (!listLoop.value) return stopAfterPlayback()
      shufflePlayedIds.clear()
      shufflePlayedIds.add(currentSong.value.id)
      candidates = songs.filter((song) => song.id !== currentSong.value.id)
    }
    const nextSong = candidates[Math.floor(Math.random() * candidates.length)]
    return playSong(nextSong, playbackQueue.value)
  }

  const index = songs.findIndex((song) => song.id === currentSong.value.id)
  if (index < 0) return
  if (index === songs.length - 1 && !listLoop.value) return stopAfterPlayback()
  return playSong(songs[(index + 1) % songs.length], playbackQueue.value)
}

const handlePlaybackCompleted = async () => {
  if (completionInFlight) return
  completionInFlight = true
  try {
    await advancePlayback()
  } finally {
    completionInFlight = false
  }
}

const handleProgressChange = (percent) => {
  const boundedPercent = Math.max(0, Math.min(100, percent))
  progress.value = boundedPercent
  if (activeChannelId.value && activeLengthSeconds.value > 0) {
    const targetSeconds = activeLengthSeconds.value * boundedPercent / 100
    currentTimeMs.value = Math.round(targetSeconds * 1000)
    currentTime.value = formatSeconds(targetSeconds)
    pendingSeekSeconds = targetSeconds
  }
}

const flushSeek = async () => {
  seekTimer = null
  if (seekInFlight || pendingSeekSeconds === null || !activeChannelId.value || activeLengthSeconds.value <= 0) return

  // BASS rejects a position exactly at the stream length; keep the visual
  // slider at 100% while seeking to the last valid sample instead.
  const maxSeekSeconds = Math.max(0, activeLengthSeconds.value - SEEK_END_EPSILON_SECONDS)
  const targetSeconds = Math.max(0, Math.min(maxSeekSeconds, pendingSeekSeconds))
  const channelId = activeChannelId.value
  pendingSeekSeconds = null
  seekCommitRequested = false
  seekInFlight = true
  try {
    await bassCall('bass_channel_seek', {
      channelId,
      seconds: targetSeconds
    })
    if (channelId === activeChannelId.value) {
      savePlaybackProgress(currentSong.value.id, Math.round(targetSeconds * 1000), true)
    }
  } catch (error) {
    console.error('调整播放进度失败:', error)
  } finally {
    seekInFlight = false
    if (pendingSeekSeconds !== null && seekCommitRequested) scheduleSeek()
  }
}

const scheduleSeek = () => {
  if (seekTimer !== null) return
  seekTimer = window.setTimeout(flushSeek, 0)
}

const handleProgressCommit = (percent) => {
  if (percent === null || percent === undefined) return
  handleProgressChange(percent)
  if (pendingSeekSeconds === null) return
  seekCommitRequested = true
  scheduleSeek()
}

const handleAddTag = () => {
  console.log('添加标签')
}

const handleAddPlaylist = () => {
  console.log('添加播放列表')
}

const handleAddPlugin = () => {
  console.log('添加插件')
}

const handleRepeat = () => {
  playbackMode.value = playbackMode.value === 'repeat-one' ? 'sequential' : 'repeat-one'
}

const handlePlaybackModeChange = (mode) => {
  playbackMode.value = ['sequential', 'shuffle', 'repeat-one'].includes(mode) ? mode : 'sequential'
  if (playbackMode.value === 'shuffle') shufflePlayedIds.add(currentSong.value?.id)
}

const handleListLoopChange = (enabled) => {
  listLoop.value = Boolean(enabled)
}

const handleVolumeChange = async (nextVolume) => {
  volume.value = Math.max(0, Math.min(100, Number(nextVolume) || 0))
  muted.value = false
  if (!activeChannelId.value) return
  await bassCall('bass_channel_set_volume', {
    channelId: activeChannelId.value,
    value: volume.value / 100
  }).catch((error) => console.error('音量变化失败:', error))
}

const handleMuteChange = async (nextMuted) => {
  muted.value = Boolean(nextMuted)
  if (!activeChannelId.value) return
  await bassCall('bass_channel_set_volume', {
    channelId: activeChannelId.value,
    value: muted.value ? 0 : volume.value / 100
  }).catch((error) => console.error('静音切换失败:', error))
}

const handleMenu = () => {
  console.log('菜单')
}

const handleQueue = () => {
  isQueueDrawerOpen.value = true
}

const handleQueueDrawerClose = () => {
  isQueueDrawerOpen.value = false
}

const handleQueueSongSelect = (song) => {
  shufflePlayedIds.clear()
  playSong(song, effectiveQueue.value)
}

const handleAdd = () => {
  console.log('添加')
}

const handleExpandPlayer = () => {
  showFullscreenPlayer.value = true
}

const handleCloseFullscreenPlayer = () => {
  showFullscreenPlayer.value = false
}

const handleBackgroundModeChange = (mode) => {
  fullscreenBackgroundMode.value = mode === 'blur' ? 'blur' : 'flowing'
}

// 获取当前页面所需的props
const getPageProps = () => {
  switch (currentPage.value) {
    case 'home':
      return homePageData
    case 'library':
      return { musicLibrary }
    case 'albums':
      return { albums: albumsData, songs: musicLibrary.songs }
    case 'artists':
      return { artists: artistsData }
    case 'effects':
      return {}
    default:
      return {}
  }
}

const getPageTransitionBlocks = (pageElement) => {
  const header = pageElement.querySelector('.page-layout-header')
  const content = pageElement.querySelector('.page-layout-content')
  const contentRoot = content?.firstElementChild
  const contentBlocks = contentRoot?.children?.length
    ? [...contentRoot.children]
    : [...(content?.children || [])]

  return [header, ...contentBlocks]
    .filter((element, index, elements) => element && elements.indexOf(element) === index)
    .filter((element) => element.getClientRects().length > 0)
}

const beforePageLeave = (pageElement) => {
  pageElement.style.position = 'absolute'
  pageElement.style.inset = '0'
  pageElement.style.width = '100%'
  pageElement.style.height = '100%'
  pageElement.style.pointerEvents = 'none'
  pageElement.style.zIndex = '0'
}

const leavePage = (pageElement, done) => {
  if (reducedMotion.value) {
    done()
    return
  }

  const animations = getPageTransitionBlocks(pageElement).map((element, index) => {
    const animation = animateElement(
      element,
      { opacity: 0, y: -10, filter: 'blur(2px)' },
      { ...APPLE_SPRING, delay: index * 0.025 }
    )
    return animation.finished.catch(() => undefined)
  })

  Promise.all(animations).then(done)
}

const afterPageLeave = (pageElement) => {
  pageElement.style.position = ''
  pageElement.style.inset = ''
  pageElement.style.width = ''
  pageElement.style.height = ''
  pageElement.style.pointerEvents = ''
  pageElement.style.zIndex = ''
}

// 组件挂载时初始化主题
onMounted(async () => {
  isAppDisposing = false
  // A webview refresh can leave the Rust BASS worker alive, so start from a clean native state.
  await releaseBassResources()
  window.addEventListener('beforeunload', handleWindowExit)
  window.addEventListener('pagehide', handleWindowExit)
  unlistenBassEvents = await listenToBassEvents(handleBassEvent)

  // 监听主题变化
  themeManager.addObserver((themeColors) => {
    currentTheme.value = themeColors
    console.log('主题已更新:', themeColors)
  })

  // 从当前歌曲初始化主题
  if (currentSong.value.cover) {
    await updateThemeFromSong(currentSong.value)
  }

  // 先绑定页面滚动，避免桌面端事件初始化失败时影响标题栏状态联动。
  await bindScrollSources()
  await libraryStore.installListeners()
  await libraryStore.refresh()
  await libraryStore.hydrateCovers()
  syncLibraryState()
})

onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', handleWindowExit)
  window.removeEventListener('pagehide', handleWindowExit)
  isAppDisposing = true
  void releaseBassResources()
  unlistenBassEvents()
  unbindScrollSources()
  libraryStore.dispose()
})
</script>

<template>
  <div class="music-player">
    <TitleBar :current-song="currentSong" :is-playing="isPlaying" :current-time="currentTime"
      :current-time-ms="currentTimeMs" :total-time="totalTime" :progress="progress" :lyrics="lyricsPayload"
      :lyrics-loading="lyricsLoading" :is-fullscreen="showFullscreenPlayer" :is-scrolled="isTitlebarScrolled"
      :playback-mode="playbackMode" :list-loop="listLoop"
      :volume="volume" :muted="muted"
      @toggle-play="handleTogglePlay" @previous="handlePrevious" @next="handleNext"
      @progress-change="handleProgressChange" @playback-mode-change="handlePlaybackModeChange"
      @list-loop-change="handleListLoopChange" @volume-change="handleVolumeChange" @mute-change="handleMuteChange"
      @queue="handleQueue"
      @progress-commit="handleProgressCommit" @expand-player="handleExpandPlayer" />

    <!-- 侧边栏 -->
    <Sidebar :sidebar-items="sidebarItems" :current-page="currentPage" :search-query="searchQuery"
      :is-dark="isDarkTheme" @search-update="handleSearchUpdate" @nav-item-click="handleNavItemClick"
      @add-tag="handleAddTag" @add-playlist="handleAddPlaylist" @add-plugin="handleAddPlugin" />

    <!-- 主内容区 -->
    <div class="main-content-wrapper">
      <Transition mode="sync" :css="false" @before-leave="beforePageLeave" @leave="leavePage"
        @after-leave="afterPageLeave">
        <KeepAlive :max="5">
          <component :is="currentPageComponent" :key="currentPage" v-bind="getPageProps()"
            @song-select="handleSongSelect" @song-play="handleSongPlay" @album-select="handleAlbumSelect"
            @album-play="handleAlbumPlay" @artist-select="handleArtistSelect" @artist-play="handleArtistPlay"
            @artist-follow="handleArtistFollow" @playlist-play="handlePlaylistPlay" @navigate="handleNavigate"
            @header-control-click="handleHeaderControlClick" @effects-change="handleEffectsChange" />
        </KeepAlive>
      </Transition>
    </div>

    <!-- 右侧详情面板 -->
    <!-- <DetailPanel :current-song="currentSong" /> -->

    <!-- 全屏播放器层；顶部迷你播放器由 TitleBar 承载 -->
    <PlayerSurface :current-song="currentSong" :is-playing="isPlaying" :current-time="currentTime"
      :current-time-ms="currentTimeMs" :total-time="totalTime" :progress="progress" :lyrics="lyricsPayload"
      :lyrics-loading="lyricsLoading" :is-fullscreen="showFullscreenPlayer" :channel-id="activeChannelId"
      :background-mode="fullscreenBackgroundMode" :queue-songs="effectiveQueue"
      :volume="volume" :muted="muted" :playback-mode="playbackMode" :list-loop="listLoop"
      @close="handleCloseFullscreenPlayer" @toggle-play="handleTogglePlay"
      @previous="handlePrevious" @next="handleNext" @progress-change="handleProgressChange"
      @progress-commit="handleProgressCommit" @volume-change="handleVolumeChange" @mute-change="handleMuteChange"
      @playback-mode-change="handlePlaybackModeChange" @list-loop-change="handleListLoopChange"
      @add-to-playlist="() => console.log('添加到播放列表')"
      @playlist-song-select="handleQueueSongSelect" @background-mode-change="handleBackgroundModeChange" />

    <Drawer :open="isQueueDrawerOpen" title="正在播放" placement="right" close-label="关闭播放列表"
      @close="handleQueueDrawerClose">
      <Playlist :songs="effectiveQueue" :current-song="currentSong" :is-playing="isPlaying"
        @song-select="handleQueueSongSelect" />
    </Drawer>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

/* 播放器界面以点击和拖拽为主，避免交互时误选中界面文字。 */
.music-player *,
.title-bar-player *,
.drawer-layer * {
  -webkit-user-select: none;
  user-select: none;
}

.music-player input,
.music-player textarea,
.music-player [contenteditable='true'],
.title-bar-player input,
.title-bar-player textarea,
.title-bar-player [contenteditable='true'],
.drawer-layer input,
.drawer-layer textarea,
.drawer-layer [contenteditable='true'] {
  -webkit-user-select: text;
  user-select: text;
}

html,
body,
button,
input,
textarea,
select {
  font-family: MiSans, 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

:root {
  --primary-color: 136, 208, 236;
  --primary-hover-color: 0, 86, 179;
  --background-color: 254, 251, 255;
  --surface-color: 254, 251, 255;
  --text-color: 28, 27, 31;
  --secondary-color: 80, 96, 110;
  --outline-color: 121, 116, 126;
  --global-color: 255, 255, 255;
  --global-inverse-color: 0, 0, 0;

  --md-sys-color-primary: 136, 208, 236;
  --md-sys-color-primary-container: 204, 231, 255;
  --md-sys-color-on-primary: 255, 255, 255;
  --md-sys-color-on-primary-container: 0, 30, 46;

  --md-sys-color-secondary: 80, 96, 110;
  --md-sys-color-secondary-container: 211, 228, 244;
  --md-sys-color-on-secondary: 255, 255, 255;
  --md-sys-color-on-secondary-container: 12, 25, 37;

  --md-sys-color-background: 254, 251, 255;
  --md-sys-color-on-background: 28, 27, 31;
  --md-sys-color-surface: 254, 251, 255;
  --md-sys-color-on-surface: 28, 27, 31;

  --md-sys-color-outline: 121, 116, 126;
  --md-sys-color-outline-variant: 202, 182, 224;
}

@media (prefers-color-scheme: dark) {
  :root {
    --global-color: 0, 0, 0;
    --global-inverse-color: 255, 255, 255;
  }
}

body {
  background: rgb(var(--background-color));
  color: rgb(var(--text-color));
  overflow: hidden;
}

/* Banner 样式 */
.music-banner {
  width: 100%;
  height: 300px;
  position: relative;
  overflow: hidden;
  border-radius: 0 0 18px 18px;
  isolation: isolate;
}

.image-container {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  border-radius: inherit;
}

.image-container img {
  -webkit-mask-image: linear-gradient(rgba(0, 0, 0, 0.1), transparent);
  mask-image: linear-gradient(rgba(0, 0, 0, 0.1), transparent);
  filter: blur(5px);
}

.music-banner .background-image {
  width: 100%;
  height: 300px;
  display: block;
  object-fit: cover;
}

.banner-content {
  position: absolute;
  inset: 0 auto auto 0;
  padding: 22px 28px;
}

.banner-content .title {
  font-size: 14px;
  font-weight: bold;
  color: rgb(var(--text-color), 0.4);
  opacity: 0.5;
  margin-bottom: 8px;
}

.library-title {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 8px 0;
}

.banner-content .description {
  margin-bottom: 15px;
  color: rgb(var(--text-color), 0.4);
  font-size: 14px;
  /* text-shadow: 0 1px 5px rgba(0, 0, 0, 0.7); */
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: rgba(var(--outline-color), 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb {
  background: rgba(var(--outline-color), 0.3);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--outline-color), 0.5);
}

#app {
  height: 100vh;
  height: 100dvh;
  width: 100vw;
}
</style>

<style scoped>
.music-player {
  display: grid;
  grid-template-columns: 280px 1fr;
  grid-template-rows: 1fr;
  grid-template-areas: "sidebar main";
  height: 100vh;
  height: 100dvh;
  background: color-mix(in srgb, rgba(var(--background-color)), rgb(var(--global-color)) 40%);
  color: rgb(var(--text-color));
  font-family: MiSans, 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.main-content-wrapper {
  grid-area: main;
  position: absolute;
  min-width: 0;
  left: 300px;
  width: calc(100% - 300px);
  top: 0;
  height: 100%;
  overflow: hidden;
}
</style>
