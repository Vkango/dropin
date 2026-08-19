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
                    <div class="playing-title">纯音乐 请欣赏</div>
                    <div class="playing-artist">{{ currentSong.title }} - {{ currentSong.artist }}</div>
                </div>
                <div class="controls">
                    <MotionButton class="control-button" :while-hover="{ scale: 1.08, backgroundColor: '#3a3a3a' }"
                        :while-press="{ scale: 0.92 }" :transition="microTransition" @click.stop="$emit('previous')">⏮️
                    </MotionButton>
                    <MotionButton class="control-button play-pause"
                        :while-hover="{ scale: 1.08, backgroundColor: '#3a3a3a' }" :while-press="{ scale: 0.92 }"
                        :transition="microTransition" @click.stop="$emit('toggle-play')">
                        {{ isPlaying ? '⏸️' : '▶️' }}
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="{ scale: 1.08, backgroundColor: '#3a3a3a' }"
                        :while-press="{ scale: 0.92 }" :transition="microTransition" @click.stop="$emit('next')">⏭️
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="{ scale: 1.08, backgroundColor: '#3a3a3a' }"
                        :while-press="{ scale: 0.92 }" :transition="microTransition" @click.stop="$emit('repeat')">🔁
                    </MotionButton>
                    <MotionButton class="control-button" :while-hover="{ scale: 1.08, backgroundColor: '#3a3a3a' }"
                        :while-press="{ scale: 0.92 }" :transition="microTransition" @click.stop="$emit('menu')">≡
                    </MotionButton>
                </div>
            </MotionDiv>
        </div>
    </div>
</template>

<script setup>
import { computed } from 'vue'
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
    totalTime: {
        type: String,
        default: '00:00'
    },
    progress: {
        type: Number,
        default: 0
    },
})

const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

const MotionDiv = motion.div
const MotionImg = motion.img
const MotionButton = motion.button

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
    padding: 0 15px;
    gap: 20px;
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
    gap: 18px;
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
    padding: 4px 8px;
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
    margin-bottom: 4px;
    font-weight: bold;
}

.playing-artist {
    font-size: 12px;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.controls {
    display: flex;
    align-items: center;
    gap: 16px;
}

.control-button {
    background: none;
    border: none;
    color: #ffffff;
    cursor: pointer;
    font-size: 16px;
    padding: 8px;
    border-radius: 50%;
    width: 24px;
    height: 24px;
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
    color: #888;
    min-width: 40px;
}

.progress-bar {
    flex: 1;
    height: 4px;
    background: #3a3a3a;
    border-radius: 2px;
    position: relative;
    cursor: pointer;
}

.progress-fill {
    height: 100%;
    background: #4a9eff;
    border-radius: 2px;
}

.floating-add {
    position: absolute;
    right: 30px;
    bottom: 20px;
}
</style>
