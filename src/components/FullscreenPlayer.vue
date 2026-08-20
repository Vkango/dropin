<template>
    <div class="fullscreen-player" :style="lyricsStyle" @click.self="$emit('close')">
        <div class="player-background" :class="`background-mode-${backgroundMode}`" aria-hidden="true">
            <FlowingBackground v-if="backgroundMode === 'flowing'" :cover="currentSong.cover" :bands="audioBands" />
            <div v-else class="backdrop-image" :style="{ backgroundImage: `url(${currentSong.cover})` }"></div>
            <div class="backdrop-wash"></div>
            <div class="backdrop-vignette"></div>
        </div>

        <div class="player-container" @click.stop>
            <main class="player-main">
                <section class="visual-column" aria-label="歌曲信息">
                    <div ref="albumStageRef" class="album-stage">
                        <MotionDiv class="album-visual" :style="albumVisualStyle"
                            :animate="{ rotate: isPlaying ? 360 : 0 }"
                            :transition="isPlaying ? loopTransition : instantTransition">
                            <!--<div class="tick-ring" aria-hidden="true">
                                <span v-for="tick in ticks" :key="tick" class="tick"
                                    :style="{ transform: `translate(-50%, -50%) rotate(${tick}deg) translateY(calc(-1 * var(--tick-radius)))` }"></span>
                            </div>-->
                            <div class="disc-shell">
                                <MotionTransition variant="albumCover" mode="out-in">
                                    <div :key="currentSong.cover" class="album-cover-frame">
                                        <img :src="currentSong.cover" :alt="currentSong.title" class="album-cover" />
                                    </div>
                                </MotionTransition>
                            </div>
                        </MotionDiv>
                    </div>

                    <MotionTransition variant="songInfo" mode="out-in">
                        <div :key="currentSong.title" class="song-details">
                            <h1 class="song-title">{{ currentSong.title }}</h1>
                            <h2 class="song-artist">
                                <User2Icon style="scale: 0.7;" />{{ currentSong.artist }}
                            </h2>
                            <p class="song-album">
                                <DiscAlbum style="scale: 0.7;" />{{ currentSong.album }}
                            </p>

                            <div v-if="songTags.length" class="song-tags">
                                <span v-for="tag in songTags" :key="tag.key" class="tag">{{ tag.label }}</span>
                            </div>
                        </div>
                    </MotionTransition>
                </section>

                <section class="lyrics-column" aria-label="歌词">
                    <div ref="lyricsWindowRef" class="lyrics-window">
                        <MotionDiv v-if="lyricRows.length" class="lyrics-track" :animate="{ y: lyricOffset }"
                            :transition="contentTransition">
                            <MotionDiv v-for="(row, index) in lyricRows" :ref="setLyricRowRef(row.key)" :key="row.key"
                                class="lyric-line" :class="{ 'lyric-interlude-row': row.type === 'interlude' }"
                                :aria-label="row.type === 'interlude' && activeLyricRowIndex === index ? '间奏' : undefined"
                                :animate="getLyricState(index)"
                                :transition="row.type === 'interlude' ? instantTransition : contentTransition">
                                <template v-if="row.type === 'interlude'">
                                    <MoreHorizontal v-if="activeLyricRowIndex === index" class="lyric-interlude-icon"
                                        :size="32" :stroke-width="2.5" fill="currentColor" aria-hidden="true" />
                                </template>
                                <template v-else>
                                    <div class="lyric-primary">{{ row.line.text }}</div>
                                    <div v-for="secondary in row.line.secondary" :key="secondary"
                                        class="lyric-secondary">
                                        {{ secondary }}
                                    </div>
                                </template>
                            </MotionDiv>
                        </MotionDiv>
                        <MotionDiv v-else-if="lyricsLoading" class="lyrics-status" :initial="{ opacity: 0 }"
                            :animate="{ opacity: 1 }" :transition="microTransition">
                            正在读取歌词...
                        </MotionDiv>
                        <div v-else-if="plainLyrics.length" class="plain-lyrics">
                            <div v-for="line in plainLyrics" :key="line" class="plain-lyric-line">
                                {{ line }}
                            </div>
                        </div>
                        <MotionDiv v-else class="lyrics-status" :initial="{ opacity: 0 }" :animate="{ opacity: 1 }"
                            :transition="microTransition">
                            暂无同步歌词
                        </MotionDiv>
                    </div>
                </section>
            </main>

            <footer class="player-footer">
                <AnimatePresence mode="wait" :initial="false">
                    <MotionDiv v-if="!isPlaybackOptionsOpen" key="transport-bar" class="footer-view transport-view"
                        :initial="footerViewInitial" :animate="footerViewAnimate" :exit="footerViewExit"
                        :transition="footerTransition">
                        <div class="footer-actions">
                            <div class="footer-side footer-side-left">
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" aria-label="收起播放器"
                                    @click="$emit('close')">
                                    <ChevronDown :size="18" :stroke-width="1.5" />
                                </MotionButton>
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" aria-label="打开播放页选项"
                                    @click="openPlaybackOptions">
                                    <SquarePen :size="18" :stroke-width="1.5" />
                                </MotionButton>
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" aria-label="全屏显示">
                                    <Maximize2 :size="18" :stroke-width="1.5" />
                                </MotionButton>

                            </div>

                            <div class="transport-column">
                                <div class="transport-controls">
                                    <MotionButton class="footer-button volume-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition" aria-label="音量">
                                        <Volume2 :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition" aria-label="上一首"
                                        @click="$emit('previous')">
                                        <SkipBack :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <MotionButton class="play-button" :while-hover="{ scale: 1.06 }"
                                        :while-press="{ scale: 0.94 }" :transition="microTransition"
                                        :aria-label="isPlaying ? '暂停' : '播放'" @click="$emit('toggle-play')">
                                        <Pause v-if="isPlaying" :size="18" :stroke-width="1.8" />
                                        <Play v-else :size="18" :stroke-width="1.8" />
                                    </MotionButton>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition" aria-label="下一首"
                                        @click="$emit('next')">
                                        <SkipForward :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                    <MotionButton class="footer-button" :while-hover="buttonHover"
                                        :while-press="buttonPress" :transition="microTransition" aria-label="随机播放"
                                        @click="$emit('shuffle')">
                                        <Shuffle :size="18" :stroke-width="1.5" />
                                    </MotionButton>
                                </div>

                                <div class="progress-section">
                                    <span class="time-display">{{ currentTime }}</span>
                                    <div class="progress-container" :class="{ 'is-dragging': isProgressDragging }"
                                        role="slider" :aria-valuenow="Math.round(progress)" aria-valuemin="0"
                                        aria-valuemax="100" tabindex="0" @keydown="handleProgressKeydown"
                                        @pointerdown="handleProgressPointerDown"
                                        @pointermove="handleProgressPointerMove" @pointerup="handleProgressPointerUp"
                                        @pointercancel="handleProgressPointerUp">
                                        <div class="progress-track">
                                            <MotionDiv class="progress-fill" :animate="{ width: `${progress}%` }"
                                                :transition="progressTransition"></MotionDiv>
                                            <MotionDiv class="progress-thumb" :animate="{ left: `${progress}%` }"
                                                :transition="progressTransition"></MotionDiv>
                                        </div>
                                    </div>
                                    <span class="time-display">{{ totalTime }}</span>
                                </div>
                            </div>

                            <div class="footer-side footer-side-right">
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" aria-label="播放设置">
                                    <SlidersHorizontal :size="18" :stroke-width="1.5" />
                                </MotionButton>
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" aria-label="歌词面板">
                                    <PanelTop :size="18" :stroke-width="1.5" />
                                </MotionButton>
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" aria-label="播放队列"
                                    @click="$emit('queue')">
                                    <ListMusic :size="18" :stroke-width="1.5" />
                                </MotionButton>
                                <MotionButton class="footer-button" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" aria-label="更多操作">
                                    <MoreHorizontal :size="18" :stroke-width="1.5" />
                                </MotionButton>
                            </div>
                        </div>
                    </MotionDiv>

                    <MotionDiv v-else key="playback-options" class="footer-view playback-options" role="group"
                        aria-label="播放页选项" :initial="footerViewInitial" :animate="footerViewAnimate"
                        :exit="footerViewExit" :transition="footerTransition"
                        @animation-complete="handlePlaybackOptionsAnimationComplete">
                        <MotionDiv class="playback-options-leading" :initial="settingsItemInitial"
                            :animate="settingsItemAnimate" :transition="settingsItemTransition(0)">
                            <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                                :transition="microTransition" aria-label="收起播放器" @click="$emit('close')">
                                <ChevronDown :size="18" :stroke-width="1.5" />
                            </MotionButton>
                            <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                                :transition="microTransition" aria-label="返回播放控制" @click="closePlaybackOptions">
                                <ArrowLeft :size="18" :stroke-width="1.5" />
                            </MotionButton>
                            <span class="playback-options-title">播放页选项</span>
                        </MotionDiv>

                        <MotionDiv class="playback-option lyrics-size-option" :initial="settingsItemInitial"
                            :animate="settingsItemAnimate" :transition="settingsItemTransition(1)">
                            <div class="option-label-row">
                                <span class="option-label">歌词大小</span>
                                <output class="option-value" aria-live="polite">{{ lyricsFontSizeValue }}px</output>
                            </div>
                            <div class="lyrics-size-slider">
                                <span class="size-mark size-mark-small" aria-hidden="true">A</span>
                                <input ref="lyricsRangeRef" class="lyrics-size-range" type="range" min="20" max="56"
                                    step="1" :value="lyricsFontSizeValue" :style="lyricsRangeStyle" aria-label="歌词大小"
                                    :aria-valuenow="lyricsFontSizeValue" aria-valuemin="20" aria-valuemax="56"
                                    :aria-valuetext="`${lyricsFontSizeValue} 像素`" @input="handleLyricsFontSizeInput"
                                    @keydown="handleLyricsRangeKeydown" />
                                <span class="size-mark size-mark-large" aria-hidden="true">A</span>
                            </div>
                        </MotionDiv>

                        <MotionDiv class="playback-option background-option-group" role="group" aria-label="背景"
                            :initial="settingsItemInitial" :animate="settingsItemAnimate"
                            :transition="settingsItemTransition(2)">
                            <span class="option-label">背景</span>
                            <div class="background-options" role="radiogroup" aria-label="背景模式">
                                <MotionButton class="background-option"
                                    :class="{ active: normalizedBackgroundMode === 'flowing' }"
                                    :while-hover="buttonHover" :while-press="buttonPress" :transition="microTransition"
                                    role="radio" :aria-checked="normalizedBackgroundMode === 'flowing'"
                                    @click="setBackgroundMode('flowing')">
                                    流沙
                                </MotionButton>
                                <MotionButton class="background-option"
                                    :class="{ active: normalizedBackgroundMode === 'blur' }" :while-hover="buttonHover"
                                    :while-press="buttonPress" :transition="microTransition" role="radio"
                                    :aria-checked="normalizedBackgroundMode === 'blur'"
                                    @click="setBackgroundMode('blur')">
                                    高斯模糊
                                </MotionButton>
                            </div>
                        </MotionDiv>
                    </MotionDiv>
                </AnimatePresence>
            </footer>
        </div>
    </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { ArrowLeft, ChevronDown, ListMusic, Maximize2, MoreHorizontal, PanelTop, Pause, Play, Shuffle, SkipBack, SkipForward, SlidersHorizontal, SquarePen, Volume2 } from '@lucide/vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import MotionTransition from './MotionTransition.vue'
