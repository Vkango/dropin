<template>
    <div class="player-controls">
        <div class="player-layout">
            <div class="left-zone">
                <div class="now-playing">
                    <MotionButton class="mini-cover-container" :while-hover="{ scale: 1.05 }"
                        :while-press="{ scale: 0.96 }" :transition="microTransition"
                        :aria-label="t('player.openFullscreen')"
                        @mousedown.stop @click.stop="$emit('expand-player')">
                        <MotionTransition variant="miniCover" mode="out-in">
                            <MotionImg :key="currentSong.cover" :src="currentSong.cover" :alt="currentSong.title"
                                :while-hover="{ scale: 1.05 }" :transition="microTransition" class="mini-cover" />
                        </MotionTransition>
                    </MotionButton>

                    <div class="song-info-section" data-tauri-drag-region role="button" tabindex="0"
                        :aria-label="t('player.openFullscreen')" @click.stop="$emit('expand-player')"
                        @keydown.enter.stop="$emit('expand-player')"
                        @keydown.space.prevent.stop="$emit('expand-player')">
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

                <div class="drag-region drag-region-left" data-tauri-drag-region :aria-label="t('player.dragWindow')"></div>
            </div>

            <div class="controls" :aria-label="t('player.playbackMode')" @mousedown.stop>
                <div class="transport-buttons">
                    <div id="mini-playback-anchor" ref="miniPopoverAnchorRef" class="mini-popover-anchor">
                        <MotionButton class="control-button secondary-control" :while-hover="buttonHover"
                            :while-press="buttonPress" :transition="microTransition"
                            :aria-label="t('player.volumeAndOrder')"
                            :aria-expanded="isPlaybackModePopoverOpen"
                            @click.stop="isPlaybackModePopoverOpen = !isPlaybackModePopoverOpen">
                            <Menu :size="14" :stroke-width="1.8" />
                        </MotionButton>
                        <PlaybackModePopover :open="isPlaybackModePopoverOpen" :mode="props.playbackMode"
                            anchor-id="mini-playback-anchor" placement="below" :list-loop="props.listLoop"
                            :include-volume="true" :volume="props.volume" :muted="props.muted"
                            @update:mode="$emit('playback-mode-change', $event)"
                            @update:list-loop="$emit('list-loop-change', $event)"
                            @update:volume="$emit('volume-change', $event)" @mute-change="$emit('mute-change', $event)"
                            @close="isPlaybackModePopoverOpen = false" />
                    </div>
                    <MotionButton class="control-button" :while-hover="buttonHover" :while-press="buttonPress"
                        :transition="microTransition" :aria-label="t('player.previous')" @click.stop="$emit('previous')">
                        <SkipBack :size="14" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button play-pause" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition"
                        :aria-label="isPlaying ? t('player.pause') : t('player.play')"
                        @click.stop="$emit('toggle-play')">
                        <Pause v-if="isPlaying" :size="15" :stroke-width="1.8" />
                        <Play v-else :size="18" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="buttonHover" :while-press="buttonPress"
                        :transition="microTransition" :aria-label="t('player.next')" @click.stop="$emit('next')">
                        <SkipForward :size="14" :stroke-width="1.8" />
                    </MotionButton>

                    <MotionButton class="control-button secondary-control" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" :aria-label="t('player.queue')"
                        @click.stop="$emit('queue')">
                        <ListMusic :size="14" :stroke-width="1.8" />
                    </MotionButton>
                </div>

                <div ref="progressRef" class="top-progress" role="slider" tabindex="0" :aria-valuenow="clampedProgress"
                    aria-valuemin="0" aria-valuemax="100" :aria-label="t('player.progress')"
                    :class="{ 'is-dragging': isProgressDragging }" @mousedown.stop
                    @pointerdown.stop.prevent="handleProgressPointerDown"
                    @keydown.left.prevent="emitProgressCommit(clampedProgress - 5)"
                    @keydown.right.prevent="emitProgressCommit(clampedProgress + 5)">
                    <div class="top-progress-track">
                        <div class="top-progress-fill" :style="{ width: `${clampedProgress}%` }"></div>
                        <span class="top-progress-thumb" :style="{ left: `${clampedProgress}%` }"></span>
                    </div>
                </div>
            </div>

            <div class="drag-region drag-region-right" data-tauri-drag-region :aria-label="t('player.dragWindow')"></div>
        </div>
    </div>
