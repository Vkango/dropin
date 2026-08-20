<template>
    <div class="player-controls">
        <div class="player-layout">
            <div class="left-zone">
                <div class="now-playing" @mousedown.stop>
                    <MotionButton class="mini-cover-container" :while-hover="{ scale: 1.05 }"
                        :while-press="{ scale: 0.96 }" :transition="microTransition" aria-label="打开全屏播放器"
                        @click.stop="$emit('expand-player')">
                        <MotionTransition variant="miniCover" mode="out-in">
                            <MotionImg :key="currentSong.cover" :src="currentSong.cover" :alt="currentSong.title"
                                :while-hover="{ scale: 1.05 }" :transition="microTransition" class="mini-cover" />
                        </MotionTransition>
                    </MotionButton>

                    <div class="song-info-section">
                        <div class="playing-title">
                            <span class="playing-artist">{{ currentSong.title }}</span>

                        </div>
                        <MotionTransition variant="miniLyric" mode="out-in">
                            <div :key="miniLyricText" class="playing-meta" :title="miniLyricText">
                                <!-- <span class="playing-time">{{ currentTime }} / {{ totalTime }}</span> -->
                                {{ miniLyricText }}
                            </div>
                        </MotionTransition>

                    </div>
                </div>

                <div class="drag-region drag-region-left" data-tauri-drag-region aria-label="拖动窗口"></div>
            </div>

            <div class="controls" aria-label="播放控制" @mousedown.stop>
                <div class="transport-buttons">
                    <MotionButton class="control-button secondary-control" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" aria-label="播放列表"
                        @click.stop="$emit('queue')">
                        <ListMusic :size="18" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="buttonHover" :while-press="buttonPress"
                        :transition="microTransition" aria-label="上一首" @click.stop="$emit('previous')">
                        <SkipBack :size="18" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button play-pause" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" :aria-label="isPlaying ? '暂停' : '播放'"
                        @click.stop="$emit('toggle-play')">
                        <Pause v-if="isPlaying" :size="18" :stroke-width="1.8" />
                        <Play v-else :size="18" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="buttonHover" :while-press="buttonPress"
                        :transition="microTransition" aria-label="下一首" @click.stop="$emit('next')">
                        <SkipForward :size="18" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button secondary-control" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" aria-label="循环播放"
                        @click.stop="$emit('repeat')">
                        <Repeat2 :size="18" :stroke-width="1.8" />
                    </MotionButton>

                </div>

                <div ref="progressRef" class="top-progress" role="slider" tabindex="0" :aria-valuenow="clampedProgress"
                    aria-valuemin="0" aria-valuemax="100" aria-label="播放进度"
                    :class="{ 'is-dragging': isProgressDragging }" @mousedown.stop
                    @pointerdown.stop.prevent="handleProgressPointerDown" @pointermove.stop="handleProgressPointerMove"
                    @pointerup.stop="handleProgressPointerUp" @pointercancel.stop="handleProgressPointerUp"
                    @lostpointercapture="handleProgressPointerUp"
                    @keydown.left.prevent="emitProgressCommit(clampedProgress - 5)"
                    @keydown.right.prevent="emitProgressCommit(clampedProgress + 5)">
                    <div class="top-progress-track">
                        <div class="top-progress-fill" :style="{ width: `${clampedProgress}%` }"></div>
                        <span class="top-progress-thumb" :style="{ left: `${clampedProgress}%` }"></span>
                    </div>
                </div>
            </div>

            <div class="drag-region drag-region-right" data-tauri-drag-region aria-label="拖动窗口"></div>
        </div>
    </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { ListMusic, Pause, Play, Repeat2, SkipBack, SkipForward } from '@lucide/vue'
import { motion, useReducedMotion } from 'motion-v'
import MotionTransition from './MotionTransition.vue'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
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
    }
})

const emit = defineEmits([
    'previous',
    'toggle-play',
    'next',
    'repeat',
    'queue',
    'progress-change',
    'progress-commit',
    'expand-player'
])

const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const MotionButton = motion.button
const MotionImg = motion.img
const progressRef = ref(null)
const isProgressDragging = ref(false)

const syncedLyrics = computed(() => props.lyrics?.lines || [])
const plainLyrics = computed(() => props.lyrics?.plainLines || [])
const activeLyric = computed(() => syncedLyrics.value.find((line) =>
    props.currentTimeMs >= line.startTimeMs && props.currentTimeMs < line.endTimeMs
))
const miniLyricText = computed(() => {
    if (props.lyricsLoading) return '正在读取歌词...'
    if (activeLyric.value?.text) return activeLyric.value.text
    if (syncedLyrics.value.length) return '...'
    if (plainLyrics.value.length) return plainLyrics.value[0]
    return '暂无歌词'
})

const clampedProgress = computed(() => Math.max(0, Math.min(100, Number(props.progress) || 0)))
const buttonHover = {
    scale: 1.08,
    color: 'rgb(var(--primary-color))',
    backgroundColor: 'rgba(var(--primary-color), 0.14)'
}
const buttonPress = { scale: 0.92 }

const emitProgress = (percent) => {
    emit('progress-change', Math.max(0, Math.min(100, Number(percent) || 0)))
}

const emitProgressCommit = (percent) => {
    const boundedPercent = Math.max(0, Math.min(100, Number(percent) || 0))
    emit('progress-change', boundedPercent)
    emit('progress-commit', boundedPercent)
}

