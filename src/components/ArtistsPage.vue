<template>
    <div class="artists-page">
        <!-- 页面标题 -->
        <div class="music-banner" @click="showAlbumDetail">
            <div class="image-container">
                <Transition name="banner-image">
                    <img :key="currentSong.cover" class="background-image" :src="currentSong.cover"
                        referrerpolicy="no-referrer">
                </Transition>
            </div>
            <div class="banner-content">
                <div class="title">DROPIN MUSIC PLAYER</div>
                <h2 class="library-title">艺术家</h2>
                <div class="description">这东西一次能吃一袋我丢
                </div>
            </div>
        </div>
        <div class="page-header">
            <h1 class="page-title"></h1>
            <div class="view-controls">
                <button class="view-btn" :class="{ active: viewMode === 'grid' }" @click="setViewMode('grid')">
                    <Icon src="/assets/list.svg" size="sm" />
                </button>
                <button class="view-btn" :class="{ active: viewMode === 'list' }" @click="setViewMode('list')">
                    <Icon src="/assets/folder.svg" size="sm" />
                </button>
            </div>
        </div>

        <!-- 筛选和排序 -->
        <div class="filter-section">
            <div class="filter-options">
                <select v-model="sortBy" class="sort-select">
                    <option value="name">按名称排序</option>
                    <option value="albums">按专辑数排序</option>
                    <option value="recent">最近播放</option>
                </select>
                <input v-model="searchQuery" type="text" placeholder="搜索艺术家..." class="search-input" />
            </div>
        </div>

        <!-- 艺术家网格/列表 -->
        <Transition name="view" mode="out-in">
            <div v-if="viewMode === 'grid'" key="grid" class="artists-grid">
                <div v-for="artist in filteredArtists" :key="artist.id" class="artist-card"
                    @click="$emit('artist-select', artist)">
                    <div class="artist-avatar">
                        <Transition name="avatar-image" mode="out-in">
                            <img :key="artist.avatar || artist.cover" :src="artist.avatar || artist.cover"
                                :alt="artist.name" />
                        </Transition>
                        <div class="artist-overlay">
                            <button class="play-btn" @click.stop="$emit('artist-play', artist)">
                                <Icon src="/assets/play.svg" size="lg" />
                            </button>
                        </div>
                    </div>
                    <div class="artist-info">
                        <h3 class="artist-name">{{ artist.name }}</h3>
                        <p class="artist-meta">{{ artist.albumCount }} 张专辑 • {{ artist.songCount }} 首歌</p>
                        <div class="artist-genres">
                            <span v-for="genre in artist.genres?.slice(0, 2)" :key="genre" class="genre-tag">
                                {{ genre }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <div v-else key="list" class="artists-list">
                <div class="list-header">
                    <div class="header-avatar"></div>
                    <div class="header-name">艺术家</div>
                    <div class="header-albums">专辑</div>
                    <div class="header-songs">歌曲</div>
                    <div class="header-genres">流派</div>
                    <div class="header-actions"></div>
                </div>
                <div v-for="artist in filteredArtists" :key="artist.id" class="artist-row"
                    @click="$emit('artist-select', artist)">
                    <div class="row-avatar">
                        <img :src="artist.avatar || artist.cover" :alt="artist.name" />
                    </div>
                    <div class="row-name">
                        <div class="name">{{ artist.name }}</div>
                        <div class="followers">{{ formatNumber(artist.followers) }} 关注者</div>
                    </div>
                    <div class="row-albums">{{ artist.albumCount }}</div>
                    <div class="row-songs">{{ artist.songCount }}</div>
                    <div class="row-genres">
                        <span v-for="genre in artist.genres?.slice(0, 3)" :key="genre" class="genre-pill">
                            {{ genre }}
                        </span>
                    </div>
                    <div class="row-actions">
                        <button class="action-btn" @click.stop="$emit('artist-play', artist)">
                            <Icon src="/assets/play.svg" size="sm" />
                        </button>
                        <button class="action-btn" @click.stop="toggleFollow(artist)">
                            <Icon :src="artist.isFollowing ? '/assets/favourite.svg' : '/assets/user.svg'" size="sm" />
                        </button>
                    </div>
                </div>
            </div>
        </Transition>
    </div>
</template>

<script setup>
import { ref, computed, inject } from 'vue'
import Icon from './Icon.vue'
const currentSong = inject('currentSong')
const props = defineProps({
    artists: {
        type: Array,
        default: () => []
    }
})

const emit = defineEmits(['artist-select', 'artist-play', 'artist-follow'])

const viewMode = ref('grid')
const sortBy = ref('name')
const searchQuery = ref('')

const setViewMode = (mode) => {
    viewMode.value = mode
}

const toggleFollow = (artist) => {
    emit('artist-follow', artist)
}

const formatNumber = (num) => {
    if (num >= 1000000) {
        return (num / 1000000).toFixed(1) + 'M'
    }
    if (num >= 1000) {
        return (num / 1000).toFixed(1) + 'K'
    }
    return num?.toString() || '0'
}

const filteredArtists = computed(() => {
    let artists = [...props.artists]

    // 搜索筛选
    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase()
        artists = artists.filter(artist =>
            artist.name.toLowerCase().includes(query) ||
            artist.genres?.some(genre => genre.toLowerCase().includes(query))
        )
    }

    // 排序
    artists.sort((a, b) => {
        switch (sortBy.value) {
            case 'name':
                return a.name.localeCompare(b.name)
            case 'albums':
                return (b.albumCount || 0) - (a.albumCount || 0)
            case 'recent':
                return new Date(b.lastPlayed) - new Date(a.lastPlayed)
            default:
                return 0
        }
    })

    return artists
})
</script>

