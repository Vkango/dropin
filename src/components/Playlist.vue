<template>
    <section class="playlist" :aria-label="title">
        <header v-if="showHeader" class="playlist-header">
            <h3 class="playlist-title">{{ title }}</h3>
            <span class="playlist-count">{{ songs.length }} 首</span>
        </header>

        <div v-if="songs.length" class="playlist-items" role="list">
            <MotionButton v-for="song in songs" :key="song.id" class="playlist-item"
                :class="{ 'is-current': isCurrentSong(song) }" type="button" role="listitem"
                :aria-current="isCurrentSong(song) ? 'true' : undefined" :aria-label="songLabel(song)"
                :while-hover="itemHover" :while-press="itemPress" :transition="microTransition"
                @click="emit('song-select', song)">
                <div class="playlist-item-leading">
                    <img class="playlist-cover" :src="song.cover || '/assets/cover.jpg'" :alt="song.title" />
                    <span v-if="isCurrentSong(song)" class="playlist-playing-indicator" aria-hidden="true">
                        <Pause v-if="isPlaying" :size="14" :stroke-width="2" />
                        <Play v-else :size="14" :stroke-width="2" />
                    </span>
                </div>

                <span class="playlist-item-info">
                    <span class="playlist-item-name">{{ song.title }}</span>
                    <span class="playlist-item-artist">{{ song.artist || '未知艺术家' }}</span>
                </span>

                <span class="playlist-item-duration">{{ song.duration || '--:--' }}</span>
            </MotionButton>
        </div>

        <div v-else class="playlist-empty">播放列表为空</div>
    </section>
</template>

<script setup>
import { computed } from 'vue'
import { motion, useReducedMotion } from 'motion-v'
import { Pause, Play } from '@lucide/vue'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
    songs: {
        type: Array,
        default: () => []
    },
    currentSong: {
        type: Object,
        default: null
    },
    isPlaying: {
        type: Boolean,
        default: false
    },
    title: {
        type: String,
        default: '正在播放'
    },
    showHeader: {
        type: Boolean,
        default: true
    }
})

const emit = defineEmits(['song-select'])
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const itemHover = { x: -2, backgroundColor: 'rgba(var(--primary-color), 0.1)' }
const itemPress = { scale: 0.985 }

const isCurrentSong = (song) => {
    if (!props.currentSong) return false
    return song.id === props.currentSong.id
        || (song.title === props.currentSong.title && song.artist === props.currentSong.artist)
}

const songLabel = (song) => `${song.title} - ${song.artist || '未知艺术家'}${isCurrentSong(song) ? '，正在播放' : ''}`
</script>

<style scoped>
.playlist {
    min-height: 100%;
    padding: 8px 10px 20px;
}

.playlist-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 10px 12px;
}

.playlist-title {
    font-size: 13px;
    font-weight: 600;
    color: rgba(var(--text-color), 0.62);
}

.playlist-count {
    color: rgba(var(--text-color), 0.42);
    font-size: 12px;
}

.playlist-items {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.playlist-item {
    display: flex;
    align-items: center;
    width: 100%;
    min-width: 0;
    gap: 11px;
    padding: 8px 10px;
    border: 0;
    border-radius: 11px;
    color: rgb(var(--text-color));
    background: transparent;
    text-align: left;
    cursor: pointer;
}

.playlist-item.is-current {
    color: rgb(var(--primary-color));
    background: rgba(var(--primary-color), 0.13);
}

.playlist-item-leading {
    position: relative;
    flex: 0 0 auto;
}

.playlist-cover {
    display: block;
    width: 46px;
    height: 46px;
    border-radius: 7px;
    object-fit: cover;
}

.playlist-playing-indicator {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    background: rgba(0, 0, 0, 0.48);
    border-radius: 7px;
}

.playlist-item-info {
    display: flex;
    flex: 1 1 auto;
    min-width: 0;
    flex-direction: column;
    gap: 4px;
}

.playlist-item-name,
.playlist-item-artist {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.playlist-item-name {
    font-size: 13px;
    font-weight: 560;
}

.playlist-item-artist {
    color: rgba(var(--text-color), 0.58);
    font-size: 12px;
}

.playlist-item-duration {
    flex: 0 0 auto;
    color: rgba(var(--text-color), 0.5);
    font-size: 12px;
}

.playlist-empty {
    padding: 50px 20px;
    color: rgba(var(--text-color), 0.5);
    font-size: 13px;
    text-align: center;
}
</style>
