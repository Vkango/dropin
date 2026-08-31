<template>
    <div class="artist-drawer">
        <div class="hero">
            <img v-if="heroImage" class="hero-bg" :src="heroImage" :alt="artist.name" referrerpolicy="no-referrer" />
            <div class="hero-fade"></div>
            <div class="hero-bottom">
                <div class="hero-meta">
                    <h3 class="hero-name">{{ artist.name }}</h3>
                    <!-- <div class="hero-counts">
                        <span>{{ t('artistCard.albumsCount', { count: artist.albums?.length ?? 0 }) }}</span>
                        <span class="dot">·</span>
                        <span>{{ t('artistCard.tracksCount', { count: artist.tracks?.length ?? 0 }) }}</span>
                    </div> -->
                    <div v-if="artist.genres?.length" class="hero-genres">
                        <span v-for="genre in artist.genres.slice(0, 3)" :key="genre" class="genre-tag">
                            {{ genre }}
                        </span>
                    </div>
                </div>
                <div class="artist-tabs" role="tablist">
                    <MotionButton class="tab" :class="{ active: activeTab === 'albums' }" role="tab"
                        :aria-selected="activeTab === 'albums'" :while-press="{ scale: 0.97 }"
                        :transition="microTransition" @click="setTab('albums')">
                        {{ t('artistDrawer.albumsTab') }}
                        <span class="tab-count">{{ artist.albums?.length ?? 0 }}</span>
                    </MotionButton>
                    <MotionButton class="tab" :class="{ active: activeTab === 'tracks' }" role="tab"
                        :aria-selected="activeTab === 'tracks'" :while-press="{ scale: 0.97 }"
                        :transition="microTransition" @click="setTab('tracks')">
                        {{ t('artistDrawer.tracksTab') }}
                        <span class="tab-count">{{ artist.tracks?.length ?? 0 }}</span>
                    </MotionButton>
                </div>
            </div>
            <MotionButton ref="backButtonRef" class="hero-back" :while-hover="{ scale: 1.08 }"
                :while-press="{ scale: 0.94 }" :transition="microTransition" :aria-label="t('drawer.close')"
                @click="$emit('close')">
                <ArrowLeft :size="18" :stroke-width="1.8" />
            </MotionButton>
            <MotionButton class="hero-play" :while-hover="{ scale: 1.08 }" :while-press="{ scale: 0.94 }"
                :transition="microTransition" :aria-label="t('artistCard.playAll')" @click="$emit('play-all')">
                <PlayIcon />
            </MotionButton>
        </div>



        <div class="tab-panels">
            <MotionTransition v-if="activeTab === 'albums'" variant="modal">
                <div class="tab-panel" role="tabpanel">
                    <div v-if="artist.albums?.length" class="album-list">
                        <MotionDiv v-for="album in artist.albums" :key="album.id" class="album-row"
                            :while-hover="{ backgroundColor: 'rgba(var(--primary-color), 0.08)' }"
                            :transition="microTransition" @click="$emit('album-select', album)">
                            <div class="album-row-cover">
                                <img :src="album.cover" :alt="album.title" />
                            </div>
                            <div class="album-row-info">
                                <div class="album-row-title">{{ album.title }}</div>
                                <div class="album-row-meta"><span v-if="album.year">{{ album.year }} · </span>{{
                                    t('albumCard.tracksCount', { count: album.trackCount ?? 0 }) }}</div>
                            </div>
                            <ChevronRight :size="16" :stroke-width="1.8" class="album-row-arrow" />
                        </MotionDiv>
                    </div>
                    <div v-else class="section-empty">{{ t('artistCard.emptyAlbums') }}</div>
                </div>
            </MotionTransition>

            <MotionTransition v-else variant="modal">
                <div class="tab-panel" role="tabpanel">
                    <div v-if="artist.tracks?.length" class="track-list">
                        <MotionDiv v-for="(track, index) in artist.tracks" :key="track.id" class="track-row"
                            :while-hover="{ backgroundColor: 'rgba(var(--primary-color), 0.08)' }"
                            :transition="microTransition" @click="$emit('track-play', track)">
                            <div class="track-number">{{ index + 1 }}</div>
                            <div class="track-album-cover">
                                <img :src="track.cover" :alt="track.album" />
                            </div>
                            <div class="track-info">
                                <div class="track-title">{{ track.title }}</div>
                                <div class="track-album">{{ track.album }}</div>
                            </div>
                            <div class="track-duration">{{ track.duration }}</div>
                        </MotionDiv>
                    </div>
                    <div v-else class="section-empty">{{ t('artistCard.emptyTracks') }}</div>
                </div>
            </MotionTransition>
        </div>
    </div>
</template>

