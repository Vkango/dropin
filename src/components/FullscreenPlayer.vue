<template>
    <div class="fullscreen-player" @click.self="$emit('close')">
        <div class="player-background" aria-hidden="true">
            <div class="backdrop-image" :style="{ backgroundImage: `url(${currentSong.cover})` }"></div>
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
                            <div class="tick-ring" aria-hidden="true">
                                <span v-for="tick in ticks" :key="tick" class="tick"
                                    :style="{ transform: `translate(-50%, -50%) rotate(${tick}deg) translateY(calc(-1 * var(--tick-radius)))` }"></span>
                            </div>
                            <div class="disc-shell">
                                <MotionTransition variant="albumCover" mode="out-in">
                                    <img :key="currentSong.cover" :src="currentSong.cover" :alt="currentSong.title"
                                        class="album-cover" />
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

                            <div class="song-tags">
                                <span class="tag">44.1KHz</span>
                                <span class="tag">1696Kbps</span>
                                <span class="tag">2 Channels</span>
                                <span class="tag">FLAC Audio</span>
                            </div>
                        </div>
                    </MotionTransition>
                </section>

                <section class="lyrics-column" aria-label="歌词">
                    <div class="lyrics-window">
                        <MotionDiv class="lyrics-track" :animate="{ y: lyricOffset }" :transition="contentTransition">
                            <MotionDiv v-for="(line, index) in displayLyrics" :key="`${currentSong.title}-${index}`"
                                class="lyric-line" :animate="getLyricState(index)" :transition="contentTransition">
                                {{ line }}
                            </MotionDiv>
                        </MotionDiv>
                    </div>
                </section>
            </main>

            <footer class="player-footer">
                <div class="footer-actions">
                    <div class="footer-side footer-side-left">
                        <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                            :transition="microTransition" aria-label="收起播放器" @click="$emit('close')">
                            <ChevronDown :size="18" :stroke-width="1.5" />
                        </MotionButton>
                        <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                            :transition="microTransition" aria-label="全屏显示">
                            <Maximize2 :size="18" :stroke-width="1.5" />
                        </MotionButton>
                        <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                            :transition="microTransition" aria-label="编辑播放页">
                            <SquarePen :size="18" :stroke-width="1.5" />
                        </MotionButton>
                    </div>

                    <div class="transport-column">
                        <div class="transport-controls">
                            <MotionButton class="footer-button volume-button" :while-hover="buttonHover"
                                :while-press="buttonPress" :transition="microTransition" aria-label="音量">
                                <Volume2 :size="18" :stroke-width="1.5" />
                            </MotionButton>
                            <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                                :transition="microTransition" aria-label="上一首" @click="$emit('previous')">
                                <SkipBack :size="18" :stroke-width="1.5" />
                            </MotionButton>
                            <MotionButton class="play-button" :while-hover="{ scale: 1.06 }"
                                :while-press="{ scale: 0.94 }" :transition="microTransition"
                                :aria-label="isPlaying ? '暂停' : '播放'" @click="$emit('toggle-play')">
                                <Pause v-if="isPlaying" :size="18" :stroke-width="1.8" />
                                <Play v-else :size="18" :stroke-width="1.8" />
                            </MotionButton>
                            <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                                :transition="microTransition" aria-label="下一首" @click="$emit('next')">
                                <SkipForward :size="18" :stroke-width="1.5" />
                            </MotionButton>
                            <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                                :transition="microTransition" aria-label="随机播放" @click="$emit('shuffle')">
                                <Shuffle :size="18" :stroke-width="1.5" />
                            </MotionButton>
                        </div>


                        <div class="progress-section">
                            <span class="time-display">{{ currentTime }}</span>
                            <div class="progress-container" role="slider" :aria-valuenow="progress" aria-valuemin="0"
                                aria-valuemax="100" tabindex="0" @click="handleProgressClick">
                                <div class="progress-track">
                                    <MotionDiv class="progress-fill" :animate="{ width: `${progress}%` }"
                                        :transition="microTransition"></MotionDiv>
                                    <MotionDiv class="progress-thumb" :animate="{ left: `${progress}%` }"
                                        :transition="microTransition"></MotionDiv>
                                </div>
                            </div>
                            <span class="time-display">{{ totalTime }}</span>
                        </div>
                    </div>

                    <div class="footer-side footer-side-right">
                        <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                            :transition="microTransition" aria-label="播放设置">
                            <SlidersHorizontal :size="18" :stroke-width="1.5" />
                        </MotionButton>
                        <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                            :transition="microTransition" aria-label="歌词面板">
                            <PanelTop :size="18" :stroke-width="1.5" />
                        </MotionButton>
                        <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                            :transition="microTransition" aria-label="播放队列" @click="$emit('queue')">
                            <ListMusic :size="18" :stroke-width="1.5" />
                        </MotionButton>
                        <MotionButton class="footer-button" :while-hover="buttonHover" :while-press="buttonPress"
                            :transition="microTransition" aria-label="更多操作">
                            <MoreHorizontal :size="18" :stroke-width="1.5" />
                        </MotionButton>
                    </div>
                </div>

            </footer>
        </div>
    </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { ChevronDown, ListMusic, Maximize2, MoreHorizontal, PanelTop, Pause, Play, Shuffle, SkipBack, SkipForward, SlidersHorizontal, SquarePen, Volume2 } from '@lucide/vue'
