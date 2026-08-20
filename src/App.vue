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
import MotionTransition from './components/MotionTransition.vue'
import { themeManager } from './utils/themeManager.js'
import { bassCall } from './services/bassApi.js'
import { useLibraryStore } from './stores/libraryStore.js'

const libraryStore = useLibraryStore()

// 当前页面状态
const currentPage = ref('home')
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
const fullscreenBackgroundMode = ref('flowing')
let snapshotTimer = null
let seekTimer = null
let pendingSeekSeconds = null
let seekInFlight = false
let seekCommitRequested = false
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

const handlePageEntered = () => {
  restorePageScroll()
  bindScrollSources()
}

// 当前页面组件
const currentPageComponent = computed(() => {
  return pageComponents[currentPage.value] || HomePage
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

const refreshPlaybackSnapshot = async () => {
  if (!activeChannelId.value || seekInFlight || pendingSeekSeconds !== null) return
  try {
    const snapshot = await bassCall('bass_channel_snapshot', { channelId: activeChannelId.value })
    const position = Number(snapshot.positionSeconds || 0)
    activeLengthSeconds.value = Number(snapshot.lengthSeconds || activeLengthSeconds.value || 0)
    currentTimeMs.value = Math.max(0, Math.round(position * 1000))
    currentTime.value = formatSeconds(position)
    totalTime.value = formatSeconds(activeLengthSeconds.value)
    progress.value = activeLengthSeconds.value > 0 ? (position / activeLengthSeconds.value) * 100 : 0
    isPlaying.value = snapshot.state === 'playing'
  } catch (error) {
    if (activeChannelId.value) console.debug('播放状态更新失败:', error)
  }
}

const startPlaybackSnapshot = () => {
  if (snapshotTimer) clearInterval(snapshotTimer)
  snapshotTimer = setInterval(refreshPlaybackSnapshot, 500)
  refreshPlaybackSnapshot()
}

const playSong = async (song) => {
  try {
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
    currentSong.value = { ...song }
    activeChannelId.value = result?.channel?.channelId || null
    isPlaying.value = true
    currentTimeMs.value = 0
    currentTime.value = '00:00'
    totalTime.value = song.duration || '00:00'
    updateThemeFromSong(song)
    startPlaybackSnapshot()
  } catch (error) {
    console.error('播放歌曲失败:', error)
  }
}

const handleSongPlay = (song) => {
  playSong(song)
}

const handleAlbumSelect = (album) => {
  console.log('选择专辑:', album.title)
  // 可以导航到专辑详情页面
}

const handleAlbumPlay = (album) => {
  const song = libraryStore.tracks.value.find((track) => track.album === album.title)
  if (song) playSong(song)
}

const handleArtistSelect = (artist) => {
  console.log('选择艺术家:', artist.name)
  // 可以导航到艺术家详情页面
}

const handleArtistPlay = (artist) => {
  const song = libraryStore.tracks.value.find((track) => track.artist === artist.name)
  if (song) playSong(song)
}

const handleArtistFollow = (artist) => {
  artist.isFollowing = !artist.isFollowing
  console.log(artist.isFollowing ? '关注' : '取消关注', artist.name)
}

const handlePlaylistPlay = (playlist) => {
  const song = libraryStore.tracks.value.find((track) => track.album === playlist.name)
  if (song) playSong(song)
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
  const songs = libraryStore.tracks.value
  const index = songs.findIndex((song) => song.id === currentSong.value.id)
  if (index >= 0 && songs.length) playSong(songs[(index - 1 + songs.length) % songs.length])
}

const handleNext = () => {
  const songs = libraryStore.tracks.value
  const index = songs.findIndex((song) => song.id === currentSong.value.id)
  if (index >= 0 && songs.length) playSong(songs[(index + 1) % songs.length])
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

  const targetSeconds = pendingSeekSeconds
  const channelId = activeChannelId.value
  pendingSeekSeconds = null
  seekCommitRequested = false
  seekInFlight = true
  try {
    await bassCall('bass_channel_seek', {
      channelId,
      seconds: targetSeconds
    })
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
  console.log('循环模式')
}

const handleMenu = () => {
  console.log('菜单')
}

const handleQueue = () => {
  console.log('播放队列')
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
      return { albums: albumsData }
    case 'artists':
      return { artists: artistsData }
    case 'effects':
      return {}
    default:
      return {}
  }
}

// 组件挂载时初始化主题
onMounted(async () => {
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
  if (snapshotTimer) clearInterval(snapshotTimer)
  if (seekTimer !== null) window.clearTimeout(seekTimer)
  unbindScrollSources()
  libraryStore.dispose()
})
</script>

<template>
  <div class="music-player">
    <TitleBar :current-song="currentSong" :is-playing="isPlaying" :current-time="currentTime"
      :current-time-ms="currentTimeMs" :total-time="totalTime" :progress="progress" :lyrics="lyricsPayload"
      :lyrics-loading="lyricsLoading" :is-fullscreen="showFullscreenPlayer" :is-scrolled="isTitlebarScrolled"
      @toggle-play="handleTogglePlay" @previous="handlePrevious" @next="handleNext"
      @progress-change="handleProgressChange" @repeat="handleRepeat" @queue="handleQueue"
      @progress-commit="handleProgressCommit" @expand-player="handleExpandPlayer" />

    <!-- 侧边栏 -->
    <Sidebar :sidebar-items="sidebarItems" :current-page="currentPage" :search-query="searchQuery" :is-dark="isDarkTheme"
      @search-update="handleSearchUpdate" @nav-item-click="handleNavItemClick" @add-tag="handleAddTag"
      @add-playlist="handleAddPlaylist" @add-plugin="handleAddPlugin" />

    <!-- 主内容区 -->
    <div class="main-content-wrapper">
      <MotionTransition variant="page" mode="out-in" @after-enter="handlePageEntered">
        <KeepAlive :max="5">
          <component :is="currentPageComponent" :key="currentPage" v-bind="getPageProps()"
            @song-select="handleSongSelect" @song-play="handleSongPlay" @album-select="handleAlbumSelect"
            @album-play="handleAlbumPlay" @artist-select="handleArtistSelect" @artist-play="handleArtistPlay"
            @artist-follow="handleArtistFollow" @playlist-play="handlePlaylistPlay" @navigate="handleNavigate"
            @header-control-click="handleHeaderControlClick" @effects-change="handleEffectsChange" />
        </KeepAlive>
      </MotionTransition>
    </div>

    <!-- 右侧详情面板 -->
    <!-- <DetailPanel :current-song="currentSong" /> -->

    <!-- 全屏播放器层；顶部迷你播放器由 TitleBar 承载 -->
    <PlayerSurface :current-song="currentSong" :is-playing="isPlaying" :current-time="currentTime"
      :current-time-ms="currentTimeMs" :total-time="totalTime" :progress="progress" :lyrics="lyricsPayload"
      :lyrics-loading="lyricsLoading" :is-fullscreen="showFullscreenPlayer" :channel-id="activeChannelId"
      :background-mode="fullscreenBackgroundMode" @close="handleCloseFullscreenPlayer" @toggle-play="handleTogglePlay"
      @previous="handlePrevious" @next="handleNext" @progress-change="handleProgressChange"
      @progress-commit="handleProgressCommit"
      @volume-change="(volume) => console.log('音量变化:', volume)" @shuffle="() => console.log('随机播放')"
      @repeat="handleRepeat" @add-to-playlist="() => console.log('添加到播放列表')" @queue="handleQueue"
      @background-mode-change="handleBackgroundModeChange" />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
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
  background: color-mix(in srgb, rgba(var(--background-color)), black 40%);
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