<script setup>
import { defineProps, defineEmits, ref } from 'vue'
import { motion, useReducedMotion } from 'motion-v'
import { computed } from 'vue'
import { ArrowLeft, ChevronRight } from '@lucide/vue'
import Icon from '@/components/ui/Icon.vue'
import MotionTransition from '@/components/ui/MotionTransition.vue'
import { INSTANT_MOTION, MICRO_SPRING } from '@/utils/motion.js'
import { useI18n } from '@/i18n/index.js'
import { PlayIcon } from '@lucide/vue'

const { t } = useI18n()
const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

const props = defineProps({
    artist: {
        type: Object,
        default: () => ({
            name: '',
            cover: '',
            genres: [],
            albums: [],
            tracks: []
        })
    }
})

defineEmits(['close', 'play-all', 'album-select', 'track-play'])

const backButtonRef = ref(null)
const heroImage = computed(() => props.artist.cover || '')

const activeTab = ref('albums')

const setTab = (tab) => {
    activeTab.value = tab
}
</script>

<style scoped>
.artist-drawer {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.hero {
    position: relative;
    flex: 0 0 auto;
    height: 240px;
    overflow: hidden;
}

.hero-bg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
}

/* 向下渐变遮罩：顶部清晰，向下透明度递减融入面板背景 */
.hero-fade {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom,
            rgba(0, 0, 0, 0.08) 0%,
            rgba(0, 0, 0, 0.22) 45%,
            color-mix(in srgb, rgb(var(--surface-color)) 96%, rgb(var(--global-color)) 4%) 100%);
}

.hero-bottom {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    padding: 16px 72px 14px 16px;
}

.hero-meta {
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.hero-name {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    color: #fff;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.45);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.hero-counts {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.85);
    text-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
}

.hero-counts .dot {
    opacity: 0.6;
}

.hero-genres {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
}

.genre-tag {
    background: rgba(255, 255, 255, 0.18);
    backdrop-filter: blur(8px);
    color: rgba(255, 255, 255, 0.95);
    padding: 3px 8px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 500;
}

.hero-back {
    position: absolute;
    top: 14px;
    left: 14px;
    z-index: 2;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: none;
    background: rgba(0, 0, 0, 0.35);
    backdrop-filter: blur(8px);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
}

.hero-play {
    position: absolute;
    right: 18px;
    bottom: 16px;
    z-index: 2;
    width: 52px;
    height: 52px;
    border-radius: 50%;
    border: none;
    background: rgb(var(--primary-color), 0.5);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}

.artist-tabs {
    display: flex;
    gap: 4px;
    padding: 4px;
    margin-top: 10px;
    border-radius: 10px;
    background: rgba(var(--surface-color), 0.25);
    width: fit-content;
}

.tab {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 12px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: rgba(var(--text-color), 0.6);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: color 160ms ease, background-color 160ms ease;
}

.tab:hover {
    color: rgb(var(--text-color));
}

.tab.active {
    background: rgba(var(--primary-color), 0.14);
    color: rgb(var(--text-color));
}

.tab-count {
    font-size: 11px;
    font-weight: 500;
    color: rgba(var(--text-color), 0.5);
    background: rgba(var(--outline-color), 0.15);
    padding: 1px 6px;
    border-radius: 8px;
}

.tab.active .tab-count {
    color: rgb(var(--primary-color));
    background: rgba(var(--primary-color), 0.14);
}

.tab-panels {
    flex: 1 1 auto;
    min-height: 0;
    overflow: hidden;
    padding: 12px 16px 16px;
}

.tab-panel {
    height: 100%;
    overflow-y: auto;
    overscroll-behavior: contain;
}

.section-empty {
    padding: 14px 12px;
    border-radius: 8px;
    background: rgba(var(--surface-color), 0.1);
    color: rgba(var(--text-color), 0.5);
    font-size: 13px;
    text-align: center;
}

.album-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.album-row {
    display: grid;
    grid-template-columns: 48px 1fr auto;
    gap: 12px;
    align-items: center;
    padding: 6px 8px;
    border-radius: 10px;
    cursor: pointer;
}

.album-row-cover {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.album-row-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.album-row-info {
    min-width: 0;
}

.album-row-title {
    color: rgb(var(--text-color));
    font-weight: 500;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.album-row-meta {
    color: rgba(var(--text-color), 0.5);
    font-size: 12px;
}

.album-row-arrow {
    color: rgba(var(--text-color), 0.4);
}

.track-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.track-row {
    display: grid;
    grid-template-columns: 24px 40px 1fr auto;
    gap: 12px;
    align-items: center;
    padding: 8px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
}

.track-album-cover {
    width: 40px;
    height: 40px;
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.track-album-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.track-number {
    color: rgba(var(--text-color), 0.5);
    font-size: 12px;
    text-align: center;
}

.track-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.track-title {
    color: rgb(var(--text-color));
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.track-album {
    color: rgba(var(--text-color), 0.6);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.track-duration {
    color: rgba(var(--text-color), 0.5);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
}
</style>
