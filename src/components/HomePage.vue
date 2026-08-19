<template>
    <div class="home-page">
        <!-- 顶部问候区域 -->
        <div class="music-banner" @click="showAlbumDetail">
            <div class="image-container">
                <MotionTransition variant="banner">
                    <img :key="currentSong.cover" class="background-image" :src="currentSong.cover"
                        referrerpolicy="no-referrer">
                </MotionTransition>
            </div>
            <div class="banner-content">
                <div class="title">DROPIN MUSIC PLAYER</div>
                <h2 class="library-title">{{ greeting }}</h2>
                <div class="description">这东西一次能吃一袋我丢
                </div>
            </div>
        </div>
        <div class="recently-played">
            <div class="section-header">
                <h2 class="section-title">最近播放</h2>
                <MotionButton class="see-all-btn" :while-hover="{ opacity: 0.8 }"
                    :transition="microTransition" @click="$emit('navigate', 'library')">查看全部</MotionButton>
            </div>
            <div class="recent-grid">
                <MotionDiv v-for="item in recentlyPlayed" :key="item.id" class="recent-item" initial="rest"
                    while-hover="hover" :variants="cardVariants" @click="$emit('song-play', item)">
                    <div class="recent-cover">
                        <MotionTransition variant="cover" mode="out-in">
                            <MotionImg :key="item.cover" :src="item.cover" :alt="item.title"
                                :variants="imageVariants" />
                        </MotionTransition>
                        <MotionDiv class="play-overlay" :variants="overlayVariants">
                            <Icon src="/assets/play.svg" size="md" />
                        </MotionDiv>
                    </div>
                    <div class="recent-info">
                        <h3 class="recent-title">{{ item.title }}</h3>
                        <p class="recent-artist">{{ item.artist }}</p>
                    </div>
                </MotionDiv>
            </div>
        </div>

        <!-- 推荐播放列表 -->
        <div class="recommended-playlists">
            <div class="section-header">
                <h2 class="section-title">为您推荐</h2>
                <MotionButton class="see-all-btn" :while-hover="{ opacity: 0.8 }"
                    :transition="microTransition" @click="$emit('navigate', 'playlists')">查看全部</MotionButton>
            </div>
            <div class="playlist-grid">
                <MotionDiv v-for="playlist in recommendedPlaylists" :key="playlist.id" class="playlist-item"
                    initial="rest" while-hover="hover" :variants="cardVariants"
                    @click="$emit('playlist-play', playlist)">
                    <div class="playlist-cover">
                        <MotionImg :src="playlist.cover" :alt="playlist.name" :variants="imageVariants" />
                        <MotionDiv class="play-overlay" :variants="overlayVariants">
                            <Icon src="/assets/play.svg" size="lg" />
                        </MotionDiv>
                    </div>
                    <div class="playlist-info">
                        <h3 class="playlist-title">{{ playlist.name }}</h3>
                        <p class="playlist-desc">{{ playlist.description }}</p>
                    </div>
                </MotionDiv>
            </div>
        </div>
    </div>
</template>

<script setup>
import { computed, onMounted, ref, inject } from 'vue'
import Icon from './Icon.vue'
import MotionTransition from './MotionTransition.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
    recentlyPlayed: {
        type: Array,
        default: () => []
    },
    recommendedPlaylists: {
        type: Array,
        default: () => []
    }
})

const emit = defineEmits(['song-play', 'playlist-play', 'navigate'])

const MotionDiv = motion.div
const MotionImg = motion.img
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const cardVariants = {
    rest: { y: 0 },
    hover: { y: -4 }
}
const imageVariants = {
    rest: { scale: 1 },
    hover: { scale: 1.05 }
}
const overlayVariants = {
    rest: { opacity: 0 },
    hover: { opacity: 1 }
}

const currentTime = ref('')
const currentSong = inject('currentSong')
const greeting = computed(() => {
    const hour = new Date().getHours()
    if (hour < 12) return '早上好'
    if (hour < 18) return '下午好'
    return '晚上好'
})

const updateTime = () => {
    const now = new Date()
    currentTime.value = now.toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
    })
}

onMounted(() => {
    updateTime()
    setInterval(updateTime, 1000)
})
</script>

<style scoped>
.home-page {
    padding: 20px 50px;
    height: 100%;
    overflow-y: auto;
}

/* 问候区域 */
.greeting-section {
    margin-bottom: 40px;
}

.greeting-text {
    font-size: 48px;
    font-weight: 700;
    margin-bottom: 8px;
    background: linear-gradient(135deg, rgba(var(--primary-color), 0.3), rgb(var(--secondary-color)));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

.current-time {
    font-size: 16px;
    color: rgba(var(--text-color), 0.7);
    font-weight: 500;
}

/* 区块标题 */
.section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
}

.section-title {
    font-size: 24px;
    font-weight: 600;
    color: rgb(var(--text-color));
}

.see-all-btn {
    background: transparent;
    border: none;
    color: rgba(var(--primary-color), 0.3);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    padding: 8px 0;
}

/* 最近播放网格 */
.recently-played {
    margin-bottom: 50px;
}

.recent-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 20px;
}

.recent-item {
    cursor: pointer;
}

.recent-cover {
    position: relative;
    aspect-ratio: 1;
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.recent-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.play-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
}

.recent-info {
    text-align: left;
}

.recent-title {
    font-size: 16px;
    font-weight: 600;
    color: rgb(var(--text-color));
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.recent-artist {
    font-size: 14px;
    color: rgba(var(--text-color), 0.7);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* 推荐播放列表 */
.playlist-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 24px;
}

.playlist-item {
    cursor: pointer;
}

.playlist-cover {
    position: relative;
    aspect-ratio: 1;
    border-radius: 16px;
    overflow: hidden;
    margin-bottom: 16px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.15);
}

.playlist-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.playlist-info {
    text-align: left;
}

.playlist-title {
    font-size: 18px;
    font-weight: 600;
    color: rgb(var(--text-color));
    margin-bottom: 6px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.playlist-desc {
    font-size: 14px;
    color: rgba(var(--text-color), 0.7);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

/* 滚动条样式 */
.home-page::-webkit-scrollbar {
    width: 4px;
}

.home-page::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
}

.home-page::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.home-page::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
}

/* 响应式设计 */
@media (max-width: 768px) {
    .home-page {
        padding: 20px 24px;
    }

    .greeting-text {
        font-size: 36px;
    }

    .recent-grid,
    .playlist-grid {
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 16px;
    }

    .section-title {
        font-size: 20px;
    }
}
</style>