import FlowingBackground from './FlowingBackground.vue'
import { bassCall } from '../services/bassApi.js'
import { APPLE_SPRING, INSTANT_MOTION, LINEAR_LOOP, MICRO_SPRING, SOFT_SPRING } from '../utils/motion.js'
import { User2Icon } from '@lucide/vue'
import { DiscAlbum } from '@lucide/vue'

const props = defineProps({
    isVisible: {
        type: Boolean,
        default: false
    },
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
    }
})

const emit = defineEmits([
    'close',
    'toggle-play',
    'previous',
    'next',
    'progress-change',
    'progress-commit',
    'volume-change',
    'shuffle',
    'repeat',
    'add-to-playlist',
    'queue',
    'background-mode-change'
])

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const isProgressDragging = ref(false)
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const contentTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)
const instantTransition = computed(() => INSTANT_MOTION)
const progressTransition = computed(() => isProgressDragging.value ? INSTANT_MOTION : microTransition.value)
const loopTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : LINEAR_LOOP)
const footerTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const footerViewInitial = { opacity: 0, y: 96, filter: 'blur(8px)' }
const footerViewAnimate = { opacity: 1, y: 0, filter: 'blur(0px)' }
const footerViewExit = { opacity: 0, y: 96, filter: 'blur(8px)' }
const settingsItemInitial = { opacity: 0, y: 16 }
const settingsItemAnimate = { opacity: 1, y: 0 }
const settingsItemTransition = (index) => reducedMotion.value
    ? INSTANT_MOTION
    : { ...APPLE_SPRING, delay: index * 0.06 }
