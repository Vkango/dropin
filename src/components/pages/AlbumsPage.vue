<template>
    <PageLayout>
        <template #header>
            <!-- 页面标题 -->
            <div class="music-banner">
                <div class="image-container">
                    <MotionTransition variant="banner">
                        <img :key="currentSong.cover" class="background-image" :src="currentSong.cover"
                            referrerpolicy="no-referrer">
                    </MotionTransition>
                </div>
                <div class="banner-content">
                    <div class="title">{{ t('app.name') }}</div>
                    <h2 class="library-title">{{ t('albums.title') }}</h2>
                    <div class="description">{{ t('albums.bannerDescription') }}
                    </div>
                </div>
            </div>
        </template>
        <div ref="pageRef" class="albums-page">
            <div class="albums-main">
                <div class="page-header">
                    <div class="filter-options">
                        <Combobox v-model="sortBy" class="filter-combo" readable :label="t('albums.sortBy')"
                            :options="sortOptions">
                            <template #icon>
                                <ListFilter size="13" :stroke-width="1.8" />
                            </template>
                        </Combobox>
                        <Combobox v-model="filterGenre" class="filter-combo" readable :label="t('albums.genreFilter')"
                            :options="genreOptions">
                            <template #icon>
                                <Tags size="13" :stroke-width="1.8" />
                            </template>
                        </Combobox>
                    </div>
                    <div class="view-controls">
                        <MotionButton class="view-btn" :class="{ active: viewMode === 'grid' }"
                            :while-press="{ scale: 0.92 }" :transition="microTransition" @click="setViewMode('grid')"
                            :aria-label="t('albums.title')">
                            <LayoutGrid size="16" :stroke-width="1.8" />
                        </MotionButton>
                        <MotionButton class="view-btn" :class="{ active: viewMode === 'list' }"
                            :while-press="{ scale: 0.92 }" :transition="microTransition" @click="setViewMode('list')"
                            :aria-label="t('albums.title')">
                            <List size="16" :stroke-width="1.8" />
                        </MotionButton>
                    </div>
                </div>

                <!-- 专辑网格/列表 -->
                <MotionTransition variant="page" mode="out-in">
                    <div v-if="viewMode === 'grid' && filteredAlbums.length" key="grid" class="albums-grid-view">
                        <div v-for="group in visibleGroups" :key="group.key" class="album-group">
                            <GroupLabel v-if="group.initial" :label="group.initial"
                                @click="handleGroupLabelClick(group.initial)" />
                            <div class="albums-grid">
                                <MotionDiv v-for="album in group.items" :key="album.id" class="album-card"
                                    initial="rest" while-hover="hover" :variants="cardVariants"
                                    @click="showAlbumDetail(album)">
                                    <div class="album-cover">
                                        <MotionTransition variant="cover" mode="out-in">
                                            <MotionImg :key="album.cover" :src="album.cover" :alt="album.title"
                                                :variants="imageVariants" />
                                        </MotionTransition>
                                        <MotionDiv class="album-overlay" :variants="overlayVariants">
                                            <MotionButton class="play-btn" :while-hover="{ scale: 1.1 }"
                                                :while-press="{ scale: 0.94 }" :transition="microTransition"
                                                @click.stop="$emit('album-play', album)">
                                                <Icon src="/assets/play.svg" size="lg" />
                                            </MotionButton>
                                        </MotionDiv>
                                    </div>
                                    <div class="album-info">
                                        <h3 class="album-title">{{ album.title }}</h3>
                                        <p class="album-artist">{{ album.artist }}</p>
                                        <p class="album-meta"><span v-if="album.year">{{ album.year }} • </span>{{
                                            t('albums.tracksCount', {
                                                count:
                                                    album.trackCount
                                            }) }}</p>
                                    </div>
                                </MotionDiv>
                            </div>
                        </div>
                    </div>

                    <div v-else-if="viewMode === 'list' && filteredAlbums.length" key="list" class="albums-list">
                        <div class="list-header">
                            <div class="header-cover"></div>
                            <div class="header-title">{{ t('albums.headerTitle') }}</div>
                            <div class="header-artist">{{ t('albums.headerArtist') }}</div>
                            <div class="header-year">{{ t('albums.headerYear') }}</div>
                            <div class="header-tracks">{{ t('albums.headerTracks') }}</div>
                            <div class="header-duration">{{ t('albums.headerDuration') }}</div>
                        </div>
                        <template v-for="group in visibleGroups" :key="group.key">
                            <GroupLabel v-if="group.initial" :label="group.initial"
                                @click="handleGroupLabelClick(group.initial)" />
                            <MotionDiv v-for="album in group.items" :key="album.id" class="album-row" initial="rest"
                                while-hover="hover" :variants="rowVariants" @click="showAlbumDetail(album)">
                                <div class="row-cover">
                                    <img :src="album.cover" :alt="album.title" />
                                </div>
                                <div class="row-title">{{ album.title }}</div>
                                <div class="row-artist">{{ album.artist }}</div>
                                <div class="row-year">{{ album.year }}</div>
                                <div class="row-tracks">{{ album.trackCount }}</div>
                                <div class="row-duration">{{ album.duration }}</div>
                                <div class="row-actions">
                                    <MotionButton class="action-btn"
                                        :while-hover="{ scale: 1.08, backgroundColor: 'rgba(var(--primary-color), 0.1)', color: 'rgba(var(--primary-color), 0.3)' }"
                                        :transition="microTransition" @click.stop="$emit('album-play', album)">
                                        <Icon src="/assets/play.svg" size="sm" />
                                    </MotionButton>
                                </div>
                            </MotionDiv>
                        </template>
                    </div>

                    <div v-else key="empty" class="albums-empty">{{ t('albums.empty') }}</div>
                </MotionTransition>
            </div>
            <AlphabetFilter :active-initial="activeInitial" :top-offset="alphabetTopOffset"
                :available-initials="availableInitials" @select="handleAlphabetSelect" />
        </div>
        <AlbumDetailCard :visible="albumDetailVisible" :album="currentAlbumDetail" :z-index="120"
            @close="hideAlbumDetail" @play-all="handlePlayAll" @track-select="handleTrackSelect"
            @track-play="handleTrackPlay" @artist-jump="handleArtistJump" />
    </PageLayout>
