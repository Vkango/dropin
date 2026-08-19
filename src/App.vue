<script setup>
import { ref, reactive, onMounted, provide, computed } from 'vue'
import Sidebar from './components/Sidebar.vue'
import MusicLibrary from './components/MusicLibrary.vue'
import HomePage from './components/HomePage.vue'
import AlbumsPage from './components/AlbumsPage.vue'
import ArtistsPage from './components/ArtistsPage.vue'
import SoundEffectsPage from './components/SoundEffectsPage.vue'
import PluginsPage from './components/PluginsPage.vue'
import FullscreenPlayer from './components/FullscreenPlayer.vue'
import DetailPanel from './components/DetailPanel.vue'
import PlayerControls from './components/PlayerControls.vue'
import TitleBar from './components/TitleBar.vue'
import { themeManager } from './utils/themeManager.js'

// 当前页面状态
const currentPage = ref('home')
const pageHistory = ref(['home'])
const pageCache = reactive(new Map())

// 全屏播放器状态
const showFullscreenPlayer = ref(false)

// 播放器动画状态
const playerAnimationState = ref('idle') // 'idle', 'expanding', 'collapsing'
const isPlayerTransitioning = ref(false)
const currentAnimationId = ref(0) // 用于跟踪当前动画序列

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
  title: '僕たち、やっと行けるんだね!',
  artist: '菅野祐悟',
  album: 'TVアニメ『はたらく細胞BLACK』Original Soundtrack',
  duration: '02:04',
  cover: '/assets/cover1.jpg'
})

provide('currentSong', currentSong)

// 播放状态
const isPlaying = ref(false)
const currentTime = ref('00:32')
const totalTime = ref('04:19')
const progress = ref(32)

// 搜索查询
const searchQuery = ref('')

// 主题状态
const currentTheme = ref(null)

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