const buttonHover = { scale: 1.08 }
const buttonPress = { scale: 0.92 }

const isPlaybackOptionsOpen = ref(false)
const compactViewport = ref(typeof window !== 'undefined' && window.matchMedia('(max-width: 720px)').matches)
const lyricsRangeRef = ref(null)
const lyricsFontSize = ref(null)
const lyricsFontSizeValue = computed(() => lyricsFontSize.value ?? (compactViewport.value ? 22 : 32))
const lyricsStyle = computed(() => ({ '--lyrics-font-size': `${lyricsFontSizeValue.value}px` }))
const lyricsRangeStyle = computed(() => {
    const progress = ((lyricsFontSizeValue.value - 20) / (56 - 20)) * 100
    return {
        background: `linear-gradient(to right, rgba(var(--primary-color), 0.9) 0%, rgba(var(--primary-color), 0.9) ${progress}%, rgba(255, 255, 255, 0.22) ${progress}%, rgba(255, 255, 255, 0.22) 100%)`
    }
})

const ZERO_BANDS = { bass: 0, mid: 0, treble: 0, level: 0 }
const audioBands = ref({ ...ZERO_BANDS })
let audioBandsTimer
let audioBandsRequest = 0
let audioBandsInFlight = false

const bassTrackInfo = ref(null)
let bassTrackInfoRequest = 0
let bassTrackInfoInFlight = false

const formatSampleRate = (value) => {
    const sampleRate = Number(value)
    if (!Number.isFinite(sampleRate) || sampleRate <= 0) return ''
    const kiloHertz = sampleRate / 1000
    return `${Number.isInteger(kiloHertz) ? kiloHertz : kiloHertz.toFixed(1).replace(/\.0$/, '')}KHz`
}

const formatBitrate = (value) => {
    const bitrate = Number(value)
    if (!Number.isFinite(bitrate) || bitrate <= 0) return ''
    return `${Math.round(bitrate)}Kbps`
}