</template>

<script setup>
import { ref, computed, inject } from 'vue'
import Icon from '@/components/ui/Icon.vue'
import Combobox from '@/components/ui/Combobox.vue'
import AlphabetFilter from '@/components/ui/AlphabetFilter.vue'
import { List, LayoutGrid, ListFilter, Tags } from '@lucide/vue'
import GroupLabel from '@/components/library/GroupLabel.vue'
import AlbumDetailCard from '@/components/library/AlbumDetailCard.vue'
import MotionTransition from '@/components/ui/MotionTransition.vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '@/utils/motion.js'
import { getAvailableInitials, groupByInitial, sortByInitial } from '@/utils/alphabet.js'
import { useAlphabetNavigation } from '@/utils/useAlphabetNavigation.js'
import { useI18n } from '@/i18n/index.js'

const { t } = useI18n()

const props = defineProps({
    albums: {
        type: Array,
        default: () => []
    },
    songs: {
        type: Array,
        default: () => []
    }
})
const currentSong = inject('currentSong')
const emit = defineEmits(['album-select', 'album-play', 'song-select', 'song-play', 'artist-jump'])

const MotionDiv = motion.div
const MotionImg = motion.img
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const cardVariants = { rest: { y: 0 }, hover: { y: -4 } }
const imageVariants = { rest: { scale: 1 }, hover: { scale: 1.05 } }
const overlayVariants = { rest: { opacity: 0 }, hover: { opacity: 1 } }
const rowVariants = {
    rest: {},
    hover: { backgroundColor: 'rgba(var(--primary-color), 0.05)' }
}