</template>

<script setup>
import { computed, onBeforeUnmount, ref } from 'vue'
import { ListMusic, Menu, Pause, Play, SkipBack, SkipForward } from '@lucide/vue'
import { motion, useReducedMotion } from 'motion-v'
import MotionTransition from './MotionTransition.vue'
import PlaybackModePopover from './PlaybackModePopover.vue'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

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
    },
    playbackMode: {
        type: String,
        default: 'sequential'
    },
    listLoop: {
        type: Boolean,
        default: false
    },
    volume: {
        type: Number,
        default: 75
    },
    muted: {
        type: Boolean,
        default: false
    }
})

const emit = defineEmits([
    'previous',
    'toggle-play',
    'next',
    'playback-mode-change',
    'list-loop-change',
    'volume-change',
    'mute-change',
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
const activeProgressPointerId = ref(null)
const isPlaybackModePopoverOpen = ref(false)
const miniPopoverAnchorRef = ref(null)

const syncedLyrics = computed(() => props.lyrics?.lines || [])
const plainLyrics = computed(() => props.lyrics?.plainLines || [])
const activeLyric = computed(() => syncedLyrics.value.find((line) =>
    props.currentTimeMs >= line.startTimeMs && props.currentTimeMs < line.endTimeMs
))
const miniLyricText = computed(() => {
    if (props.lyricsLoading) return t('player.loadingLyrics')
    if (activeLyric.value?.text) return activeLyric.value.text
    if (syncedLyrics.value.length) return '...'
    if (plainLyrics.value.length) return plainLyrics.value[0]
    return t('player.noLyrics')
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

const emitProgressFromPointer = (event, element = progressRef.value) => {
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
    activeProgressPointerId.value = event.pointerId
    progressRef.value?.setPointerCapture?.(event.pointerId)
    emitProgressFromPointer(event)
    window.addEventListener('pointermove', handleProgressPointerMove)
    window.addEventListener('pointerup', handleProgressPointerUp)
    window.addEventListener('pointercancel', handleProgressPointerUp)
}

const handleProgressPointerMove = (event) => {
    if (isProgressDragging.value && event.pointerId === activeProgressPointerId.value) {
        emitProgressFromPointer(event)
    }
}

const handleProgressPointerUp = (event) => {
    if (!isProgressDragging.value || event.pointerId !== activeProgressPointerId.value) return
    const percent = emitProgressFromPointer(event)
    emit('progress-commit', percent ?? 0)
    isProgressDragging.value = false
    activeProgressPointerId.value = null
    window.removeEventListener('pointermove', handleProgressPointerMove)
    window.removeEventListener('pointerup', handleProgressPointerUp)
    window.removeEventListener('pointercancel', handleProgressPointerUp)
    if (progressRef.value?.hasPointerCapture?.(event.pointerId)) {
        progressRef.value.releasePointerCapture(event.pointerId)
    }
}

onBeforeUnmount(() => {
    window.removeEventListener('pointermove', handleProgressPointerMove)
    window.removeEventListener('pointerup', handleProgressPointerUp)
    window.removeEventListener('pointercancel', handleProgressPointerUp)
})
</script>

<style scoped>
.player-controls {
    position: relative;
    width: 100%;
    height: 60px;
    min-width: 0;
    overflow: visible;
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
    flex: 1 1 180px;
    min-width: 0;
    min-height: 36px;
    overflow: hidden;
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 0 4px;
    border-radius: 8px;
    cursor: pointer;
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

.mini-popover-anchor {
    position: relative;
    display: inline-flex;
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