<style scoped>
.artists-page {
    padding: 20px 50px;
    height: 100%;
    overflow-y: auto;
}

/* 页面标题 */
.page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
}

.page-title {
    font-size: 48px;
    font-weight: 700;
    color: rgb(var(--text-color));
}

.view-controls {
    display: flex;
    gap: 8px;
}

.view-btn {
    background: rgba(var(--surface-color), 0.1);
    border: 1px solid rgba(var(--outline-color), 0.2);
    border-radius: 8px;
    padding: 8px 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    color: rgba(var(--text-color), 0.7);
}

.view-btn:hover {
    background: rgba(var(--primary-color), 0.1);
    color: rgba(var(--primary-color), 0.3);
}

.view-btn.active {
    background: rgba(var(--primary-color), 0.3);
    color: white;
    border-color: rgba(var(--primary-color), 0.3);
}

/* 筛选区域 */
.filter-section {
    margin-bottom: 30px;
}

.filter-options {
    display: flex;
    gap: 16px;
    align-items: center;
}

.sort-select,
.search-input {
    background: rgba(var(--surface-color), 0.1);
    border: 1px solid rgba(var(--outline-color), 0.2);
    border-radius: 8px;
    padding: 8px 12px;
    color: rgb(var(--text-color));
    font-size: 14px;
}

.search-input {
    width: 240px;
}

.sort-select:focus,
.search-input:focus {
    outline: none;
    border-color: rgba(var(--primary-color), 0.3);
}

.search-input::placeholder {
    color: rgba(var(--text-color), 0.5);
}

/* 网格视图 */
.artists-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 24px;
}

.artist-card {
    cursor: pointer;
    transition: transform 0.2s ease;
    text-align: center;
}

.artist-card:hover {
    transform: translateY(-4px);
}

.artist-avatar {
    position: relative;
    width: 180px;
    height: 180px;
    border-radius: 50%;
    overflow: hidden;
    margin: 0 auto 16px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.artist-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
}

.artist-card:hover .artist-avatar img {
    transform: scale(1.05);
}

