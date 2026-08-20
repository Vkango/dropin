<template>
    <MotionTransition variant="modal" appear>
        <div v-if="visible" class="modal-overlay" @click="handleOverlayClick">
            <MotionTransition variant="card" appear>
                <div v-if="visible" class="album-card" @click.stop>
                    <MotionButton class="close-button" :while-hover="{ scale: 1.08 }" :while-press="{ scale: 0.94 }"
                        :transition="microTransition" @click="$emit('close')">
                        <Icon src="/assets/close.svg" size="sm" />
                    </MotionButton>
                    <div class="album-cover">
                        <MotionTransition variant="albumCover" mode="out-in">
                            <MotionImg :key="album.coverUrl" :src="album.coverUrl" :alt="album.title"
                                :while-hover="{ scale: 1.05 }" :transition="microTransition" />
                        </MotionTransition>
                    </div>
                    <div class="album-info">
                        <div class="album-header">
                            <div class="album-type">{{ album.type || '音乐专辑' }} · {{ album.tracks.length }}首歌</div>
                            <h2 class="album-title">{{ album.title }}</h2>
                            <div class="album-artist">
                                <Icon src="/assets/user.svg" size="sm" />
                                <span>{{ album.artist }}</span>
                            </div>
                        </div>

                        <!-- 操作按钮 -->
                        <div class="action-buttons">
                            <MotionButton class="play-all-btn"
                                :while-hover="{ y: -1, backgroundColor: 'rgba(var(--primary-color), 0.3)' }"
                                :while-press="{ scale: 0.96 }" :transition="microTransition" @click="$emit('play-all')">
                                <Icon src="/assets/play.svg" size="sm" />
                                <span>Play all</span>
                            </MotionButton>
                        </div>

                        <!-- 歌曲列表 -->
                        <div class="track-list">
                            <MotionDiv v-for="(track, index) in album.tracks" :key="track.id" class="track-item"
                                :while-hover="{ backgroundColor: 'rgba(var(--primary-color), 0.08)' }"
                                :transition="microTransition" @click="$emit('track-play', track)">
                                <div class="track-number">{{ index + 1 }}</div>
                                <div class="track-info">
                                    <div class="track-title">{{ track.title }}</div>
                                    <div class="track-artist">{{ track.artist || album.artist }}</div>
                                </div>
                                <div class="track-duration">{{ track.duration }}</div>
                            </MotionDiv>
                        </div>
                    </div>
                </div>
            </MotionTransition>
        </div>
    </MotionTransition>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue'
import Icon from './Icon.vue'
import MotionTransition from './MotionTransition.vue'
import { motion, useReducedMotion } from 'motion-v'
import { computed } from 'vue'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const MotionButton = motion.button
const MotionDiv = motion.div
const MotionImg = motion.img
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

const props = defineProps({
    visible: {
        type: Boolean,
        default: false
    },
    album: {
        type: Object,
        default: () => ({
            title: '',
            artist: '',
            cover: '',
            type: '音乐专辑',
            songCount: 0,
            tracks: []
        })
    }
})

const emit = defineEmits(['close', 'play-all', 'track-select', 'track-play'])

const handleOverlayClick = () => {
    emit('close')
}
</script>

<style scoped>
/* 背景遮罩 - 简单的淡入淡出，无3D效果 */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 20px;
    overflow: hidden;
}

/* 卡片本体 - 3D动画效果 */
.album-card {
    background: rgba(var(--global-color), 0.8);
    border: 1px solid rgba(var(--outline-color), 0.2);
    border-radius: 5px;
    box-shadow:
        0 24px 48px rgba(0, 0, 0, 0.4),
        0 8px 24px rgba(0, 0, 0, 0.2);
    height: min(600px, calc(100vh - 40px));
    max-height: calc(100vh - 40px);
    min-height: 0;
    width: 1100px;
    display: grid;
    grid-template-columns: 600px 1fr;
    gap: 30px;
    /* padding: 30px; */
    position: relative;
    overflow: hidden;
    backdrop-filter: blur(20px);
}

.close-button {
    position: absolute;
    top: 16px;
    right: 16px;
    background: transparent;
    border: none;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 10;
}

.album-cover {
    position: relative;
    width: 600px;
    height: 600px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.album-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.album-info {
    display: flex;
    flex-direction: column;
    gap: 24px;
    overflow-y: auto;
    min-height: 0;
    height: 100%;
    padding: 20px 8px 20px 0;
    margin: 0;
}

.album-header {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.album-type {
    font-size: 12px;
    color: rgba(var(--text-color), 0.7);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.album-title {
    font-size: 24px;
    font-weight: bold;
    color: rgb(var(--text-color));
    margin: 0;
    line-height: 1.2;
}

.album-artist {
    display: flex;
    align-items: center;
    gap: 8px;
    color: rgba(var(--text-color), 0.8);
    font-size: 14px;
}

.action-buttons {
    display: flex;
    gap: 12px;
}

.play-all-btn {
    background: rgb(var(--primary-color), 0.1);
    color: rgba(var(--primary-color), 0.3);
    border: none;
    border-radius: 8px;
    padding: 12px 20px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
}

.track-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.track-item {
    display: grid;
    grid-template-columns: 24px 1fr auto;
    gap: 12px;
    align-items: center;
    padding: 8px 12px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
}

.track-number {
    color: rgba(var(--text-color), 0.5);
    font-size: 12px;
    text-align: center;
}

.track-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.track-title {
    color: rgb(var(--text-color));
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.track-artist {
    color: rgba(var(--text-color), 0.6);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.track-duration {
    color: rgba(var(--text-color), 0.5);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
}

/* 响应式设计 */
@media (max-width: 768px) {
    .album-card {
        grid-template-columns: 1fr;
        grid-template-rows: auto 1fr;
        max-width: 95vw;
        height: min(90vh, calc(100vh - 40px));
        padding: 20px;
        gap: 20px;
    }

    .album-cover {
        height: 250px;
        justify-self: center;
        max-width: 250px;
    }

    .album-info {
        height: 100%;
        padding: 0 8px 0 0;
    }

    .album-title {
        font-size: 20px;
    }
}
</style>
