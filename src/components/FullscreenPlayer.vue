<template>

    <AnimatePresence>
        <MotionDiv v-if="shouldShow" class="fullscreen-player" :initial="{ opacity: 0 }"
            :animate="{ opacity: 1 }" :exit="{ opacity: 0 }" :transition="springTransition"
            @click.self="$emit('close')">
            <!-- AMLL 背景渲染器 -->
            <div style="position: absolute; width: 100%; height: 100%; background-color: rgb(var(--primary-color));"></div>
            <MotionDiv class="player-container" :initial="playerEnterState" :animate="playerOpenState"
                :exit="playerExitState" :transition="springTransition" @click.stop>
            <!-- 关闭按钮 -->
            <MotionButton class="close-button" :while-hover="{ scale: 1.1 }" :while-press="{ scale: 0.94 }"
                :transition="microTransition" @click="$emit('close')">
                <Icon src="/assets/close.svg" size="md" />
            </MotionButton>

            <!-- 主要内容区 -->
            <div class="player-content">
                <!-- 专辑封面区域 -->
                <div class="album-section">
                    <div class="vinyl-area">
                        <MotionDiv class="vinyl-container" :animate="{ rotate: isPlaying ? 360 : 0 }"
                            :transition="isPlaying ? loopTransition : instantTransition">
                            <div class="vinyl-record">
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
                            <h2 class="song-artist">{{ currentSong.artist }}</h2>
                            <p class="song-album">{{ currentSong.album }}</p>

                            <!-- 歌曲标签 -->
                            <div class="song-tags">
                                <span class="tag">High-Res</span>
                                <span class="tag">FLAC</span>
                                <span class="tag">44.1kHz</span>
                            </div>
                        </div>
                    </MotionTransition>
                </div>

                <!-- 歌曲信息和歌词区域 -->
                <div class="song-info">


                    <!-- AMLL 歌词显示 -->
                    <div class="lyrics-container">

                    </div>
                </div>

                <!-- 播放控制区域 -->
                <div class="player-controls">
                    <!-- 进度条 -->
                    <div class="additional-controls">
                        <MotionButton class="control-btn" :while-hover="{ scale: 1.08 }" :while-press="{ scale: 0.94 }"
                            :transition="microTransition" @click="$emit('add-to-playlist')">
                            <Icon src="/assets/inbox.svg" size="sm" />
                        </MotionButton>

                        <div class="volume-control">
                            <Icon src="/assets/inventory.svg" size="sm" />
                            <div class="volume-slider">
                                <input type="range" min="0" max="100" v-model="volume" class="volume-input"
                                    @input="$emit('volume-change', $event.target.value)" />
                            </div>
                        </div>

                        <MotionButton class="control-btn" :while-hover="{ scale: 1.08 }" :while-press="{ scale: 0.94 }"
                            :transition="microTransition" @click="$emit('queue')">
                            <Icon src="/assets/list.svg" size="sm" />
                        </MotionButton>
                    </div>
                    <div class="progress-section">
                        <span class="time-display">{{ currentTime }}</span>
                        <div class="progress-container" @click="handleProgressClick">
                            <div class="progress-track">
                                <MotionDiv class="progress-fill" :animate="{ width: progress + '%' }"
                                    :transition="microTransition"></MotionDiv>
                                <MotionDiv class="progress-thumb" :animate="{ left: progress + '%' }"
                                    :transition="microTransition"></MotionDiv>
                            </div>
                        </div>
                        <span class="time-display">{{ totalTime }}</span>
                    </div>

                    <!-- 播放按钮组 -->

                    <!-- 音量和其他控制 -->

                </div>
            </div>
            </MotionDiv>

        <!-- <LyricPlayer :lyricLines="lyricLines" :currentTime="currentTimeMs" class="amll-lyric-player" -->
        <!-- style="position: absolute; left: 0px; top: 0px; width: 100%; height: 100%;" /> -->
        </MotionDiv>
    </AnimatePresence>

</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import Icon from './Icon.vue'
import MotionTransition from './MotionTransition.vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { APPLE_SPRING, INSTANT_MOTION, LINEAR_LOOP, MICRO_SPRING } from '../utils/motion.js'
import { LyricPlayer, BackgroundRender } from "@applemusic-like-lyrics/vue";
import { EplorRenderer } from '@applemusic-like-lyrics/core';
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
    },
})

const MotionDiv = motion.div
const MotionButton = motion.button

const reducedMotion = useReducedMotion()
const springTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const instantTransition = computed(() => INSTANT_MOTION)
const loopTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : LINEAR_LOOP)
const shouldShow = ref(false)

const playerEnterState = {
    opacity: 0,
    rotateX: 45,
    rotateY: -25,
    scaleX: 0.25,
    scaleY: 0.08,
    x: '-35vw',
    y: '40vh',
    z: -500,
    filter: 'blur(6px) brightness(0.4)'
}