.artist-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.2s ease;
}

.artist-card:hover .artist-overlay {
    opacity: 1;
}

.play-btn {
    background: rgba(var(--primary-color), 0.3);
    border: none;
    border-radius: 50%;
    width: 56px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
    color: white;
}

.play-btn:hover {
    transform: scale(1.1);
}

.artist-info {
    text-align: center;
}

.artist-name {
    font-size: 18px;
    font-weight: 600;
    color: rgb(var(--text-color));
    margin-bottom: 6px;
}

.artist-meta {
    font-size: 14px;
    color: rgba(var(--text-color), 0.7);
    margin-bottom: 8px;
}

.artist-genres {
    display: flex;
    gap: 6px;
    justify-content: center;
    flex-wrap: wrap;
}

.genre-tag {
    background: rgba(var(--primary-color), 0.1);
    color: rgba(var(--primary-color), 0.3);
    padding: 4px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
}

/* 列表视图 */
.artists-list {
    width: 100%;
}

.list-header {
    display: grid;
    grid-template-columns: 60px 1fr 80px 80px 200px 100px;
    gap: 16px;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(var(--outline-color), 0.2);
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 500;
    color: rgba(var(--text-color), 0.7);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.artist-row {
    display: grid;
    grid-template-columns: 60px 1fr 80px 80px 200px 100px;
    gap: 16px;
    align-items: center;
    padding: 12px 16px;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s ease;
}

.artist-row:hover {
    background: rgba(var(--primary-color), 0.05);
}

.row-avatar img {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    object-fit: cover;
}

.row-name .name {
    font-weight: 500;
    color: rgb(var(--text-color));
    margin-bottom: 2px;
}

.row-name .followers {
    font-size: 12px;
    color: rgba(var(--text-color), 0.5);
}

.row-albums,
.row-songs {
    color: rgba(var(--text-color), 0.7);
    font-size: 14px;
    text-align: center;
}

.row-genres {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
}

.genre-pill {
    background: rgba(var(--outline-color), 0.1);
    color: rgba(var(--text-color), 0.7);
    padding: 2px 6px;
    border-radius: 8px;
    font-size: 11px;
}

.row-actions {
    display: flex;
    gap: 8px;
    justify-content: center;
}

.action-btn {
    background: transparent;
    border: none;
    color: rgba(var(--text-color), 0.5);
    cursor: pointer;
    padding: 6px;
    border-radius: 50%;
    transition: all 0.2s ease;
}

.action-btn:hover {
    background: rgba(var(--primary-color), 0.1);
    color: rgba(var(--primary-color), 0.3);
}

/* 视图切换动画 */
.view-enter-active,
.view-leave-active {
    transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.view-enter-from {
    opacity: 0;
    transform: translateY(20px);
}

.view-leave-to {
    opacity: 0;
    transform: translateY(-20px);
}

/* 头像图片过渡动画 */
.avatar-image-enter-active,
.avatar-image-leave-active {
    transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.avatar-image-enter-from {
    opacity: 0;
    transform: scale(0.9);
}

.avatar-image-leave-to {
    opacity: 0;
    transform: scale(1.1);
}

/* 滚动条样式 */
.artists-page::-webkit-scrollbar {
    width: 4px;
}

.artists-page::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
}

.artists-page::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.artists-page::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
}

/* 响应式设计 */
@media (max-width: 768px) {
    .artists-page {
        padding: 20px 24px;
    }

    .page-title {
        font-size: 36px;
    }

    .artists-grid {
        grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
        gap: 16px;
    }

    .artist-avatar {
        width: 120px;
        height: 120px;
    }

    .search-input {
        width: 180px;
    }

    .artists-list {
        overflow-x: auto;
    }

    .list-header,
    .artist-row {
        grid-template-columns: 60px 1fr 60px 60px 120px 80px;
        gap: 12px;
    }
}
</style>
