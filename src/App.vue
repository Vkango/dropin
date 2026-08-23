<script setup>
import { ref, reactive, onMounted, onBeforeUnmount, provide, computed, watch, nextTick } from 'vue'
import Sidebar from './components/Sidebar.vue'
import MusicLibrary from './components/MusicLibrary.vue'
import HomePage from './components/HomePage.vue'
import AlbumsPage from './components/AlbumsPage.vue'
import ArtistsPage from './components/ArtistsPage.vue'
import SoundEffectsPage from './components/SoundEffectsPage.vue'
import PluginsPage from './components/PluginsPage.vue'
import SettingsPage from './components/SettingsPage.vue'
import DetailPanel from './components/DetailPanel.vue'
import PlayerSurface from './components/PlayerSurface.vue'
import TitleBar from './components/TitleBar.vue'
import Drawer from './components/Drawer.vue'
import Playlist from './components/Playlist.vue'
import Notification from './components/notification/Notification.vue'
import LoadingWithTip from './components/notification/LoadingWithTip.vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { themeManager } from './utils/themeManager.js'
import { bassCall, listenToBassEvents } from './services/bassApi.js'
import { createBassEffectsRuntime } from './services/bassEffectsRuntime.js'
import { createBassPlaybackRuntime } from './services/bassPlaybackRuntime.js'
import { smtcApi, listenToSmtcEvents } from './services/smtcApi.js'
import { useLibraryStore } from './stores/libraryStore.js'
import { useAppSettingsStore } from './stores/appSettingsStore.js'
import { useI18n } from './i18n/index.js'
import { activateLocale } from './stores/i18nStore.js'
import { animateElement, APPLE_SPRING, INSTANT_MOTION, SOFT_SPRING } from './utils/motion.js'

const libraryStore = useLibraryStore()
const settingsStore = useAppSettingsStore()
const effectsRuntime = createBassEffectsRuntime(settingsStore)
const playbackRuntime = createBassPlaybackRuntime(settingsStore)
const { t } = useI18n()

// 当前页面状态
const currentPage = ref('home')
const reducedMotion = useReducedMotion()
const pageHistory = ref(['home'])
const pageCache = reactive(new Map())

// 全屏播放器状态
const showFullscreenPlayer = ref(false)
const isTitlebarScrolled = ref(false)
let unbindScrollSources = () => { }

// 侧边栏布局：宽度持久化到 settings.json；窗口过窄时切换为抽屉
const SIDEBAR_MIN_WIDTH = 200
const SIDEBAR_MAX_WIDTH = 480
const DEFAULT_SIDEBAR_WIDTH = 280
const DRAWER_BREAKPOINT = 860
const sidebarWidth = ref(
  Number.isFinite(settingsStore.state.sidebarWidth)
    ? Math.max(SIDEBAR_MIN_WIDTH, Math.min(SIDEBAR_MAX_WIDTH, settingsStore.state.sidebarWidth))
    : DEFAULT_SIDEBAR_WIDTH
)
const isSidebarDrawer = ref(
  typeof window !== 'undefined' && window.matchMedia(`(max-width: ${DRAWER_BREAKPOINT}px)`).matches
)
const isSidebarDrawerOpen = ref(false)
let sidebarDrawerMediaQuery = null
let sidebarDrawerQueryHandler = null

const toggleSidebarDrawer = () => {
  isSidebarDrawerOpen.value = !isSidebarDrawerOpen.value
}

const closeSidebarDrawer = () => {
  isSidebarDrawerOpen.value = false
}

const updateSidebarWidth = (value) => {
  const next = Math.max(SIDEBAR_MIN_WIDTH, Math.min(SIDEBAR_MAX_WIDTH, Math.round(Number(value) || 0)))
  sidebarWidth.value = next
  settingsStore.updateSidebarWidth(next)
}

const handleSidebarResize = (value) => {
  sidebarWidth.value = Math.max(SIDEBAR_MIN_WIDTH, Math.min(SIDEBAR_MAX_WIDTH, Math.round(Number(value) || 0)))
}

const handleSidebarResizeCommit = (value) => {
  updateSidebarWidth(value)
}

// 抽屉弹出动画：Apple Spring
const MotionDiv = motion.div
const sidebarDrawerTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const sidebarScrimTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)

// 页面组件映射
const pageComponents = {
  home: HomePage,
  library: MusicLibrary,
  albums: AlbumsPage,
  artists: ArtistsPage,
  effects: SoundEffectsPage,
  plugins: PluginsPage,
  settings: SettingsPage
}