const playerOpenState = {
    opacity: 1,
    rotateX: 0,
    rotateY: 0,
    scaleX: 1,
    scaleY: 1,
    x: 0,
    y: 0,
    z: 0,
    filter: 'blur(0px) brightness(1)'
}

const playerExitState = { ...playerEnterState }

watch(() => props.isVisible, (visible) => {
    shouldShow.value = visible
}, { immediate: true })

// 创建测试用的歌词数据（AMLL格式）
const createTestLyrics = () => {
    // 使用AMLL的标准格式
    return [
        {
            startTime: 0,
            endTime: 3000,
            words: [
                { startTime: 0, endTime: 1000, word: "申必" },
                { startTime: 1000, endTime: 2000, word: "申必比" },
                { startTime: 2000, endTime: 3000, word: "我" }
            ]
        },
        {
            startTime: 3000,
            endTime: 6000,
            words: [
                { startTime: 3000, endTime: 4000, word: "能够" },
                { startTime: 4000, endTime: 5000, word: "遇见" },
                { startTime: 5000, endTime: 6000, word: "你" }
            ]
        },
        {
            startTime: 6000,
            endTime: 9000,
            words: [
                { startTime: 6000, endTime: 7000, word: "在最美好的" },
                { startTime: 7000, endTime: 8000, word: "时光" },
                { startTime: 8000, endTime: 9000, word: "里" }
            ]
        },
        {
            startTime: 9000,
            endTime: 12000,
            words: [
                { startTime: 9000, endTime: 10000, word: "音乐" },
                { startTime: 10000, endTime: 11000, word: "播放着" },
                { startTime: 11000, endTime: 12000, word: "一点光彩" }
            ]
        },
        {
            startTime: 12000,
            endTime: 15000,
            words: [
                { startTime: 12000, endTime: 13000, word: "感觉" },
                { startTime: 13000, endTime: 14000, word: "是" },
                { startTime: 14000, endTime: 15000, word: "特别的美" }
            ]
        },
        {
            startTime: 15000,
            endTime: 18000,
            words: [
                { startTime: 15000, endTime: 16000, word: "曾尝" },
                { startTime: 16000, endTime: 17000, word: "遗失" },
                { startTime: 17000, endTime: 18000, word: "意时" }
            ]
        },
        {
            startTime: 18000,
            endTime: 21000,
            words: [
                { startTime: 18000, endTime: 19000, word: "却找到" },
                { startTime: 19000, endTime: 20000, word: "快乐" },
                { startTime: 20000, endTime: 21000, word: "点" }
            ]
        },
        {
            startTime: 21000,
            endTime: 24000,
            words: [
                { startTime: 21000, endTime: 22000, word: "联想" },
                { startTime: 22000, endTime: 23000, word: "会" },
                { startTime: 23000, endTime: 24000, word: "知" }
            ]
        },
        {
            startTime: 24000,
            endTime: 27000,
            words: [
                { startTime: 24000, endTime: 25000, word: "就是" },
                { startTime: 25000, endTime: 27000, word: "自己" }
            ]
        },
        {
            startTime: 27000,
            endTime: 30000,
            words: [
                { startTime: 27000, endTime: 28000, word: "原来是个" },
                { startTime: 28000, endTime: 30000, word: "幸运儿" }
            ]
        },
        {
            startTime: 30000,
            endTime: 33000,
            words: [
                { startTime: 30000, endTime: 31000, word: "每人" },
                { startTime: 31000, endTime: 32000, word: "命中" },
                { startTime: 32000, endTime: 33000, word: "都有责" }
            ]
        },
        {
            startTime: 33000,
            endTime: 36000,
            words: [
                { startTime: 33000, endTime: 34000, word: "每一刻" },
                { startTime: 34000, endTime: 35000, word: "都" },
                { startTime: 35000, endTime: 36000, word: "计算着" }
            ]
        }
    ]
}

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

const volume = ref(75)
const shuffleMode = ref(false)
const repeatMode = ref(false)
const lyricLines = ref(createTestLyrics())

// 转换当前时间为毫秒，供AMLL使用
const currentTimeMs = computed(() => {
    // 将时间字符串转换为毫秒
    const [minutes, seconds] = props.currentTime.split(':').map(Number)
    return (minutes * 60 + seconds) * 1000
})

// 进度条点击处理
const handleProgressClick = (event) => {
    const rect = event.currentTarget.getBoundingClientRect()
    const percent = ((event.clientX - rect.left) / rect.width) * 100
    emit('progress-change', Math.max(0, Math.min(100, percent)))
}

// 键盘事件处理
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