import { motion, useReducedMotion } from 'motion-v'
import MotionTransition from './MotionTransition.vue'
import { INSTANT_MOTION, LINEAR_LOOP, MICRO_SPRING, SOFT_SPRING } from '../utils/motion.js'
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
    totalTime: {
        type: String,
        default: '00:00'
    },
    progress: {
        type: Number,
        default: 0
    }
})

const emit = defineEmits([
    'close',
    'toggle-play',
    'previous',
    'next',
    'progress-change',
    'volume-change',
    'shuffle',
    'repeat',
    'add-to-playlist',
    'queue'
])

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const contentTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)
const instantTransition = computed(() => INSTANT_MOTION)
const loopTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : LINEAR_LOOP)
const buttonHover = { scale: 1.08 }
const buttonPress = { scale: 0.92 }

const volume = ref(75)
const albumStageRef = ref(null)
const albumSize = ref(0)
let albumResizeObserver
const tickCount = 52
const ticks = Array.from({ length: tickCount }, (_, index) => (index * 360) / tickCount)
const lyricStep = 62

const createTestLyrics = () => [
    { startTime: 0, endTime: 3000, words: [{ word: '申必' }, { word: '申必比' }, { word: '我' }] },
    { startTime: 3000, endTime: 6000, words: [{ word: '能够' }, { word: '遇见' }, { word: '你' }] },
    { startTime: 6000, endTime: 9000, words: [{ word: '在最美好的' }, { word: '时光' }, { word: '里' }] },
    { startTime: 9000, endTime: 12000, words: [{ word: '音乐' }, { word: '播放着' }, { word: '一点光彩' }] },
    { startTime: 12000, endTime: 15000, words: [{ word: '感觉' }, { word: '是' }, { word: '特别的美' }] },
    { startTime: 15000, endTime: 18000, words: [{ word: '曾尝' }, { word: '遗失' }, { word: '意时' }] },
    { startTime: 18000, endTime: 21000, words: [{ word: '却找到' }, { word: '快乐' }, { word: '点' }] },
    { startTime: 21000, endTime: 24000, words: [{ word: '联想' }, { word: '会' }, { word: '知' }] },
    { startTime: 24000, endTime: 27000, words: [{ word: '就是' }, { word: '自己' }] },
    { startTime: 27000, endTime: 30000, words: [{ word: '原来是个' }, { word: '幸运儿' }] },
    { startTime: 30000, endTime: 33000, words: [{ word: '每人' }, { word: '命中' }, { word: '都有责' }] },
    { startTime: 33000, endTime: 36000, words: [{ word: '每一刻' }, { word: '都' }, { word: '计算着' }] }
]

const lyricLines = ref(createTestLyrics())
const currentTimeMs = computed(() => {
    const [minutes, seconds] = props.currentTime.split(':').map(Number)
    return (minutes * 60 + seconds) * 1000
})
const displayLyrics = computed(() => lyricLines.value.map(line => line.words.map(word => word.word).join(' ')))
const activeLyricIndex = computed(() => {
    const index = lyricLines.value.findIndex(line => currentTimeMs.value >= line.startTime && currentTimeMs.value < line.endTime)
    return index >= 0 ? index : Math.max(0, lyricLines.value.length - 1)
})
const lyricOffset = computed(() => -(activeLyricIndex.value * lyricStep + 22))
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
    const distance = Math.abs(index - activeLyricIndex.value)
    return {
        opacity: distance === 0 ? 1 : Math.max(0.22, 0.7 - distance * 0.13),
        scale: distance === 0 ? 1 : Math.max(0.88, 1 - distance * 0.035),
        filter: distance === 0 ? 'blur(0px)' : `blur(${Math.min(6, distance * 1.5)}px)`,
        color: distance === 0 ? '#ffffff' : 'rgba(255, 255, 255, 0.5)',
        fontWeight: distance === 0 ? 750 : 600
    }
}