const emitProgressFromPointer = (event) => {
    const element = progressRef.value
    if (!element) return null

    const rect = element.getBoundingClientRect()
    if (!rect.width) return null
    const percent = Math.max(0, Math.min(100, (event.clientX - rect.left) / rect.width * 100))
    emitProgress(percent)
    return percent
}

const handleProgressPointerDown = (event) => {
    if (event.pointerType === 'mouse' && event.button !== 0) return
    isProgressDragging.value = true
    progressRef.value?.setPointerCapture?.(event.pointerId)
    emitProgressFromPointer(event)
}

const handleProgressPointerMove = (event) => {
    if (isProgressDragging.value) emitProgressFromPointer(event)
}

const handleProgressPointerUp = (event) => {
    if (!isProgressDragging.value) return
    const percent = emitProgressFromPointer(event)
    emit('progress-commit', percent)
    isProgressDragging.value = false
    if (progressRef.value?.hasPointerCapture?.(event.pointerId)) {
        progressRef.value.releasePointerCapture(event.pointerId)
    }
}
</script>

<style scoped>
.player-controls {
    position: relative;
    width: 100%;
    height: 60px;
    min-width: 0;
    overflow: hidden;
    color: rgb(var(--text-color));
    font-family: MiSans, 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.top-progress {
    position: absolute;
    z-index: 2;
    bottom: 5px;
    width: 100%;
    height: 8px;
    cursor: grab;
    outline: none;
    touch-action: none;
    user-select: none;
    border-radius: 10px;
}

.top-progress.is-dragging {
    cursor: grabbing;
}

.top-progress-track {
    position: relative;
    top: 2px;
    width: 100%;
    height: 4px;
    border-radius: 10px;
    background-color: rgba(var(--text-color), 0.05);
}

.top-progress-fill {
    height: 100%;
    border-radius: inherit;
    background: rgb(var(--primary-color), 0.2);
}

.top-progress-thumb {
    position: absolute;
    top: 50%;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: rgb(var(--primary-color));
    box-shadow: 0 1px 5px rgba(0, 0, 0, 0.35);
    opacity: 0;
    pointer-events: none;
    transform: translate(-50%, -50%) scale(0.7);
    transition: opacity 120ms ease, transform 120ms ease;
}

.top-progress:hover .top-progress-thumb,
.top-progress:focus-visible .top-progress-thumb,
.top-progress.is-dragging .top-progress-thumb {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
}

.player-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: center;
    width: 100%;
    height: 60px;
    min-width: 0;
    padding: 0 18px;
}

.left-zone {
    display: flex;
    align-items: center;
    min-width: 0;
    grid-column: 1;
}

.now-playing {
    display: flex;
    align-items: center;
    flex: 0 1 auto;
    min-width: 0;
    gap: 20px;
}

.drag-region {
    height: 60px;
    min-width: 18px;
    background: transparent;
}

.drag-region-left {
    flex: 1 1 auto;
}

.drag-region-right {
    grid-column: 3;
    width: 100%;
}

.mini-cover-container {
    position: relative;
    flex: 0 0 36px;
    width: 36px;
    height: 36px;
    padding: 0;
    overflow: hidden;
    border: 0;
    border-radius: 6px;
    color: inherit;
    background: transparent;
    cursor: pointer;
    perspective: 600px;
    transform-style: preserve-3d;
}

.mini-cover {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: 6px;
    object-fit: cover;
    backface-visibility: hidden;
    transform-origin: center;
    will-change: transform, opacity, filter;
}

.song-info-section {
    min-width: 0;
    overflow: hidden;
    display: flex;
    gap: 10px;
    align-items: center;
}

.playing-title,
.playing-artist,
.playing-time {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.playing-title {
    /* margin-bottom: 4px; */
    font-size: 14px;
    font-weight: 700;
    /* line-height: 1.2; */
}

.playing-meta {
    display: flex;
    min-width: 0;
    /* gap: 8px; */
    color: rgba(var(--text-color), 0.58);
    font-size: 13px;
    /* line-height: 1.2; */
}

.playing-artist {
    min-width: 0;
}

.playing-time {
    flex: 0 0 auto;
}

.controls {
    position: relative;
    display: flex;
    flex-direction: column;
    grid-column: 2;
    align-items: stretch;
    justify-content: center;
    height: 60px;
    justify-self: center;
}

.transport-buttons {
    display: flex;
    flex: 0 0 60px;
    align-items: center;
    justify-content: center;
    gap: 9px;
    padding: 0 20px;
    margin: 0 10px;
    margin-bottom: 8px;
}

.control-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 7px;
    border: none;
    border-radius: 50%;
    color: rgb(var(--text-color));
    background: transparent;
    cursor: pointer;
}

.play-pause {
    width: 36px;
    height: 36px;
    color: rgb(var(--text-color));
    background: rgba(var(--surface-color), 0.16);
}

@media (max-width: 720px) {
    .player-layout {
        grid-template-columns: auto minmax(0, 1fr);
        gap: 8px;
        padding-right: 8px;
        padding-left: 8px;
    }

    .left-zone {
        grid-column: 1;
    }

    .song-info-section,
    .secondary-control,
    .drag-region-left {
        display: none;
    }

    .controls {
        grid-column: 2;
        justify-self: start;
        width: 100%;
    }

    .transport-buttons {
        gap: 3px;
    }

    .drag-region-right {
        display: none;
    }

    .mini-cover-container {
        flex-basis: 32px;
        width: 32px;
        height: 32px;
    }
}
</style>
