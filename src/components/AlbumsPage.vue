<template>
    <PageLayout>
        <template #header>
        <!-- 页面标题 -->
        <div class="music-banner" @click="showAlbumDetail">
            <div class="image-container">
                <MotionTransition variant="banner">
                    <img :key="currentSong.cover" class="background-image" :src="currentSong.cover"
                        referrerpolicy="no-referrer">
                </MotionTransition>
            </div>
            <div class="banner-content">
                <div class="title">DROPIN MUSIC PLAYER</div>
                <h2 class="library-title">专辑</h2>
                <div class="description">这东西一次能吃一袋我丢
                </div>
            </div>
        </div>
        </template>
        <div class="albums-page">
        <div class="page-header">
            <h1 class="page-title"></h1>
            <div class="view-controls">
                <MotionButton class="view-btn" :class="{ active: viewMode === 'grid' }"
                    :while-hover="{ y: -1, backgroundColor: 'rgba(var(--primary-color), 0.1)', color: 'rgba(var(--primary-color), 0.3)' }"
                    :while-press="{ scale: 0.96 }" :transition="microTransition"
                    @click="setViewMode('grid')">
                    <Icon src="/assets/list.svg" size="sm" />
                </MotionButton>
                <MotionButton class="view-btn" :class="{ active: viewMode === 'list' }"
                    :while-hover="{ y: -1, backgroundColor: 'rgba(var(--primary-color), 0.1)', color: 'rgba(var(--primary-color), 0.3)' }"
                    :while-press="{ scale: 0.96 }" :transition="microTransition"
                    @click="setViewMode('list')">
                    <Icon src="/assets/folder.svg" size="sm" />
                </MotionButton>
            </div>
        </div>

        <!-- 筛选和排序 -->
        <div class="filter-section">
            <div class="filter-options">
                <Combobox v-model="sortBy" class="sort-select" :options="sortOptions" />
                <Combobox v-model="filterGenre" class="genre-select" :options="genreOptions" />
            </div>
        </div>

        <!-- 专辑网格/列表 -->
        <MotionTransition variant="page" mode="out-in">
            <div v-if="viewMode === 'grid'" key="grid" class="albums-grid">
                <MotionDiv v-for="album in filteredAlbums" :key="album.id" class="album-card" initial="rest"
                    while-hover="hover" :variants="cardVariants"
                    @click="$emit('album-select', album)">
                    <div class="album-cover">
                        <MotionTransition variant="cover" mode="out-in">
                            <MotionImg :key="album.cover" :src="album.cover" :alt="album.title"
                                :variants="imageVariants" />
                        </MotionTransition>
                        <MotionDiv class="album-overlay" :variants="overlayVariants">
                            <MotionButton class="play-btn" :while-hover="{ scale: 1.1 }" :while-press="{ scale: 0.94 }"
                                :transition="microTransition" @click.stop="$emit('album-play', album)">
                                <Icon src="/assets/play.svg" size="lg" />
                            </MotionButton>
                        </MotionDiv>
                    </div>
                    <div class="album-info">
                        <h3 class="album-title">{{ album.title }}</h3>
                        <p class="album-artist">{{ album.artist }}</p>
                        <p class="album-meta">{{ album.year }} • {{ album.trackCount }} 首歌</p>
                    </div>
                </MotionDiv>
            </div>

            <div v-else key="list" class="albums-list">
                <div class="list-header">
                    <div class="header-cover"></div>
                    <div class="header-title">标题</div>
                    <div class="header-artist">艺术家</div>
                    <div class="header-year">年份</div>
                    <div class="header-tracks">歌曲数</div>
                    <div class="header-duration">时长</div>
                </div>
                <MotionDiv v-for="album in filteredAlbums" :key="album.id" class="album-row" initial="rest"
                    while-hover="hover" :variants="rowVariants"
                    @click="$emit('album-select', album)">
                    <div class="row-cover">
                        <img :src="album.cover" :alt="album.title" />
                    </div>
                    <div class="row-title">{{ album.title }}</div>
                    <div class="row-artist">{{ album.artist }}</div>
                    <div class="row-year">{{ album.year }}</div>
                    <div class="row-tracks">{{ album.trackCount }}</div>
                    <div class="row-duration">{{ album.duration }}</div>
                    <div class="row-actions">
                        <MotionButton class="action-btn" :while-hover="{ scale: 1.08, backgroundColor: 'rgba(var(--primary-color), 0.1)', color: 'rgba(var(--primary-color), 0.3)' }"
                            :transition="microTransition" @click.stop="$emit('album-play', album)">
                            <Icon src="/assets/play.svg" size="sm" />
                        </MotionButton>
                    </div>
                </MotionDiv>
            </div>
        </MotionTransition>
        </div>
    </PageLayout>