const handleProgressClick = (event) => {
    const rect = event.currentTarget.getBoundingClientRect()
    const percent = ((event.clientX - rect.left) / rect.width) * 100
    emit('progress-change', Math.max(0, Math.min(100, percent)))
}

const handleKeydown = (event) => {
    if (!props.isVisible) return

    switch (event.key) {
        case 'Escape':
            emit('close')
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
    albumResizeObserver = new ResizeObserver(updateAlbumSize)
    if (albumStageRef.value) albumResizeObserver.observe(albumStageRef.value)
    requestAnimationFrame(updateAlbumSize)
})

onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
    albumResizeObserver?.disconnect()
})

watch(() => props.currentSong.title, () => {
    lyricLines.value = createTestLyrics()
})
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
    z-index: -1;
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
    background:
        linear-gradient(112deg, rgba(0, 0, 0, 0.48) 0%, rgba(39, 11, 7, 0.28) 45%, rgba(0, 0, 0, 0.52) 100%),
        linear-gradient(180deg, rgba(0, 0, 0, 0.16), rgba(0, 0, 0, 0.4));
}

.backdrop-vignette {
    box-shadow: inset 0 0 220px rgba(0, 0, 0, 0.6);
}

.player-container {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
}

.player-main {
    display: grid;
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
    width: 77%;
    height: 77%;
    display: grid;
    place-items: center;
    border: clamp(7px, 0.7vw, 13px) solid rgba(119, 109, 102, 0.62);
    border-radius: 50%;
    background: rgba(217, 207, 200, 0.74);
    box-shadow: 0 18px 38px rgba(66, 45, 39, 0.22);
}

.disc-shell::before {
    content: '';
    position: absolute;
    inset: 8%;
    border-radius: 50%;
    border: 2px solid rgba(101, 87, 80, 0.44);
    pointer-events: none;
}

.album-cover {
    position: absolute;
    inset: 7%;
    width: 86%;
    height: 86%;
    border-radius: 50%;
    object-fit: cover;
    box-shadow: 0 8px 20px rgba(48, 31, 26, 0.28);
}

.song-details {
    flex: 0 0 auto;
    width: min(100%, 500px);
    margin: 28px 0 52px clamp(0px, 3vw, 75px);
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
    mask-image: linear-gradient(to bottom, transparent 0%, #000 16%, #000 84%, transparent 100%);
    -webkit-mask-image: linear-gradient(to bottom, transparent 0%, #000 16%, #000 84%, transparent 100%);
}

.lyrics-track {
    position: absolute;
    top: 50%;
    left: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 18px;
    transform-origin: center;
}

.lyric-line {
    width: 100%;
    min-height: 44px;
    color: rgba(255, 255, 255, 0.5);
    font-size: 32px;
    line-height: 1.18;
    letter-spacing: -0.035em;
    text-align: left;
    transform-origin: left center;
    will-change: transform, opacity, filter;
}

.lyric-line:first-child {
    margin-top: -32px;
}

@media (max-height: 820px) and (min-width: 721px) {
    .visual-column {
        padding: 28px 6vw 0;
    }

    .album-visual {
        width: min(460px, 42vh, 36vw);
    }

    .song-details {
        margin-top: 14px;
    }

    .song-title {
        font-size: 30px;
    }

    .song-artist,
    .song-album {
        margin-top: 10px;
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
    right: 0;
    bottom: 0;
    left: 0;
    height: 95px;
    padding: 0 20px;
}

.footer-actions {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    height: 62px;
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

.footer-button.active {
    color: #f0a07d;
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
    font-weight: 600;
    text-align: center;
}

.progress-container {
    flex: 1;
    min-width: 0;
    cursor: pointer;
    outline: none;
}

.progress-track {
    position: relative;
    width: 100%;
    height: 5px;
    overflow: visible;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.3);
}

.progress-fill {
    height: 100%;
    border-radius: inherit;
    background: #d65a32;
}

.progress-thumb {
    position: absolute;
    top: 50%;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: #d65a32;
    box-shadow: 0 2px 7px rgba(0, 0, 0, 0.4);
    transform: translate(-50%, -50%);
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
        font-size: 32px;
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
        font-size: 22px;
        text-align: center;
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
