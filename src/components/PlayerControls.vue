<template>
    <div class="player-controls">
        <div class="current-playing">
            <MotionDiv class="mini-cover-container" :while-hover="{ scale: 1.05 }" :while-press="{ scale: 0.96 }"
                :transition="microTransition" @click="$emit('expand-player')">
                <MotionTransition variant="miniCover" mode="out-in">
                    <MotionImg :key="currentSong.cover" :src="currentSong.cover" :alt="currentSong.title"
                        :while-hover="{ scale: 1.05 }" :transition="microTransition" class="mini-cover" />
                </MotionTransition>
            </MotionDiv>
            <MotionDiv class="song-info-section" :while-hover="{ backgroundColor: 'rgba(255, 255, 255, 0.05)' }"
                :transition="microTransition" @click="$emit('expand-player')">
                <div class="playing-info">
                    <div class="playing-title" :title="miniLyricText">{{ miniLyricText }}</div>
                    <div class="playing-artist">{{ currentSong.title }} - {{ currentSong.artist }}</div>
                </div>
                <div class="controls">
                    <MotionButton class="control-button" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" aria-label="上一首"
                        @click.stop="$emit('previous')">
                        <SkipBack :size="17" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button play-pause"
                        :while-hover="buttonHover" :while-press="buttonPress" :transition="microTransition"
                        :aria-label="isPlaying ? '暂停' : '播放'" @click.stop="$emit('toggle-play')">
                        <Pause v-if="isPlaying" :size="17" :stroke-width="1.8" />
                        <Play v-else :size="17" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" aria-label="下一首"
                        @click.stop="$emit('next')">
                        <SkipForward :size="17" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" aria-label="循环播放"
                        @click.stop="$emit('repeat')">
                        <Repeat2 :size="17" :stroke-width="1.8" />
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="buttonHover"
                        :while-press="buttonPress" :transition="microTransition" aria-label="更多操作"
                        @click.stop="$emit('menu')">
                        <Menu :size="17" :stroke-width="1.8" />
                    </MotionButton>
                </div>
            </MotionDiv>
        </div>
    </div>
</template>

<script setup>
import { computed } from 'vue'
import { motion, useReducedMotion } from 'motion-v'
import { Menu, Pause, Play, Repeat2, SkipBack, SkipForward } from '@lucide/vue'
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
    },
})

const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

const MotionDiv = motion.div
const MotionImg = motion.img
const MotionButton = motion.button

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

const buttonHover = { scale: 1.08, color: 'rgb(var(--primary-color))', backgroundColor: 'rgba(var(--primary-color), 0.14)' }
const buttonPress = { scale: 0.92 }

const emit = defineEmits([
    'previous',
    'toggle-play',
    'next',
    'repeat',
    'menu',
    'progress-change',
    'add',
    'expand-player'
])

const handleProgressClick = (event) => {
    const rect = event.currentTarget.getBoundingClientRect()
    const percent = (event.clientX - rect.left) / rect.width * 100
    emit('progress-change', percent)
}
</script>

<style scoped>
.player-controls {
    display: flex;
    align-items: center;
    padding: 0 18px;
    gap: 22px;
    width: fit-content;
    max-width: 100%;
    height: 100%;
    min-width: 0;
    overflow: hidden;
    backdrop-filter: blur(20px);
}

.current-playing {
    display: flex;
    align-items: center;
    gap: 20px;
    flex: 0 1 auto;
    min-width: 0;
    max-width: 100%;
}

.mini-cover-container {
    position: relative;
    width: 70px;
    height: 70px;
    border-radius: 4px;
    overflow: hidden;
    cursor: pointer;
}

.song-info-section {
    min-width: 0;
    max-width: calc(100vw - 123px);
    cursor: pointer;
    border-radius: 4px;
    padding: 7px 10px 8px;
}

.mini-cover {
    width: 100%;
    height: 100%;
    border-radius: 4px;
    object-fit: cover;
}

.playing-title {
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 5px;
    font-weight: bold;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.playing-artist {
    font-size: 12px;
    color: rgba(var(--text-color), 0.6);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.controls {
    display: flex;
    align-items: center;
    gap: 10px;
}

.control-button {
    background: none;
    border: none;
    color: rgb(var(--text-color));
    cursor: pointer;
    padding: 7px;
    border-radius: 50%;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.progress-section {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    max-width: 300px;
}

.time {
    font-size: 12px;
    color: rgba(var(--text-color), 0.6);
    min-width: 40px;
}

.progress-bar {
    flex: 1;
    height: 4px;
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
    position: relative;
    cursor: pointer;
}

.progress-fill {
    height: 100%;
    background: rgb(var(--primary-color));
    border-radius: 2px;
}

.floating-add {
    position: absolute;
    right: 30px;
    bottom: 20px;
}
</style>
