<template>
    <AnimatePresence>
        <MotionDiv v-if="isFullscreen" class="player-backdrop" :initial="{ opacity: 0 }" :animate="{ opacity: 1 }"
            :exit="{ opacity: 0 }" :transition="backdropTransition">
            <MotionDiv class="player-backdrop-shade" :initial="{ opacity: 0, backdropFilter: 'blur(0px)' }"
                :animate="{ opacity: 0.7, backdropFilter: 'blur(14px)' }"
                :exit="{ opacity: 0, backdropFilter: 'blur(0px)' }" :transition="backdropTransition" />
        </MotionDiv>
    </AnimatePresence>

    <AnimatePresence>
        <MotionDiv v-if="isFullscreen" class="player-surface" :initial="fullLayerInitial" :animate="fullLayerOpen"
            :exit="fullLayerClosed" :transition="contentTransition">
            <FullscreenPlayer :is-visible="isFullscreen" :current-song="currentSong" :is-playing="isPlaying"
                :current-time="currentTime" :current-time-ms="currentTimeMs" :total-time="totalTime"
                :progress="progress" :lyrics="lyrics" :lyrics-loading="lyricsLoading" :channel-id="channelId"
                :background-mode="backgroundMode" :queue-songs="queueSongs" :volume="volume" :muted="muted"
                :playback-mode="playbackMode" :list-loop="listLoop" @close="$emit('close')"
                @toggle-play="$emit('toggle-play')" @previous="$emit('previous')" @next="$emit('next')"
                @progress-change="$emit('progress-change', $event)" @progress-commit="$emit('progress-commit', $event)"
                @volume-change="$emit('volume-change', $event)" @mute-change="$emit('mute-change', $event)"
                @playback-mode-change="$emit('playback-mode-change', $event)"
                @list-loop-change="$emit('list-loop-change', $event)" @add-to-playlist="$emit('add-to-playlist')"
                @playlist-song-select="$emit('playlist-song-select', $event)"
                @background-mode-change="$emit('background-mode-change', $event)" />
        </MotionDiv>
    </AnimatePresence>
</template>

<script setup>
import { computed } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import FullscreenPlayer from '@/components/player/FullscreenPlayer.vue'
import { APPLE_SPRING, INSTANT_MOTION, SOFT_SPRING } from '@/utils/motion.js'

defineProps({
    currentSong: {
        type: Object,
        required: true
    },
    isPlaying: {
        type: Boolean,
        default: false
    },
    currentTime: {
        type: String,
        default: '00:00'
    },
    currentTimeMs: {
        type: Number,
        default: 0
    },
    totalTime: {
        type: String,
        default: '00:00'
    },
    progress: {
        type: Number,
        default: 0
    },
    lyrics: {
        type: Object,
        default: null
    },
    lyricsLoading: {
        type: Boolean,
        default: false
    },
    channelId: {
        type: Number,
        default: null
    },
    backgroundMode: {
        type: String,
        default: 'flowing'
    },
    queueSongs: {
        type: Array,
        default: () => []
    },
    volume: {
        type: Number,
        default: 75
    },
    muted: {
        type: Boolean,
        default: false
    },
    playbackMode: {
        type: String,
        default: 'sequential'
    },
    listLoop: {
        type: Boolean,
        default: false
    },
    isFullscreen: {
        type: Boolean,
        default: false
    }
})

defineEmits([
    'close',
    'toggle-play',
    'previous',
    'next',
    'progress-change',
    'progress-commit',
    'volume-change',
    'mute-change',
    'playback-mode-change',
    'list-loop-change',
    'add-to-playlist',
    'playlist-song-select',
    'background-mode-change'
])

const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const backdropTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)
const contentTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)

const fullLayerInitial = {
    opacity: 0,
    scale: 0.02,
    filter: 'blur(14px)'
}
const fullLayerOpen = {
    opacity: 1,
    scale: 1,
    filter: 'blur(0px)'
}
const fullLayerClosed = {
    opacity: 0,
    scale: 0.02,
    filter: 'blur(14px)'
}
</script>

<style scoped>
.player-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    pointer-events: none;
    background: rgba(0, 0, 0, 0.18);
}

.player-backdrop-shade {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    will-change: opacity, backdrop-filter;
}

.player-surface {
    position: fixed;
    inset: 0;
    z-index: 1000;
    overflow: hidden;
    height: 100dvh;
    transform-origin: 0% 0%;
    pointer-events: auto;
    will-change: transform, opacity, filter;
}
</style>