const bassFormatName = (info) => {
    const bassFormat = String(info?.format || '').trim()
    if (bassFormat) return `${bassFormat.toUpperCase()} Audio`

    const source = String(info?.filename || '')
        .split(/[?#]/, 1)[0]
        .split(/[\\/]/).pop() || ''
    const extension = source.includes('.') ? source.split('.').pop().toUpperCase() : ''
    const fallback = String(props.currentSong?.format || props.currentSong?.codec || '')
        .replace(/^\./, '')
        .replace(/\s+Audio$/i, '')
        .trim()
    const format = extension || fallback
    return format ? `${format.toUpperCase()} Audio` : ''
}

const songTags = computed(() => {
    const info = bassTrackInfo.value
    if (!info) return []

    return [
        { key: 'sample-rate', label: formatSampleRate(info.frequency) },
        { key: 'bitrate', label: formatBitrate(info.bitrate ?? props.currentSong?.bitrate) },
        {
            key: 'channels',
            label: Number(info.channels) > 0
                ? `${info.channels} ${Number(info.channels) === 1 ? 'Channel' : 'Channels'}`
                : ''
        },
        { key: 'format', label: bassFormatName(info) }
    ].filter((tag) => tag.label)
})

const normalizedBackgroundMode = computed(() => props.backgroundMode === 'blur' ? 'blur' : 'flowing')

const stopAudioBands = () => {
    if (audioBandsTimer) window.clearInterval(audioBandsTimer)
    audioBandsTimer = undefined
    audioBandsRequest++
    audioBandsInFlight = false
    audioBands.value = { ...ZERO_BANDS }
}

const refreshAudioBands = async () => {
    if (!props.isVisible || normalizedBackgroundMode.value !== 'flowing' || !props.channelId || audioBandsInFlight) return
    const requestId = ++audioBandsRequest
    const channelId = props.channelId
    audioBandsInFlight = true
    try {
        const result = await bassCall('bass_channel_fft', { channelId, fftSize: 512 })
        if (requestId === audioBandsRequest && channelId === props.channelId) {
            audioBands.value = result?.bands ? { ...ZERO_BANDS, ...result.bands } : { ...ZERO_BANDS }
        }
    } catch (error) {
        if (requestId === audioBandsRequest) audioBands.value = { ...ZERO_BANDS }
    } finally {
        audioBandsInFlight = false
    }
}

const startAudioBands = () => {
    stopAudioBands()
    if (!props.isVisible || normalizedBackgroundMode.value !== 'flowing' || !props.channelId) return
    refreshAudioBands()
    audioBandsTimer = window.setInterval(refreshAudioBands, 100)
}

const stopBassTrackInfo = () => {
    bassTrackInfoRequest++
    bassTrackInfoInFlight = false
    bassTrackInfo.value = null
}

const refreshBassTrackInfo = async () => {
    if (!props.isVisible || !props.channelId || bassTrackInfoInFlight) return
    const requestId = ++bassTrackInfoRequest
    const channelId = props.channelId
    bassTrackInfoInFlight = true
    try {
        const result = await bassCall('bass_channel_info', { channelId })
        if (requestId === bassTrackInfoRequest && channelId === props.channelId) {
            bassTrackInfo.value = result
        }
    } catch (error) {
        if (requestId === bassTrackInfoRequest) bassTrackInfo.value = null
    } finally {
        bassTrackInfoInFlight = false
    }
}

const startBassTrackInfo = () => {
    stopBassTrackInfo()
    if (!props.isVisible || !props.channelId) return
    refreshBassTrackInfo()
}

const openPlaybackOptions = () => {
    isPlaybackOptionsOpen.value = true
}

const closePlaybackOptions = () => {
    isPlaybackOptionsOpen.value = false
}

const setBackgroundMode = (mode) => {
    if (mode === normalizedBackgroundMode.value) return
    emit('background-mode-change', mode)
}

const handleLyricsFontSizeInput = (event) => {
    const nextSize = Number(event.target.value)
    lyricsFontSize.value = Math.max(20, Math.min(56, nextSize))
}

const handleLyricsRangeKeydown = (event) => {
    if (event.key !== 'Escape') event.stopPropagation()
}

const updateCompactViewport = () => {
    compactViewport.value = window.matchMedia('(max-width: 720px)').matches
}

const handlePlaybackOptionsAnimationComplete = () => {
    if (isPlaybackOptionsOpen.value) lyricsRangeRef.value?.focus()
}

const volume = ref(75)
const albumStageRef = ref(null)
const lyricsWindowRef = ref(null)
const lyricRowRefs = new Map()
const albumSize = ref(0)
let albumResizeObserver
let lyricsResizeObserver
const tickCount = 52
const ticks = Array.from({ length: tickCount }, (_, index) => (index * 360) / tickCount)
const syncedLyrics = computed(() => props.lyrics?.lines || [])
const interludes = computed(() => props.lyrics?.interludes || [])
const plainLyrics = computed(() => props.lyrics?.plainLines || [])
const lyricTimelineRows = computed(() => {
    const rows = [
        ...syncedLyrics.value.map((line, index) => ({
            type: 'line',
            key: `line-${line.startTimeMs}-${index}`,
            startTimeMs: line.startTimeMs,
            endTimeMs: line.endTimeMs,
            line
        })),
        ...interludes.value.map((interlude, index) => ({
            type: 'interlude',
            key: `interlude-${interlude.startTimeMs}-${index}`,
            startTimeMs: interlude.startTimeMs,
            endTimeMs: interlude.endTimeMs
        }))
    ]

    return rows.sort((left, right) => left.startTimeMs - right.startTimeMs)
})
const activeLyricTimelineRow = computed(() => lyricTimelineRows.value.find((row) =>
    props.currentTimeMs >= row.startTimeMs && props.currentTimeMs < row.endTimeMs
))
const lyricRows = computed(() => {
    const activeInterludeKey = activeLyricTimelineRow.value?.type === 'interlude'
        ? activeLyricTimelineRow.value.key
        : null

    return lyricTimelineRows.value.filter((row) =>
        row.type !== 'interlude' || row.key === activeInterludeKey
    )
})
const activeLyricRowIndex = computed(() => lyricRows.value.findIndex((row) =>
    props.currentTimeMs >= row.startTimeMs && props.currentTimeMs < row.endTimeMs
))
const lyricOffset = ref(0)

const setLyricRowRef = (key) => (value) => {
    const element = value?.$el ?? value
    if (element) lyricRowRefs.set(key, element)
    else lyricRowRefs.delete(key)
}

const measureLyrics = async () => {
    await nextTick()
    const windowElement = lyricsWindowRef.value
    const activeRowKey = activeLyricTimelineRow.value?.key
    const lineElement = activeRowKey ? lyricRowRefs.get(activeRowKey) : null
    if (!windowElement || !lineElement || !activeRowKey) {
        lyricOffset.value = 0
        return
    }

    lyricOffset.value = windowElement.clientHeight / 2
        - lineElement.offsetTop
        - lineElement.offsetHeight / 2
}
const albumVisualStyle = computed(() => {
    const size = albumSize.value || 300
    return {
        ...(albumSize.value > 0 ? {
            width: `${albumSize.value}px`,
            height: `${albumSize.value}px`
        } : {}),
        '--tick-radius': `${size * 0.46}px`,
        '--tick-width': `${Math.max(4, size * 0.018)}px`,
        '--tick-height': `${Math.max(12, size * 0.065)}px`
    }
})

const getLyricState = (index) => {
    const distance = activeLyricRowIndex.value < 0
        ? index + 1
        : Math.abs(index - activeLyricRowIndex.value)
    return {
        opacity: distance === 0 ? 1 : Math.max(0.22, 0.7 - distance * 0.13),
        scale: distance === 0 ? 1 : Math.max(0.88, 1 - distance * 0.035),
        filter: distance === 0 ? 'blur(0px)' : `blur(${Math.min(6, distance * 1.5)}px)`,
        color: distance === 0 ? '#ffffff' : 'rgba(255, 255, 255, 0.5)',
        fontWeight: distance === 0 ? 750 : 600
    }
}

const progressFromPointer = (event) => {
    const rect = event.currentTarget.getBoundingClientRect()
    const percent = ((event.clientX - rect.left) / rect.width) * 100
    return Math.max(0, Math.min(100, percent))
}

const handleProgressPointerDown = (event) => {
    if (event.button !== 0) return
    isProgressDragging.value = true
    event.currentTarget.setPointerCapture?.(event.pointerId)
    emit('progress-change', progressFromPointer(event))
}

const handleProgressPointerMove = (event) => {
    if (!isProgressDragging.value) return
    emit('progress-change', progressFromPointer(event))
}

const handleProgressPointerUp = (event) => {
    if (!isProgressDragging.value) return
    const nextProgress = progressFromPointer(event)
    emit('progress-change', nextProgress)
    emit('progress-commit', nextProgress)
    event.currentTarget.releasePointerCapture?.(event.pointerId)
    isProgressDragging.value = false
}

const handleProgressKeydown = (event) => {
    let nextProgress = props.progress
    if (event.key === 'ArrowLeft' || event.key === 'ArrowDown') nextProgress -= 5
    else if (event.key === 'ArrowRight' || event.key === 'ArrowUp') nextProgress += 5
    else if (event.key === 'Home') nextProgress = 0
    else if (event.key === 'End') nextProgress = 100
    else return

    event.preventDefault()
    event.stopPropagation()
    const boundedProgress = Math.max(0, Math.min(100, nextProgress))
    emit('progress-change', boundedProgress)
    emit('progress-commit', boundedProgress)
}

const handleKeydown = (event) => {
    if (!props.isVisible) return

    switch (event.key) {
        case 'Escape':
            if (isPlaybackOptionsOpen.value) closePlaybackOptions()
            else emit('close')
            break
        case ' ':
            event.preventDefault()
            emit('toggle-play')
            break
        case 'ArrowLeft':
            emit('previous')
            break
        case 'ArrowRight':
            emit('next')
            break
    }
}

const updateAlbumSize = () => {
    const stage = albumStageRef.value
    if (!stage) return

    const nextSize = Math.max(0, Math.min(stage.clientWidth, stage.clientHeight, 560))
    if (nextSize > 0) albumSize.value = nextSize
}

onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
    window.addEventListener('resize', updateCompactViewport)
    albumResizeObserver = new ResizeObserver(updateAlbumSize)
    if (albumStageRef.value) albumResizeObserver.observe(albumStageRef.value)
    lyricsResizeObserver = new ResizeObserver(measureLyrics)
    if (lyricsWindowRef.value) lyricsResizeObserver.observe(lyricsWindowRef.value)
    requestAnimationFrame(updateAlbumSize)
    measureLyrics()
    startAudioBands()
})

onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
    window.removeEventListener('resize', updateCompactViewport)
    albumResizeObserver?.disconnect()
    lyricsResizeObserver?.disconnect()
    lyricRowRefs.clear()
    stopAudioBands()
    stopBassTrackInfo()
})

watch([lyricRows, activeLyricRowIndex, activeLyricTimelineRow], measureLyrics, { deep: true, flush: 'post' })
watch(() => [props.isVisible, props.channelId, normalizedBackgroundMode.value], startAudioBands)
watch(() => [props.isVisible, props.channelId, props.currentSong?.id, props.currentSong?.title], startBassTrackInfo, { immediate: true })
</script>

<style scoped>
.fullscreen-player {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    color: #f7f5f3;
    background: #090807;
    isolation: isolate;
}

.player-background,
.backdrop-image,
.backdrop-wash,
.backdrop-vignette {
    position: absolute;
    inset: 0;
}

.player-background {
    z-index: 0;
    overflow: hidden;
    background: #090807;
}

.backdrop-image {
    background-position: center;
    background-size: cover;
    filter: blur(48px) saturate(1.15) brightness(0.65);
    opacity: 0.62;
    transform: scale(1.14);
}

.backdrop-wash {
    background: rgba(7, 5, 5, 0.18);
}

.backdrop-vignette {
    box-shadow: inset 0 0 220px rgba(0, 0, 0, 0.6);
}

