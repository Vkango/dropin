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
                    <h2 class="library-title">{{ t('artists.title') }}</h2>
                    <div class="description">{{ t('artists.bannerDescription') }}
                    </div>
                </div>
            </div>
        </template>
        <div ref="pageRef" class="artists-page">
            <div class="artists-main">
                <div class="page-header">
                    <div class="filter-options">
                        <Combobox v-model="sortBy" class="filter-combo" readable :label="t('artists.sortBy')"
                            :options="sortOptions">
                            <template #icon>
                                <ListFilter size="13" :stroke-width="1.8" />
                            </template>
                        </Combobox>
                        <input v-model="searchQuery" type="text" :placeholder="t('artists.searchPlaceholder')"
                            class="search-input" />
                    </div>
                    <div class="view-controls">
                        <MotionButton class="view-btn" :class="{ active: viewMode === 'grid' }"
                            :while-press="{ scale: 0.92 }" :transition="microTransition" @click="setViewMode('grid')"
                            :aria-label="t('artists.title')">
                            <LayoutGrid size="16" :stroke-width="1.8" />
                        </MotionButton>
                        <MotionButton class="view-btn" :class="{ active: viewMode === 'list' }"
                            :while-press="{ scale: 0.92 }" :transition="microTransition" @click="setViewMode('list')"
                            :aria-label="t('artists.title')">
                            <List size="16" :stroke-width="1.8" />
                        </MotionButton>
                    </div>
                </div>

                <!-- 艺术家网格/列表 -->
                <MotionTransition variant="page" mode="out-in">
                    <div v-if="viewMode === 'grid' && filteredArtists.length" key="grid" class="artists-grid-view">
                        <div v-for="group in visibleGroups" :key="group.key" class="artist-group">
                            <GroupLabel v-if="group.initial" :label="group.initial"
                                @click="handleGroupLabelClick(group.initial)" />
                            <div class="artists-grid">
                                <MotionDiv v-for="artist in group.items" :key="artist.id" class="artist-card"
                                    initial="rest" while-hover="hover" :variants="cardVariants">
                                    <div class="artist-avatar">
                                        <MotionTransition variant="cover" mode="out-in">
                                            <MotionImg :key="artist.avatar || artist.cover"
                                                :src="artist.avatar || artist.cover" :alt="artist.name"
                                                :variants="imageVariants" />
                                        </MotionTransition>
                                        <MotionDiv class="artist-overlay" :variants="overlayVariants">
                                            <MotionButton class="play-btn" :while-hover="{ scale: 1.1 }"
                                                :while-press="{ scale: 0.94 }" :transition="microTransition"
                                                @click.stop="$emit('artist-play', artist)">
                                                <Icon src="/assets/play.svg" size="lg" />
                                            </MotionButton>
                                        </MotionDiv>
                                    </div>
                                    <div class="artist-info">
                                        <h3 class="artist-name">{{ artist.name }}</h3>
                                        <p class="artist-meta">{{ t('artists.albumsCount', { count: artist.albumCount })
                                            }} • {{
                                                t('artists.songsCount', { count: artist.songCount }) }}</p>
                                        <div class="artist-genres">
                                            <span v-for="genre in artist.genres?.slice(0, 2)" :key="genre"
                                                class="genre-tag">
                                                {{ genre }}
                                            </span>
                                        </div>
                                    </div>
                                </MotionDiv>
                            </div>
                        </div>
                    </div>

                    <div v-else-if="viewMode === 'list' && filteredArtists.length" key="list" class="artists-list">
                        <div class="list-header">
                            <div class="header-avatar"></div>
                            <div class="header-name">{{ t('artists.headerName') }}</div>
                            <div class="header-albums">{{ t('artists.headerAlbums') }}</div>
                            <div class="header-songs">{{ t('artists.headerSongs') }}</div>
                            <div class="header-genres">{{ t('artists.headerGenres') }}</div>
                            <div class="header-actions"></div>
                        </div>
                        <template v-for="group in visibleGroups" :key="group.key">
                            <GroupLabel v-if="group.initial" :label="group.initial"
                                @click="handleGroupLabelClick(group.initial)" />
                            <MotionDiv v-for="artist in group.items" :key="artist.id" class="artist-row" initial="rest"
                                while-hover="hover" :variants="rowVariants">
                                <div class="row-avatar">
                                    <img :src="artist.avatar || artist.cover" :alt="artist.name" />
                                </div>
                                <div class="row-name">
                                    <div class="name">{{ artist.name }}</div>
                                    <div class="followers">{{ t('artists.followers', {
                                        count:
                                        formatNumber(artist.followers) }) }}</div>
                                </div>
                                <div class="row-albums">{{ artist.albumCount }}</div>
                                <div class="row-songs">{{ artist.songCount }}</div>
                                <div class="row-genres">
                                    <span v-for="genre in artist.genres?.slice(0, 3)" :key="genre" class="genre-pill">
                                        {{ genre }}
                                    </span>
                                </div>
                                <div class="row-actions">
                                    <MotionButton class="action-btn"
                                        :while-hover="{ scale: 1.08, backgroundColor: 'rgba(var(--primary-color), 0.1)', color: 'rgba(var(--primary-color), 0.3)' }"
                                        :transition="microTransition" @click.stop="$emit('artist-play', artist)">
                                        <Icon src="/assets/play.svg" size="sm" />
                                    </MotionButton>
                                    <MotionButton class="action-btn"
                                        :while-hover="{ scale: 1.08, backgroundColor: 'rgba(var(--primary-color), 0.1)', color: 'rgba(var(--primary-color), 0.3)' }"
                                        :transition="microTransition" @click.stop="toggleFollow(artist)">
                                        <Icon :src="artist.isFollowing ? '/assets/favourite.svg' : '/assets/user.svg'"
                                            size="sm" />
                                    </MotionButton>
                                </div>
                            </MotionDiv>
                        </template>
                    </div>

                    <div v-else key="empty" class="artists-empty">{{ t('artists.empty') }}</div>
                </MotionTransition>
            </div>
            <AlphabetFilter :active-initial="activeInitial" :top-offset="alphabetTopOffset"
                :available-initials="availableInitials" @select="handleAlphabetSelect" />
        </div>
    </PageLayout>