// 页面切换逻辑
const navigateToPage = (pageId) => {
  if (pageId === currentPage.value) return

  // 缓存当前页面状态
  if (currentPage.value) {
    pageCache.set(currentPage.value, {
      scrollPosition: document.querySelector('.main-content')?.scrollTop || 0,
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

  // 恢复页面状态
  setTimeout(() => {
    const cachedState = pageCache.get(pageId)
    if (cachedState) {
      const mainContent = document.querySelector('.main-content')
      if (mainContent) {
        mainContent.scrollTop = cachedState.scrollPosition
      }
    }
  }, 100)
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

const handleHeaderControlClick = (control) => {
  console.log('头部控制点击:', control.label)
}

const handleSongSelect = (song) => {
  currentSong.value = { ...song }
  updateThemeFromSong(song)
}

const handleSongPlay = (song) => {
  currentSong.value = { ...song }
  isPlaying.value = true
  updateThemeFromSong(song)
}

const handleAlbumSelect = (album) => {
  console.log('选择专辑:', album.title)
  // 可以导航到专辑详情页面
}

const handleAlbumPlay = (album) => {
  console.log('播放专辑:', album.title)
  // 播放专辑第一首歌
}

const handleArtistSelect = (artist) => {
  console.log('选择艺术家:', artist.name)
  // 可以导航到艺术家详情页面
}

const handleArtistPlay = (artist) => {
  console.log('播放艺术家:', artist.name)
  // 播放艺术家的热门歌曲
}

const handleArtistFollow = (artist) => {
  artist.isFollowing = !artist.isFollowing
  console.log(artist.isFollowing ? '关注' : '取消关注', artist.name)
}

const handlePlaylistPlay = (playlist) => {
  console.log('播放播放列表:', playlist.name)
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

const handleTogglePlay = () => {
  isPlaying.value = !isPlaying.value
}

const handlePrevious = () => {
  console.log('播放上一首')
}

const handleNext = () => {
  console.log('播放下一首')
}

const handleProgressChange = (percent) => {
  progress.value = percent
  console.log('进度更改:', percent + '%')
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

const handleAdd = () => {
  console.log('添加')
}

const handleExpandPlayer = () => {
  const animationId = ++currentAnimationId.value // 生成新的动画ID

  isPlayerTransitioning.value = true
  playerAnimationState.value = 'expanding'

  // 立即显示全屏播放器（用于开始动画）
  showFullscreenPlayer.value = true

  // 动画完成后重置状态（检查动画ID避免竞态条件）
  setTimeout(() => {
    if (currentAnimationId.value === animationId) {
      playerAnimationState.value = 'idle'
      isPlayerTransitioning.value = false
    }
  }, 800) // 与动画时长一致
}

const handleCloseFullscreenPlayer = () => {
  const animationId = ++currentAnimationId.value // 生成新的动画ID

  isPlayerTransitioning.value = true
  playerAnimationState.value = 'collapsing'

  // 动画完成后隐藏全屏播放器（检查动画ID避免竞态条件）
  setTimeout(() => {
    if (currentAnimationId.value === animationId) {
      showFullscreenPlayer.value = false
      playerAnimationState.value = 'idle'
      isPlayerTransitioning.value = false
    }
  }, 600) // 与动画时长一致
}// 获取当前页面所需的props
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
})
</script>

<template>
  <div class="music-player">
    <!-- 侧边栏 -->
    <Sidebar :sidebar-items="sidebarItems" :search-query="searchQuery" @search-update="handleSearchUpdate"
      @nav-item-click="handleNavItemClick" @add-tag="handleAddTag" @add-playlist="handleAddPlaylist"
      @add-plugin="handleAddPlugin" />

    <!-- 主内容区 -->
    <div class="main-content-wrapper">
      <Transition name="page" mode="out-in">
        <KeepAlive :max="5">
          <component :is="currentPageComponent" :key="currentPage" v-bind="getPageProps()"
            @song-select="handleSongSelect" @song-play="handleSongPlay" @album-select="handleAlbumSelect"
            @album-play="handleAlbumPlay" @artist-select="handleArtistSelect" @artist-play="handleArtistPlay"
            @artist-follow="handleArtistFollow" @playlist-play="handlePlaylistPlay" @navigate="handleNavigate"
            @header-control-click="handleHeaderControlClick" @effects-change="handleEffectsChange"
            class="main-content" />
        </KeepAlive>
      </Transition>
    </div>

    <!-- 右侧详情面板 -->
    <!-- <DetailPanel :current-song="currentSong" /> -->

    <!-- 底部播放控制栏 -->
    <PlayerControls :current-song="currentSong" :is-playing="isPlaying" :current-time="currentTime"
      :total-time="totalTime" :progress="progress" :animation-state="playerAnimationState"
      :is-transitioning="isPlayerTransitioning" @toggle-play="handleTogglePlay" @previous="handlePrevious"
      @next="handleNext" @progress-change="handleProgressChange" @repeat="handleRepeat" @menu="handleMenu"
      @add="handleAdd" @expand-player="handleExpandPlayer" />

    <!-- 全屏播放器 -->
    <FullscreenPlayer :is-visible="showFullscreenPlayer" :current-song="currentSong" :is-playing="isPlaying"
      :current-time="currentTime" :total-time="totalTime" :progress="progress" :animation-state="playerAnimationState"
      :is-transitioning="isPlayerTransitioning" @close="handleCloseFullscreenPlayer" @toggle-play="handleTogglePlay"
      @previous="handlePrevious" @next="handleNext" @progress-change="handleProgressChange"
      @volume-change="(volume) => console.log('音量变化:', volume)" @shuffle="() => console.log('随机播放')"
      @repeat="handleRepeat" @add-to-playlist="() => console.log('添加到播放列表')" @queue="() => console.log('播放队列')" />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --primary-color: 136, 208, 236;
  --primary-hover-color: 0, 86, 179;
  --background-color: 254, 251, 255;
  --surface-color: 254, 251, 255;
  --text-color: 28, 27, 31;
  --secondary-color: 80, 96, 110;
  --outline-color: 121, 116, 126;

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

/* 页面切换动画 */
.page-enter-active,
.page-leave-active {
  transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

.page-enter-to,
.page-leave-from {
  opacity: 1;
  transform: translateX(0);
}

body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif;
  background: #1a1a1a;
  color: #ffffff;
  overflow: hidden;
  transition: all 0.3s ease;
}

/* Banner 样式 */
.music-banner {
  width: calc(100% + 100px);
  height: 300px;
  position: relative;
  margin: -20px -50px;
  margin-bottom: 20px;
}

.image-container {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.image-container img {
  -webkit-mask-image: linear-gradient(rgba(0, 0, 0, 0.1), transparent);
  mask-image: linear-gradient(rgba(0, 0, 0, 0.1), transparent);
  filter: blur(5px);
  transition: transform 0.3s ease;
}

/* Banner 图片过渡动画 */
.banner-image-enter-active {
  transition: all 0.5s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

.banner-image-leave-active {
  transition: all 0.5s cubic-bezier(0.55, 0.085, 0.68, 0.53);
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

.banner-image-enter-from {
  opacity: 0;
  transform: scale(1.1) translateY(30px);
  filter: blur(15px);
}

.banner-image-leave-to {
  opacity: 0;
  transform: scale(0.9) translateY(-30px);
  filter: blur(15px);
}

.banner-image-enter-to,
.banner-image-leave-from {
  opacity: 1;
  transform: scale(1) translateY(0);
  filter: blur(5px);
}

.music-banner .background-image {
  width: 100%;
  height: 300px;
  object-fit: cover;
}

.banner-content {
  position: absolute;
  top: 30px;
  padding: 15px 45px;
}

.banner-content .title {
  font-size: 14px;
  font-weight: bold;
  color: #ffffff;
  opacity: 0.5;
  margin-bottom: 8px;
  text-shadow: 0 2px 10px rgba(0, 0, 0, 0.7);
}

.library-title {
  font-size: 32px;
  font-weight: 700;
  margin: 0 0 8px 0;
}

.banner-content .description {
  margin-bottom: 15px;
  opacity: 0.8;
  color: #ffffff;
  font-size: 14px;
  text-shadow: 0 1px 5px rgba(0, 0, 0, 0.7);
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
  grid-template-areas:
    "sidebar main detail"
    "controls controls controls";
  height: 100vh;
  background: var(--md-sys-color-background);
  color: #ffffff;
  font-family: 'Inter', Arial, sans-serif;
}

.main-content-wrapper {
  grid-area: main;
  position: relative;
  overflow: hidden;
}

.main-content {
  height: 100%;
  overflow-y: auto;
}
</style>