.player-container {
    position: relative;
    z-index: 1;
    width: 100%;
    height: 100%;
    overflow: hidden;
}

.player-main {
    display: grid;
    position: relative;
    z-index: 1;
    grid-template-columns: minmax(420px, 50%) minmax(0, 50%);
    width: 100%;
    height: calc(100% - 156px);
    min-height: 0;
}

.visual-column {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-start;
    padding: clamp(80px, 18vh, 250px) clamp(32px, 8.8vw, 230px) 0;
    min-height: 0;
    overflow: hidden;
}

.album-stage {
    flex: 1 1 auto;
    min-height: 0;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

.album-visual {
    position: relative;
    flex: 0 0 auto;
    width: min(100%, 560px);
    height: auto;
    aspect-ratio: 1;
    margin-left: 0;
    display: grid;
    place-items: center;
    transform-origin: center;
    --tick-radius: 138px;
}

.tick-ring {
    position: absolute;
    inset: 0;
}

.tick {
    position: absolute;
    top: 50%;
    left: 50%;
    width: var(--tick-width, 5px);
    height: var(--tick-height, 18px);
    border-radius: 99px;
    background: rgba(115, 105, 99, 0.52);
    transform-origin: center;
}

.disc-shell {
    position: relative;
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    border: clamp(7px, 0.7vw, 13px) solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.2);
    /* box-shadow: 0 18px 38px rgba(66, 45, 39, 0.22); */
}

.disc-shell::before {
    content: '';
    position: absolute;
    inset: 8%;
    border-radius: 50%;
    border: 2px solid rgba(101, 87, 80, 0.44);
    pointer-events: none;
}

.album-cover-frame {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    transform-origin: 50% 50%;
}

.album-cover {
    position: absolute;
    inset: 2.5%;
    width: 95%;
    height: 95%;
    border-radius: 50%;
    object-fit: cover;
    transform-origin: 50% 50%;
    box-shadow: 0 8px 20px rgba(48, 31, 26, 0.28);
}

.song-details {
    flex: 0 0 auto;
    width: min(100%, 500px);
    margin: 50px 0 52px clamp(0px, 3vw, 75px);
    text-align: left;
}

.song-title,
.song-artist,
.song-album {
    margin: 0;
    color: #f7f5f3;
    display: flex;
    gap: 10px;
}

.song-title {
    font-size: 20px;
    font-weight: bold;
}

.song-artist {
    margin-top: 20px;
    font-size: 16px;
    font-weight: normal;
    opacity: 0.5;
}

.song-album {
    margin-top: 10px;
    font-size: 16px;
    opacity: 0.5;
}

.song-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 20px;
}

.tag {
    padding: 3px 5px;
    border-radius: 5px;
    color: rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.12);
    font-size: 11px;
    /* font-weight: 650; */
    line-height: 1;
}