</template>

<script setup>
import { ref, computed, inject } from 'vue'
import Icon from '@/components/ui/Icon.vue'
import Combobox from '@/components/ui/Combobox.vue'
import AlphabetFilter from '@/components/ui/AlphabetFilter.vue'
import { List, LayoutGrid, ListFilter } from '@lucide/vue'
import GroupLabel from '@/components/library/GroupLabel.vue'
import MotionTransition from '@/components/ui/MotionTransition.vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '@/utils/motion.js'
import { getAvailableInitials, groupByInitial, sortByInitial } from '@/utils/alphabet.js'
import { useAlphabetNavigation } from '@/utils/useAlphabetNavigation.js'
import { useI18n } from '@/i18n/index.js'

const { t } = useI18n()
const currentSong = inject('currentSong')
const props = defineProps({
    artists: {
        type: Array,
        default: () => []
    }
})

const emit = defineEmits(['artist-select', 'artist-play', 'artist-follow'])

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
const searchQuery = ref('')
const pageRef = ref(null)
const sortOptions = computed(() => [
    { value: 'name', label: t('artists.sortName') },
    { value: 'albums', label: t('artists.sortAlbums') },
    { value: 'recent', label: t('artists.sortRecent') }
])

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

const artistsAfterFilters = computed(() => {
    let artists = [...props.artists]

    // 搜索筛选
    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase()
        artists = artists.filter(artist =>
            artist.name.toLowerCase().includes(query) ||
            artist.genres?.some(genre => genre.toLowerCase().includes(query))
        )
    }

    return artists
})

const availableInitials = computed(() => getAvailableInitials(artistsAfterFilters.value, (artist) => artist.name))

const filteredArtists = computed(() => {
    let artists = [...artistsAfterFilters.value]

    if (sortBy.value === 'name') {
        return sortByInitial(artists, (artist) => artist.name)
    }

    // 排序
    artists.sort((a, b) => {
        switch (sortBy.value) {
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

const groupedArtists = computed(() => groupByInitial(filteredArtists.value, (artist) => artist.name))
const visibleGroups = computed(() => sortBy.value === 'name'
    ? groupedArtists.value.map((group) => ({ ...group, key: group.initial }))
    : [{ key: 'all', initial: '', items: filteredArtists.value }])

const { activeInitial, alphabetTopOffset, handleAlphabetSelect, handleGroupLabelClick } = useAlphabetNavigation(
    pageRef,
    availableInitials
)
</script>

<style scoped>
.artists-page {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    width: 100%;
}

.artists-main {
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

.filter-options {
    display: flex;
    gap: 30px;
    align-items: center;
}

.artists-grid-view {
    width: 100%;
}

.artist-group {
    width: 100%;
}

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

.filter-combo {
    flex: 0 0 auto;
}

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
    text-align: center;
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
}

/* 空状态 */
.artists-empty {
    padding: 60px 0;
    text-align: center;
    color: rgba(var(--text-color), 0.5);
    font-size: 14px;
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