</template>

<script setup>
import { ref, computed, inject } from 'vue'
import Icon from './Icon.vue'
import Combobox from './Combobox.vue'
import MotionTransition from './MotionTransition.vue'
import PageLayout from './PageLayout.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
    albums: {
        type: Array,
        default: () => []
    }
})
const currentSong = inject('currentSong')
const emit = defineEmits(['album-select', 'album-play'])

const MotionDiv = motion.div
const MotionImg = motion.img
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const cardVariants = { rest: { y: 0 }, hover: { y: -4 } }
const imageVariants = { rest: { scale: 1 }, hover: { scale: 1.05 } }
const overlayVariants = { rest: { opacity: 0 }, hover: { opacity: 1 } }
const rowVariants = {
    rest: { backgroundColor: 'rgba(0, 0, 0, 0)' },
    hover: { backgroundColor: 'rgba(var(--primary-color), 0.05)' }
}

const viewMode = ref('grid')
const sortBy = ref('name')
const filterGenre = ref('')
const sortOptions = [
    { value: 'name', label: '按名称排序' },
    { value: 'artist', label: '按艺术家排序' },
    { value: 'year', label: '按年份排序' },
    { value: 'recent', label: '最近添加' }
]
const genreOptions = [
    { value: '', label: '所有流派' },
    { value: '电子', label: '电子' },
    { value: '流行', label: '流行' },
    { value: '摇滚', label: '摇滚' },
    { value: '古典', label: '古典' }
]

const setViewMode = (mode) => {
    viewMode.value = mode
}

const filteredAlbums = computed(() => {
    let albums = [...props.albums]

    // 筛选流派
    if (filterGenre.value) {
        albums = albums.filter(album =>
            album.genres && album.genres.includes(filterGenre.value)
        )
    }

    // 排序
    albums.sort((a, b) => {
        switch (sortBy.value) {
            case 'name':
                return a.title.localeCompare(b.title)
            case 'artist':
                return a.artist.localeCompare(b.artist)
            case 'year':
                return parseInt(b.year) - parseInt(a.year)
            case 'recent':
                return new Date(b.addedDate) - new Date(a.addedDate)
            default:
                return 0
        }
    })

    return albums
})
</script>

<style scoped>
.albums-page {
    width: 100%;
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
    color: rgba(var(--text-color), 0.7);
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
}

.sort-select,
.genre-select {
    min-width: 160px;
}

/* 网格视图 */
.albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 24px;
}

.album-card {
    cursor: pointer;
}

.album-cover {
    position: relative;
    aspect-ratio: 1;
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 16px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.album-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.album-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
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
    color: white;
}

.album-info {
    text-align: left;
}

.album-title {
    font-size: 16px;
    font-weight: 600;
    color: rgb(var(--text-color));
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.album-artist {
    font-size: 14px;
    color: rgba(var(--text-color), 0.7);
    margin-bottom: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.album-meta {
    font-size: 12px;
    color: rgba(var(--text-color), 0.5);
}

/* 列表视图 */
.albums-list {
    width: 100%;
}

.list-header {
    display: grid;
    grid-template-columns: 60px 1fr 200px 80px 80px 100px 60px;
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

.album-row {
    display: grid;
    grid-template-columns: 60px 1fr 200px 80px 80px 100px 60px;
    gap: 16px;
    align-items: center;
    padding: 12px 16px;
    border-radius: 8px;
    cursor: pointer;
}

.row-cover img {
    width: 44px;
    height: 44px;
    border-radius: 6px;
    object-fit: cover;
}

.row-title {
    font-weight: 500;
    color: rgb(var(--text-color));
}

.row-artist,
.row-year,
.row-tracks,
.row-duration {
    color: rgba(var(--text-color), 0.7);
    font-size: 14px;
}

.action-btn {
    background: transparent;
    border: none;
    color: rgba(var(--text-color), 0.5);
    cursor: pointer;
    padding: 8px;
    border-radius: 50%;
}

/* 滚动条样式 */
.albums-page::-webkit-scrollbar {
    width: 4px;
}

.albums-page::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
}

.albums-page::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.albums-page::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
}

/* 响应式设计 */
@media (max-width: 768px) {
    .albums-page {
        padding: 20px 24px;
    }

    .page-title {
        font-size: 36px;
    }

    .albums-grid {
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 16px;
    }

    .albums-list {
        overflow-x: auto;
    }

    .list-header,
    .album-row {
        grid-template-columns: 60px 1fr 150px 60px 60px 80px 50px;
        gap: 12px;
    }
}
</style>