.lyrics-column {
    position: relative;
    min-width: 0;
    height: 100%;
    min-height: 0;
    overflow: hidden;
}

.lyrics-window {
    position: absolute;
    inset: 6% 8% 10% 2%;
    overflow: hidden;
    display: flex;
    align-items: stretch;
    justify-content: center;
    mask-image: linear-gradient(to bottom, transparent 0%, #000 16%, #000 84%, transparent 100%);
    -webkit-mask-image: linear-gradient(to bottom, transparent 0%, #000 16%, #000 84%, transparent 100%);
    height: 95%
}

.lyrics-track {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 18px;
    padding: 38% 0;
    transform-origin: center;
    padding-left: 10px;
}

.lyric-line {
    width: 100%;
    min-height: 44px;
    padding: 5px 0;
    color: rgba(255, 255, 255, 0.5);
    font-size: var(--lyrics-font-size, 32px);
    line-height: 1.18;
    letter-spacing: -0.035em;
    text-align: left;
    transform-origin: left center;
    will-change: transform, opacity, filter;
}

.lyric-primary {
    font-weight: inherit;
}

.lyric-secondary {
    margin-top: 8px;
    color: rgba(255, 255, 255, 0.58);
    font-size: 1em;
    font-weight: 550;
    line-height: 1.25;
    letter-spacing: -0.015em;
}

.lyrics-status,
.plain-lyrics {
    width: 100%;
    color: rgba(255, 255, 255, 0.52);
    text-align: left;
}

.lyrics-status {
    align-self: center;
    padding: 0 8%;
    font-size: 22px;
}

.lyric-interlude-row {
    display: flex;
    align-items: center;
    justify-content: flex-start;
}

.lyric-interlude-icon {
    flex: 0 0 auto;
}

.plain-lyrics {
    align-self: center;
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-height: 76%;
    overflow: hidden;
    padding: 0 4%;
}

.plain-lyric-line {
    font-size: var(--lyrics-font-size, 32px);
    line-height: 1.35;
}

@media (max-height: 820px) and (min-width: 721px) {

    .album-visual {
        width: min(460px, 42vh, 36vw);
    }

    .song-tags {
        gap: 7px;
        margin-top: 14px;
    }

    .tag {
        padding: 5px 9px;
    }
}

.player-footer {
    position: absolute;
    z-index: 2;
    right: 0;
    bottom: 0;
    left: 0;
    height: 95px;
    overflow: hidden;
    padding: 0 20px;
}

.footer-view {
    width: 100%;
    min-height: 62px;
    will-change: transform, opacity, filter;
}

.transport-view {
    display: flex;
    align-items: center;
}

.footer-actions {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    width: 100%;
    height: 62px;
}

.playback-options {
    display: grid;
    grid-template-columns: minmax(180px, 0.8fr) minmax(300px, 1.25fr) minmax(250px, 1fr);
    align-items: center;
    gap: clamp(26px, 5vw, 88px);
}

.playback-options-leading,
.playback-option,
.background-options,
.lyrics-size-slider,
.option-label-row {
    display: flex;
    align-items: center;
}

.playback-options-leading {
    gap: 14px;
    min-width: 0;
}

.playback-options-title {
    overflow: hidden;
    color: rgba(255, 255, 255, 0.86);
    font-size: 14px;
    font-weight: 650;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.playback-option {
    min-width: 0;
    gap: 18px;
}

.lyrics-size-option {
    display: grid;
    gap: 8px;
}

.option-label-row {
    justify-content: space-between;
    gap: 16px;
}

.option-label {
    color: rgba(255, 255, 255, 0.58);
    font-size: 12px;
    white-space: nowrap;
    margin-left: auto;
}

.option-value {
    min-width: 42px;
    color: rgba(255, 255, 255, 0.86);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    text-align: right;
}

.lyrics-size-slider {
    gap: 10px;
    width: 100%;
}

.size-mark {
    flex: 0 0 auto;
    color: rgba(255, 255, 255, 0.5);
    font-weight: 700;
    line-height: 1;
}

.size-mark-small {
    font-size: 11px;
}

.size-mark-large {
    font-size: 18px;
}

.lyrics-size-range {
    flex: 1;
    min-width: 100px;
    height: 4px;
    padding: 0;
    border: 0;
    border-radius: 99px;
    appearance: none;
    cursor: pointer;
    outline: none;
}

.lyrics-size-range::-webkit-slider-thumb {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.92);
    border-radius: 50%;
    background: rgb(var(--primary-color));
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
    appearance: none;
}

.lyrics-size-range::-moz-range-thumb {
    width: 10px;
    height: 10px;
    border: 2px solid rgba(255, 255, 255, 0.92);
    border-radius: 50%;
    background: rgb(var(--primary-color));
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
}

.lyrics-size-range:focus-visible {
    outline: 2px solid rgba(var(--primary-color), 0.72);
    outline-offset: 5px;
}

.background-option-group {
    justify-content: space-between;
}

.background-options {
    gap: 8px;
}

.background-option {
    min-width: 76px;
    padding: 8px 12px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 9px;
    color: rgba(255, 255, 255, 0.66);
    background: rgba(255, 255, 255, 0.06);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
}

.background-option.active {
    border-color: rgba(var(--primary-color), 0.72);
    color: #ffffff;
    background: rgba(var(--primary-color), 0.22);
}

.footer-side {
    display: flex;
    align-items: center;
    align-self: center;
    height: 48px;
    gap: 16px;
}

.footer-side-right {
    justify-content: flex-end;
}

.transport-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: center;
    height: 48px;
    gap: 28px;
}

.transport-column {
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: center;
    flex-direction: column;
}

.footer-button,
.play-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    color: #f7f5f3;
    background: transparent;
    cursor: pointer;
}