// 当前播放的歌曲
const currentSong = ref({
  title: t('player.noSongSelected'),
  artist: t('player.importHint'),
  album: '',
  duration: '00:00',
  cover: '/assets/cover.jpg'
})

const idleSong = () => ({
  title: t('player.noSongSelected'),
  artist: t('player.importHint'),
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
const volume = computed(() => settingsStore.state.volume)
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
let unlistenSmtcEvents = () => { }
let snapshotInFlight = false
let lastRecordedTrackId = null
let lastRecordedPositionMs = -1
let smtcMediaRequestId = 0
const lyricsPayload = ref(null)
const lyricsLoading = ref(false)
let lyricsRequestId = 0

// 通知系统
const notificationRef = ref(null)
provide('notification', notificationRef)
let activeProgressNotifyId = null

// 搜索查询
const searchQuery = ref('')

// 主题状态
const currentTheme = ref(null)
const isDarkTheme = computed(() => currentTheme.value?.isDark ?? themeManager.isDarkMode)
const applyThemeSettings = () => {
  themeManager.configure({
    themeMode: settingsStore.state.themeMode,
    autoAlbumTheme: settingsStore.state.autoAlbumTheme,
    manualThemeColor: settingsStore.state.manualThemeColor
  })
}
const handleThemeChange = (themeColors) => {
  currentTheme.value = themeColors
}

watch(
  () => [
    settingsStore.state.themeMode,
    settingsStore.state.autoAlbumTheme,
    settingsStore.state.manualThemeColor
  ],
  async ([themeMode, autoAlbumTheme, manualThemeColor], previous) => {
    themeManager.configure({ themeMode, autoAlbumTheme, manualThemeColor })
    if (autoAlbumTheme && previous?.[1] === false && currentSong.value.cover) {
      await updateThemeFromSong(currentSong.value)
    }
  }
)

watch(
  () => settingsStore.state.language,
  (language) => {
    void activateLocale(language)
  }
)

// 音乐库数据（由 libraryStore 驱动，初始为空）
const musicLibrary = reactive({
  totalSongs: 0,
  totalDuration: '0s',
  songs: []
})

// 侧边栏导航项（标签随语言切换响应更新）
const sidebarItems = reactive([
  { id: 'home', icon: 'home.svg', label: t('sidebar.home'), active: true },
  { id: 'library', icon: 'library.svg', label: t('sidebar.library'), active: false },
  { id: 'albums', icon: 'album.svg', label: t('sidebar.albums'), active: false },
  { id: 'artists', icon: 'artists.svg', label: t('sidebar.artists'), active: false },
  { id: 'effects', icon: 'effect.svg', label: t('sidebar.effects'), active: false },
  { id: 'plugins', icon: 'plugin.svg', label: t('sidebar.plugins'), active: false }
])

watch(
  () => useI18n().locale.value,
  () => {
    const labels = {
      home: t('sidebar.home'),
      library: t('sidebar.library'),
      albums: t('sidebar.albums'),
      artists: t('sidebar.artists'),
      effects: t('sidebar.effects'),
      plugins: t('sidebar.plugins')
    }
    sidebarItems.forEach((item) => {
      if (labels[item.id]) item.label = labels[item.id]
    })
  }
)

// 数据集合
const albumsData = reactive([])

// 主内容区左边距：跟随侧边栏宽度（抽屉模式下为 0）
const mainContentStyle = computed(() =>
  isSidebarDrawer.value ? { left: '0px', width: '100%' } : { left: `${sidebarWidth.value}px`, width: `calc(100% - ${sidebarWidth.value}px)` }
)
const sidebarWidthStyle = computed(() => ({ width: `${sidebarWidth.value}px` }))

const artistsData = reactive([])

const homePageData = reactive({
  recentlyPlayed: [],
  recommendedPlaylists: []
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
  if (songs.length && (!activeChannelId.value || currentSong.value.title === t('player.noSongSelected'))) {
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
  if (isSidebarDrawer.value) closeSidebarDrawer()
  console.log('导航点击:', item.label)
}

const handleScanNotify = (name, payload) => {
  if (!notificationRef.value) return
  const notify = notificationRef.value
  if (name === 'media/scan-progress' && payload?.jobId) {
    if (!activeProgressNotifyId) return
    if (payload.state === 'running') {
      notify.updateNotification(activeProgressNotifyId, {
        props: {
          Tip: t('notification.scanProgress', {
            imported: payload.imported ?? 0,
            scanned: payload.scanned ?? 0,
            skipped: payload.skipped ?? 0,
            failed: payload.failed ?? 0
          })
        }
      })
    }
  } else if (name === 'media/scan-finished' && activeProgressNotifyId) {
    const id = activeProgressNotifyId
    activeProgressNotifyId = null
    if (payload.state === 'cancelled') {
      notify.updateNotification(id, {
        title: t('notification.importing'),
        props: { Tip: t('notification.scanStarted') },
        duration: 4000
      })
    } else {
      notify.updateNotification(id, {
        title: t('notification.importDone'),
        props: {
          Tip: t('notification.scanFinished', {
            imported: payload.imported ?? 0,
            skipped: payload.skipped ?? 0,
            failed: payload.failed ?? 0
          })
        },
        duration: 6000
      })
    }
  } else if (name === 'media/error' && activeProgressNotifyId) {
    const id = activeProgressNotifyId
    activeProgressNotifyId = null
    notify.updateNotification(id, {
      title: t('notification.importFailed'),
      props: { Tip: String(payload?.error?.message || payload?.error || 'unknown error') },
      duration: 8000
    })
  }
}

const handleHeaderControlClick = async (control) => {
  if (!['system', 'local', 'import'].includes(control.id)) return
  try {
    const result = await libraryStore.mediaApi.pickFolder()
    if (!result?.path) return
    if (notificationRef.value) {
      activeProgressNotifyId = await notificationRef.value.addNotification(
        t('notification.importing'),
        t('notification.source'),
        LoadingWithTip,
        null,
        { Tip: t('notification.scanStarted') },
        0
      )
    }
    await libraryStore.addRootAndScan(result.path)
  } catch (error) {
    console.error('导入音乐目录失败:', error)
    if (activeProgressNotifyId && notificationRef.value) {
      const id = activeProgressNotifyId
      activeProgressNotifyId = null
      notificationRef.value.updateNotification(id, {
        title: t('notification.importFailed'),
        props: { Tip: String(error?.message || error) },
        duration: 8000
      })
    }
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
        warnings: [error?.message || t('player.noLyrics')]
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

const smtcDurationMs = (song = currentSong.value) => {
  const trackDuration = Number(song?.durationMs)
  if (Number.isFinite(trackDuration) && trackDuration > 0) return Math.round(trackDuration)
  return Math.max(0, Math.round(Number(activeLengthSeconds.value || 0) * 1000))
}

const safeSmtcCall = (promise, label) => {
  void promise.catch((error) => console.debug(`SMTC ${label} failed:`, error))
}

const syncSmtcTimeline = (positionMs = currentTimeMs.value, durationMs = smtcDurationMs()) => {
  if (!activeChannelId.value) return
  safeSmtcCall(smtcApi.setTimeline(positionMs, durationMs), 'timeline update')
}

const syncSmtcPlaybackStatus = () => {
  safeSmtcCall(smtcApi.setPlaybackStatus(isPlaying.value), 'playback status update')
}

const syncSmtcMedia = async (song) => {
  const requestId = ++smtcMediaRequestId
  let thumbnailPath = null
  if (song?.coverId) {
    try {
      const payload = await libraryStore.mediaApi.coverPath(song.coverId)
      thumbnailPath = payload?.path || null
    } catch (error) {
      console.debug('SMTC cover path lookup failed:', error)
    }
  }
  if (requestId !== smtcMediaRequestId || currentSong.value.id !== song?.id) return
  safeSmtcCall(smtcApi.setMediaInfo({
    title: song?.title || t('player.unknownSong'),
    artist: song?.artist || '',
    album: song?.album || '',
    thumbnailPath
  }), 'media info update')
  syncSmtcTimeline(0, smtcDurationMs(song))
  syncSmtcPlaybackStatus()
}

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
    syncSmtcPlaybackStatus()
    syncSmtcTimeline(currentTimeMs.value, activeLengthSeconds.value * 1000)
    savePlaybackProgress(currentSong.value.id, currentTimeMs.value, forceRecord)

    const reversePlayback = settingsStore.state.playback?.reverse === true
    const reachedPlaybackEnd = reversePlayback
      ? position <= 0.35
      : position >= activeLengthSeconds.value - 0.35
    if (wasPlaying && snapshot.state === 'stopped' && activeLengthSeconds.value > 0
      && reachedPlaybackEnd) {
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

const releaseBassResources = async () => {
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
  await effectsRuntime.closeHandles()
  playbackRuntime.close()
  activeChannelId.value = null
  playbackQueue.value = null
  shufflePlayedIds.clear()
  completionInFlight = false
  currentSong.value = idleSong()
  isPlaying.value = false
  currentTimeMs.value = 0
  currentTime.value = '00:00'
  totalTime.value = '00:00'
  progress.value = 0
  safeSmtcCall(smtcApi.setMediaInfo({
    title: 'Dropin',
    artist: 'Player',
    album: '',
    thumbnailPath: null
  }), 'idle media info reset')
  safeSmtcCall(smtcApi.setPlaybackStatus(false), 'playback status reset')
  safeSmtcCall(smtcApi.setTimeline(0, 0), 'timeline reset')
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
      await effectsRuntime.closeHandles()
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
      await playbackRuntime.prepareChannel(activeChannelId.value)
      await effectsRuntime.applyToChannel(activeChannelId.value)
      await bassCall('bass_channel_play', { channelId: activeChannelId.value, restart: true })
      await effectsRuntime.setVolume(activeChannelId.value, volume.value, muted.value)
        .catch((error) => console.debug('设置音量失败:', error))
    }
    isPlaying.value = true
    currentTimeMs.value = 0
    currentTime.value = '00:00'
    totalTime.value = song.duration || '00:00'
    lastRecordedTrackId = song.id
    lastRecordedPositionMs = 0
    void syncSmtcMedia(song)
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

const handleSmtcEvent = (event) => {
  switch (event) {
    case 'play':
      if (!activeChannelId.value) {
        if (libraryStore.tracks.value[0]) void playSong(libraryStore.tracks.value[0])
      } else if (!isPlaying.value) {
        void handleTogglePlay()
      }
      break
    case 'pause':
      if (isPlaying.value) void handleTogglePlay()
      break
    case 'stop':
      void stopAfterPlayback()
      break
    case 'previous':
      handlePrevious()
      break
    case 'next':
      handleNext()
      break
    default:
      break
  }
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
    syncSmtcTimeline(currentTimeMs.value, activeLengthSeconds.value * 1000)
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

const handleAddTag = async () => {
  const label = prompt('新标签名称（如：BPM 128、轻柔）')
  if (!label || !label.trim()) return
  try {
    await libraryStore.createTag(label.trim())
  } catch (error) {
    console.error('创建标签失败:', error)
  }
}

const handleAddPlaylist = async () => {
  const name = prompt('新播放列表名称')
  if (!name || !name.trim()) return
  try {
    await libraryStore.createPlaylist(name.trim())
  } catch (error) {
    console.error('创建播放列表失败:', error)
  }
}

const handleSelectPlaylist = async (playlist) => {
  try {
    const tracks = await libraryStore.playlistTracks(playlist.id)
    if (tracks.length) {
      shufflePlayedIds.clear()
      playSong(tracks[0], tracks)
    }
  } catch (error) {
    console.error('加载播放列表失败:', error)
  }
}

const handleSelectTag = async (tag) => {
  try {
    const tracks = tag ? await libraryStore.tracksByTag(tag.id) : libraryStore.tracks.value
    if (tracks.length) {
      shufflePlayedIds.clear()
      playSong(tracks[0], tracks)
    }
  } catch (error) {
    console.error('加载标签歌曲失败:', error)
  }
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
  settingsStore.updateVolume(nextVolume)
  muted.value = false
  if (!activeChannelId.value) return
  await effectsRuntime.setVolume(activeChannelId.value, volume.value, false)
    .catch((error) => console.error('音量变化失败:', error))
}

const handleMuteChange = async (nextMuted) => {
  muted.value = Boolean(nextMuted)
  if (!activeChannelId.value) return
  await effectsRuntime.setVolume(activeChannelId.value, volume.value, muted.value)
    .catch((error) => console.error('静音切换失败:', error))
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

const handleSidebarResizeStart = (event) => {
  if (event.button !== 0) return
  event.preventDefault()

  const onMove = (moveEvent) => {
    handleSidebarResize(moveEvent.clientX)
  }
  const onUp = () => {
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
    window.removeEventListener('pointercancel', onUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    handleSidebarResizeCommit(sidebarWidth.value)
  }

  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
  window.addEventListener('pointercancel', onUp)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
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
      return { effectsRuntime, playbackRuntime }
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
  await effectsRuntime.loadCatalog()
  window.addEventListener('beforeunload', handleWindowExit)
  window.addEventListener('pagehide', handleWindowExit)

  // 窗口过窄时侧边栏切换为抽屉
  sidebarDrawerMediaQuery = window.matchMedia(`(max-width: ${DRAWER_BREAKPOINT}px)`)
  sidebarDrawerQueryHandler = (event) => {
    isSidebarDrawer.value = event.matches
    if (!event.matches) isSidebarDrawerOpen.value = false
  }
  sidebarDrawerMediaQuery.addEventListener('change', sidebarDrawerQueryHandler)

  unlistenBassEvents = await listenToBassEvents(handleBassEvent)
  unlistenSmtcEvents = await listenToSmtcEvents(handleSmtcEvent)

  // 监听主题变化
  themeManager.addObserver(handleThemeChange)
  applyThemeSettings()
  currentTheme.value = themeManager.getCurrentColors()

  // 从当前歌曲初始化主题
  if (currentSong.value.cover) {
    await updateThemeFromSong(currentSong.value)
  }

  // 先绑定页面滚动，避免桌面端事件初始化失败时影响标题栏状态联动。
  await bindScrollSources()
  await libraryStore.installListeners(handleScanNotify)
  await libraryStore.refresh()
  await libraryStore.hydrateCovers()
  syncLibraryState()
})

onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', handleWindowExit)
  window.removeEventListener('pagehide', handleWindowExit)
  if (sidebarDrawerMediaQuery && sidebarDrawerQueryHandler) {
    sidebarDrawerMediaQuery.removeEventListener('change', sidebarDrawerQueryHandler)
    sidebarDrawerMediaQuery = null
    sidebarDrawerQueryHandler = null
  }
  isAppDisposing = true
  void releaseBassResources()
  unlistenBassEvents()
  unlistenSmtcEvents()
  themeManager.removeObserver(handleThemeChange)
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
      :volume="volume" :muted="muted" :is-drawer="isSidebarDrawer" :is-drawer-open="isSidebarDrawerOpen"
      @toggle-play="handleTogglePlay" @previous="handlePrevious" @next="handleNext"
      @progress-change="handleProgressChange" @playback-mode-change="handlePlaybackModeChange"
      @list-loop-change="handleListLoopChange" @volume-change="handleVolumeChange" @mute-change="handleMuteChange"
      @queue="handleQueue" @menu="toggleSidebarDrawer"
      @progress-commit="handleProgressCommit" @expand-player="handleExpandPlayer" />

    <!-- 宽屏常驻侧边栏：无背景、无分割线，右缘可拖拽调宽（写入 settings.json） -->
    <div v-if="!isSidebarDrawer" class="sidebar-shell" :style="sidebarWidthStyle">
      <Sidebar :sidebar-items="sidebarItems" :current-page="currentPage" :search-query="searchQuery"
        :is-dark="isDarkTheme" :playlists="libraryStore.playlists.value" :tags="libraryStore.tags.value"
        @search-update="handleSearchUpdate" @nav-item-click="handleNavItemClick"
        @add-tag="handleAddTag" @add-playlist="handleAddPlaylist" @add-plugin="handleAddPlugin"
        @select-playlist="handleSelectPlaylist" @select-tag="handleSelectTag" />

      <div class="sidebar-resize-handle" title="拖动调整侧边栏宽度" @pointerdown="handleSidebarResizeStart" />
    </div>

    <!-- 窄屏抽屉式侧边栏：背景色 + 背景模糊，Apple Spring 弹出 -->
    <Teleport to="body">
      <AnimatePresence>
        <MotionDiv v-if="isSidebarDrawer && isSidebarDrawerOpen" class="sidebar-drawer-layer"
          role="presentation" @click.self="closeSidebarDrawer">
          <MotionDiv class="sidebar-drawer-scrim" :initial="{ opacity: 0 }" :animate="{ opacity: 1 }"
            :exit="{ opacity: 0 }" :transition="sidebarScrimTransition" aria-hidden="true"
            @click="closeSidebarDrawer" />
          <MotionDiv class="sidebar-drawer-panel" :style="sidebarWidthStyle"
            :initial="{ x: '-100%', opacity: 0.4 }" :animate="{ x: 0, opacity: 1 }"
            :exit="{ x: '-100%', opacity: 0.4 }" :transition="sidebarDrawerTransition">
            <Sidebar :sidebar-items="sidebarItems" :current-page="currentPage" :search-query="searchQuery"
              :is-dark="isDarkTheme" :playlists="libraryStore.playlists.value" :tags="libraryStore.tags.value"
              :is-drawer="true"
              @search-update="handleSearchUpdate" @nav-item-click="handleNavItemClick"
              @add-tag="handleAddTag" @add-playlist="handleAddPlaylist" @add-plugin="handleAddPlugin"
              @select-playlist="handleSelectPlaylist" @select-tag="handleSelectTag"
              @collapse="closeSidebarDrawer" />
          </MotionDiv>
        </MotionDiv>
      </AnimatePresence>
    </Teleport>

    <!-- 主内容区 -->
    <div class="main-content-wrapper" :style="mainContentStyle">
      <Transition mode="sync" :css="false" @before-leave="beforePageLeave" @leave="leavePage"
        @after-leave="afterPageLeave">
        <KeepAlive :max="5">
          <component :is="currentPageComponent" :key="currentPage" v-bind="getPageProps()"
            @song-select="handleSongSelect" @song-play="handleSongPlay" @album-select="handleAlbumSelect"
            @album-play="handleAlbumPlay" @artist-select="handleArtistSelect" @artist-play="handleArtistPlay"
            @artist-follow="handleArtistFollow" @playlist-play="handlePlaylistPlay" @navigate="handleNavigate"
            @header-control-click="handleHeaderControlClick" />
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

    <Drawer :open="isQueueDrawerOpen" :title="t('player.queueTitle')" placement="right"
      :close-label="t('player.closePlaylist')"
      @close="handleQueueDrawerClose">
      <Playlist :songs="effectiveQueue" :current-song="currentSong" :is-playing="isPlaying"
        @song-select="handleQueueSongSelect" />
    </Drawer>

    <Notification ref="notificationRef" class="app-notification-layer" />
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

  /* 白色图标资源（close.svg / sys_music.svg 等）在亮色主题下反转为深色 */
  --invert: 1;
}

@media (prefers-color-scheme: dark) {
  :root {
    --global-color: 0, 0, 0;
    --global-inverse-color: 255, 255, 255;
    --invert: 0;
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
  display: block;
  position: relative;
  height: 100vh;
  height: 100dvh;
  overflow: hidden;
  background: color-mix(in srgb, rgba(var(--background-color)), rgb(var(--global-color)) 40%);
  color: rgb(var(--text-color));
  font-family: MiSans, 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.sidebar-shell {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 40;
  height: 100%;
  min-width: 0;
  display: flex;
}

.sidebar-shell .sidebar-scroll {
  flex: 1 1 auto;
  width: auto;
  min-width: 0;
}

/* 尺寸调节分割线：平时与调节中均不显示，仅靠光标提示 */
.sidebar-shell .sidebar-resize-handle {
  flex: 0 0 5px;
  cursor: col-resize;
  border: transparent;
  background: transparent;
}

.sidebar-drawer-layer {
  position: fixed;
  inset: 0;
  z-index: 1150;
  display: flex;
  pointer-events: auto;
}

.sidebar-drawer-scrim {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.28);
  will-change: opacity;
}

.sidebar-drawer-panel {
  position: relative;
  z-index: 1;
  height: 100%;
  min-width: 0;
  display: flex;
  overflow: hidden;
  color: rgb(var(--text-color));
  background: color-mix(in srgb, rgb(var(--surface-color)) 90%, rgb(var(--global-color)) 10%);
  backdrop-filter: blur(18px) saturate(1.12);
  -webkit-backdrop-filter: blur(18px) saturate(1.12);
  box-shadow: 14px 0 40px rgba(0, 0, 0, 0.22);
  will-change: transform, opacity;
}

.sidebar-drawer-panel .sidebar-scroll {
  flex: 1 1 auto;
  width: auto;
  min-width: 0;
}

.main-content-wrapper {
  position: absolute;
  min-width: 0;
  top: 0;
  height: 100%;
  overflow: hidden;
}

.app-notification-layer {
  position: fixed;
  top: 80px;
  right: 16px;
  z-index: 1200;
  width: 320px;
  max-width: calc(100vw - 32px);
  pointer-events: none;
}

.app-notification-layer :deep(.notification) {
  pointer-events: auto;
}
</style>
