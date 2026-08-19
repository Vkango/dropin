<template>
    <footer class="player-controls" :class="animationClass">
        <div class="current-playing">
            <div class="mini-cover-container" @click="$emit('expand-player')">
                <Transition name="mini-cover" mode="out-in">
                    <img :key="currentSong.cover" :src="currentSong.cover" :alt="currentSong.title"
                        class="mini-cover" />
                </Transition>
            </div>
            <div class="song-info-section" @click="$emit('expand-player')">
                <div class="playing-info">
                    <div class="playing-title">纯音乐 请欣赏</div>
                    <div class="playing-artist">{{ currentSong.title }} - {{ currentSong.artist }}</div>
                </div>
                <div class="controls">
                    <button class="control-button" @click.stop="$emit('previous')">⏮️</button>
                    <button class="control-button play-pause" @click.stop="$emit('toggle-play')">
                        {{ isPlaying ? '⏸️' : '▶️' }}
                    </button>
                    <button class="control-button" @click.stop="$emit('next')">⏭️</button>
                    <button class="control-button" @click.stop="$emit('repeat')">🔁</button>
                    <button class="control-button" @click.stop="$emit('menu')">≡</button>
                </div>
            </div>

        </div>
    </footer>
</template>

<script setup>
import { defineProps, defineEmits, computed } from 'vue'

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
    animationState: {
        type: String,
        default: 'idle' // 'idle', 'expanding', 'collapsing'
    },
    isTransitioning: {
        type: Boolean,
        default: false
    }
})

// 计算动画类名
const animationClass = computed(() => {
    if (props.animationState === 'expanding') {
        return 'player-expanding'
    } else if (props.animationState === 'collapsing') {
        return 'player-appearing'
    }
    return ''
})

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
    background: rgba(var(--secondary-color), 0.2);
    /* border-top: 1px solid rgba(var(--secondary-color), 0.5); */
    display: flex;
    backdrop-filter: blur(20px);
    align-items: center;
    padding: 0 15px;
    gap: 20px;
    position: absolute;
    bottom: 20px;
    left: 20px;
    border-radius: 5px;
    height: 90px;
    transition: all 0.3s ease;
    transform-origin: 0% 100%;
    /* 左下角为变换原点，与FullscreenPlayer一致 */
    will-change: transform, opacity, filter;
    backface-visibility: hidden;
}

/* PlayerControls 展开动画 - 放大并淡出 */
.player-controls.player-expanding {
    animation: player-expand-fadeout 0.8s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
    pointer-events: none;
    /* 动画期间禁用交互 */
}

/* PlayerControls 出现动画 - 从小到正常大小 */
.player-controls.player-appearing {
    animation: player-appear-fadein 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
    pointer-events: none;
    /* 动画期间禁用交互 */
}

/* PlayerControls 展开动画关键帧 */
@keyframes player-expand-fadeout {
    0% {
        transform: scale(1) translate(0, 0);
        opacity: 1;
        filter: blur(0px);
    }

    30% {
        transform: scale(1.2) translate(-5px, -8px);
        opacity: 0.8;
        filter: blur(1px);
    }

    60% {
        transform: scale(2.5) translate(-15px, -25px);
        opacity: 0.4;
        filter: blur(3px);
    }

    100% {
        transform: scale(4) translate(-25px, -40px);
        opacity: 0;
        filter: blur(6px);
    }
}

/* PlayerControls 出现动画关键帧 */
@keyframes player-appear-fadein {
    0% {
        transform: scale(0.3) translate(-15px, -25px);
        opacity: 0;
        filter: blur(8px);
    }

    30% {
        transform: scale(0.6) translate(-8px, -12px);
        opacity: 0.4;
        filter: blur(4px);
    }

    70% {
        transform: scale(0.9) translate(-2px, -3px);
        opacity: 0.8;
        filter: blur(1px);
    }

    100% {
        transform: scale(1) translate(0, 0);
        opacity: 1;
        filter: blur(0px);
    }
}

.current-playing {
    display: flex;
    align-items: center;
    gap: 18px;
    flex: 1;
}

.mini-cover-container {
    position: relative;
    width: 70px;
    height: 70px;
    border-radius: 4px;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s ease;
}

.mini-cover-container:hover {
    transform: scale(1.05);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.song-info-section {
    cursor: pointer;
    transition: all 0.2s ease;
    border-radius: 4px;
    padding: 4px 8px;
}

.song-info-section:hover {
    background: rgba(255, 255, 255, 0.05);
}

.mini-cover {
    width: 100%;
    height: 100%;
    border-radius: 4px;
    object-fit: cover;
    transition: transform 0.3s ease;
}

.mini-cover:hover {
    transform: scale(1.05);
}

/* Mini-cover 过渡动画 */
.mini-cover-enter-active,
.mini-cover-leave-active {
    transition: all 0.1s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    position: absolute;
    top: 0;
    left: 0;
}

.mini-cover-enter-from {
    opacity: 0;
    transform: scale(0.8) rotate(-5deg);
    filter: blur(3px);
}

.mini-cover-leave-to {
    opacity: 0;
    transform: scale(1.2) rotate(5deg);
    filter: blur(3px);
}

.mini-cover-enter-to,
.mini-cover-leave-from {
    opacity: 1;
    transform: scale(1) rotate(0deg);
    filter: blur(0);
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

.control-button:hover {
    background: #3a3a3a;
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
    transition: width 0.1s;
}

.floating-add {
    position: absolute;
    right: 30px;
    bottom: 20px;
}
</style>