.footer-button {
    width: 32px;
    height: 32px;
}

.play-button {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.14);
}

.progress-section {
    display: flex;
    align-items: center;
    gap: 24px;
    width: min(48vw, 900px);
    margin: 13px auto 0;
}

.time-display {
    min-width: 42px;
    color: rgba(255, 255, 255, 0.68);
    font-size: 12px;
    text-align: center;
}

.progress-container {
    flex: 1;
    min-width: 0;
    padding: 9px 0;
    cursor: grab;
    outline: none;
    touch-action: none;
    user-select: none;
}

.progress-container.is-dragging {
    cursor: grabbing;
}

.progress-track {
    position: relative;
    width: 100%;
    height: 3px;
    overflow: visible;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.3);
}

.progress-fill {
    height: 100%;
    border-radius: inherit;
    background: rgb(var(--primary-color));
}

.progress-thumb {
    position: absolute;
    top: 50%;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: rgb(var(--primary-color));
    box-shadow: 0 2px 7px rgba(0, 0, 0, 0.4);
    opacity: 0;
    pointer-events: none;
    transform: translate(-50%, -50%) scale(0.5);
    transition: opacity 120ms ease, transform 120ms ease;
}

.progress-container:hover .progress-thumb,
.progress-container:focus-visible .progress-thumb,
.progress-container.is-dragging .progress-thumb {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
}

@media (max-width: 1050px) {
    .player-main {
        grid-template-columns: minmax(340px, 46%) minmax(0, 54%);
    }

    .visual-column {
        padding-left: 7vw;
        padding-right: 3vw;
    }

    .album-visual {
        width: min(37vw, 48vh, 420px);
        min-width: 270px;
    }

    .lyric-line {
        font-size: var(--lyrics-font-size, 32px);
    }

    .playback-options {
        grid-template-columns: minmax(155px, 0.65fr) minmax(220px, 1.25fr) minmax(200px, 0.85fr);
        gap: 18px;
        padding: 0 20px;
    }

    .transport-controls {
        gap: 14px;
    }

    .play-button {
        width: 48px;
        height: 48px;
    }
}

@media (max-width: 720px) {
    .player-main {
        display: block;
        height: calc(100% - 174px);
    }

    .visual-column {
        align-items: center;
        height: 55%;
        padding: 25px 22px 0;
    }

    .album-visual {
        width: min(52vw, 34vh);
        min-width: 180px;
    }

    .song-details {
        width: 100%;
        margin-top: 12px;
        text-align: center;
    }

    .song-title {
        font-size: 22px;
    }

    .song-artist,
    .song-album {
        margin-top: 8px;
    }

    .song-tags {
        justify-content: center;
        margin-top: 11px;
    }

    .lyrics-column {
        height: 45%;
    }

    .lyrics-window {
        inset: 0 24px;
    }

    .lyrics-track {
        gap: 18px;
    }

    .lyric-line {
        min-height: 44px;
        font-size: var(--lyrics-font-size, 22px);
        text-align: center;
    }

    .playback-options {
        grid-template-areas:
            'leading background'
            'lyrics lyrics';
        grid-template-columns: auto minmax(0, 1fr);
        gap: 10px 12px;
        align-content: center;
        height: 100%;
        padding: 14px 20px;
    }

    .playback-options-leading {
        grid-area: leading;
    }

    .lyrics-size-option {
        grid-area: lyrics;
    }

    .background-option-group {
        grid-area: background;
        justify-content: flex-end;
        gap: 10px;
    }

    .background-option {
        min-width: auto;
        padding: 7px 9px;
    }

    .playback-options-title {
        font-size: 12px;
    }

    .player-footer {
        bottom: 0;
        height: 140px;
        padding: 0 20px;
    }

    .footer-actions {
        grid-template-columns: auto 1fr auto;
    }

    .footer-side {
        gap: 0;
    }

    .footer-side-left .footer-button:nth-child(2) {
        display: none;
    }

    .volume-button {
        display: none;
    }

    .transport-controls {
        gap: 3px;
    }

    .footer-button {
        width: 24px;
        height: 24px;
    }

    .play-button {
        width: 40px;
        height: 40px;
    }

    .progress-section {
        width: 100%;
        gap: 8px;
        margin-top: 13px;
    }

    .time-display {
        min-width: 38px;
        font-size: 12px;
    }
}
</style>