// 监听键盘事件
onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
    // 初始化歌词
    lyricLines.value = createTestLyrics()
    console.log('AMLL歌词已初始化:', lyricLines.value)
})

onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
})

// 监听歌曲变化，重新初始化歌词
watch(() => props.currentSong.title, () => {
    lyricLines.value = createTestLyrics()
    console.log('歌曲变化，重新加载歌词')
})
</script>

<style scoped>
.fullscreen-player {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    /* backdrop-filter: ; */
    z-index: 100;
    will-change: transform, opacity, filter;
    backface-visibility: hidden;
    width: 100vw;
    height: 100vh;
}

.player-container {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transform-origin: 50% 100%;
}

.close-button {
    position: absolute;
    top: 40px;
    right: 40px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    width: 48px;
    height: 48px;
    cursor: pointer;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
}

.player-content {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1.05fr;
    grid-template-rows: 1fr 100px;
    grid-template-areas:
        "album info"
        "controls controls";
    height: 100%;
    min-height: 600px;
}

.album-section {
    grid-area: album;
    display: grid;
    grid-template-rows: 1fr 200px;
    grid-template-columns: 1fr;
    grid-template-areas:
        "vinyl-area"
        "song-details";
    height: 100%;
    min-height: 500px;
}

.vinyl-area {
    grid-area: vinyl-area;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    width: 100%;
    height: 100%;
    align-self: center;
    justify-self: center;
    min-height: 200px;
    max-width: 500px;
    max-height: 500px;
}

.vinyl-container {
    width: calc(100vh - 400px);
    aspect-ratio: 1 / 1;
    max-width: calc(100% - 40px);
    max-height: calc(100% - 40px);
}

.vinyl-record {
    position: relative;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: linear-gradient(45deg, #1a1a1a, #2a2a2a, #1a1a1a);
    box-shadow:
        0 0 0 2px #333,
        0 20px 50px rgba(0, 0, 0, 0.5),
        inset 0 0 0 20px rgba(0, 0, 0, 0.2);
}

.album-cover {
    position: absolute;
    top: 15%;
    left: 15%;
    width: 70%;
    height: 70%;
    border-radius: 50%;
    object-fit: cover;
    box-shadow: 0 0 20px rgba(0, 0, 0, 0.5);
}

.song-info {
    grid-area: info;
    display: flex;
    flex-direction: column;
    gap: 30px;
    padding: 40px 0;
}

.song-details {
    margin-left: auto;
    margin-right: auto;
    text-align: left;
    width: 400px;
}

.song-title {
    font-size: 35px;
    font-weight: 700;
    color: white;
    margin-bottom: 16px;
    line-height: 1.1;

}

.song-artist {
    font-size: 21px;
    color: rgba(255, 255, 255, 0.8);
    margin-bottom: 8px;
    font-weight: 500;
}

.song-album {
    font-size: 16px;
    color: rgba(255, 255, 255, 0.6);
    margin-bottom: 20px;
}

.song-tags {
    display: flex;
    gap: 8px;
}

.tag {
    background: rgba(var(--primary-color), 0.2);
    color: rgba(var(--primary-color), 0.3);
    padding: 6px 12px;
    border-radius: 16px;
    font-size: 12px;
    font-weight: 500;
    border: 1px solid rgba(var(--primary-color), 0.3);
}

/* 歌词区域 */
.lyrics-container {
    flex: 1;
    overflow: hidden;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.2);
}

.player-controls {
    grid-area: controls;
    display: flex;
    flex-direction: column;
    padding: 0px 0;
    will-change: transform;
}

.progress-section {
    padding-top: 10px;
    display: flex;
    align-items: center;
    gap: 20px;
    width: 40%;
    align-self: center;
}

.time-display {
    color: rgba(255, 255, 255, 0.7);
    font-size: 14px;
    font-weight: 500;
    min-width: 45px;
}

.progress-container {
    flex: 1;
    cursor: pointer;
}

.progress-track {
    position: relative;
    height: 6px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
    overflow: hidden;
}

.progress-fill {
    height: 100%;
    background: linear-gradient(90deg, rgba(var(--primary-color), 0.3), rgba(var(--primary-color), 0.8));
    border-radius: 3px;
}

.progress-thumb {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 16px;
    height: 16px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    opacity: 0;
}

.control-btn {
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    width: 56px;
    height: 56px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.8);
}

.control-btn.active {
    background: rgba(var(--primary-color), 0.3);
    color: rgba(var(--primary-color), 0.3);
}

.additional-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 40px;
}

.volume-control {
    display: flex;
    align-items: center;
    gap: 12px;
}

.volume-slider {
    width: 100px;
}

.volume-input {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
    -webkit-appearance: none;
    appearance: none;
}

.volume-input::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    background: white;
    border-radius: 50%;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
}

</style>