const viewMode = ref('grid')
const sortBy = ref('name')
const filterGenre = ref('')
const pageRef = ref(null)
const sortOptions = computed(() => [
    { value: 'name', label: t('albums.sortName') },
    { value: 'artist', label: t('albums.sortArtist') },
    { value: 'year', label: t('albums.sortYear') },
    { value: 'recent', label: t('albums.sortRecent') }
])
const genreOptions = computed(() => [
    { value: '', label: t('albums.genreAll') },
    { value: '电子', label: t('albums.genreElectronic') },
    { value: '流行', label: t('albums.genrePop') },
    { value: '摇滚', label: t('albums.genreRock') },
    { value: '古典', label: t('albums.genreClassical') }
])

const setViewMode = (mode) => {
    viewMode.value = mode
}

const albumsAfterFilters = computed(() => {
    let albums = [...props.albums]

    if (filterGenre.value) {
        albums = albums.filter(album =>
            album.genres && album.genres.includes(filterGenre.value)
        )
    }

    return albums
})

const availableInitials = computed(() => getAvailableInitials(albumsAfterFilters.value, (album) => album.title))

const filteredAlbums = computed(() => {
    let albums = [...albumsAfterFilters.value]

    if (sortBy.value === 'name') {
        return sortByInitial(albums, (album) => album.title)
    }

    albums.sort((a, b) => {
        switch (sortBy.value) {
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

const groupedAlbums = computed(() => groupByInitial(filteredAlbums.value, (album) => album.title))
const visibleGroups = computed(() => sortBy.value === 'name'
    ? groupedAlbums.value.map((group) => ({ ...group, key: group.initial }))
    : [{ key: 'all', initial: '', items: filteredAlbums.value }])

const { activeInitial, alphabetTopOffset, handleAlphabetSelect, handleGroupLabelClick } = useAlphabetNavigation(
    pageRef,
    availableInitials
)

const albumDetailVisible = ref(false)
const currentAlbumDetail = ref(null)
const currentAlbum = ref(null)

const showAlbumDetail = (album) => {
    currentAlbum.value = album
    const tracks = props.songs.filter((song) => song.album === album.title)
    currentAlbumDetail.value = {
        ...album,
        coverUrl: album.cover,
        type: t('albumCard.typeMusicAlbum'),
        tracks: tracks.map((song, index) => ({
            id: song.id,
            number: index + 1,
            title: song.title,
            artist: song.artist,
            duration: song.duration,
            url: song.url
        }))
    }
    albumDetailVisible.value = true
}

const hideAlbumDetail = () => {
    albumDetailVisible.value = false
}

const handlePlayAll = () => {
    if (currentAlbum.value) emit('album-play', currentAlbum.value)
}

const resolveTrack = (track) => props.songs.find((song) => song.id === track.id || song.title === track.title)

const handleTrackSelect = (track) => {
    const song = resolveTrack(track)
    if (song) emit('song-select', song)
}

const handleTrackPlay = (track) => {
    const song = resolveTrack(track)
    if (song) emit('song-play', song)
}

const handleArtistJump = (artistName) => {
    if (artistName) emit('artist-jump', { name: artistName })
}
</script>

<style scoped>
.albums-page {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    width: 100%;
}

.albums-main {
    min-width: 0;
    flex: 1;
}

/* 页面标题 */
.page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    margin-bottom: 30px;
}

.page-title {
    font-size: 48px;
    font-weight: 700;
    color: rgb(var(--text-color));
}

.view-controls {
    display: flex;
    align-items: center;
    gap: 6px;
}

.filter-options {
    display: flex;
    gap: 30px;
    align-items: center;
}

.view-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: rgba(var(--text-color), 0.45);
    cursor: pointer;
    transition: color 160ms ease;
}

.view-btn:hover {
    color: rgba(var(--text-color), 0.85);
}

.view-btn.active {
    color: rgb(var(--primary-color));
}

.albums-grid-view {
    width: 100%;
}

.album-group {
    width: 100%;
}

.filter-combo {
    flex: 0 0 auto;
}

/* 网格视图 */
.albums-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 40px;
    margin: 20px 0 10px 16px;
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

/* 空状态 */
.albums-empty {
    padding: 60px 0;
    text-align: center;
    color: rgba(var(--text-color), 0.5);
    font-size: 14px;
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
