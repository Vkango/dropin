<template>
    <main class="main-content">
        <!-- Banner 区域 -->
        <div class="music-banner" @click="showAlbumDetail">
            <div class="image-container">
                <MotionTransition variant="banner">
                    <img :key="bannerImage" class="background-image" :src="bannerImage" referrerpolicy="no-referrer">
                </MotionTransition>
            </div>
            <div class="banner-content">
                <div class="title">DROPIN MUSIC PLAYER</div>
                <h2 class="library-title">库</h2>
                <div class="description">{{ musicLibrary.totalSongs }} songs • {{ musicLibrary.totalDuration }}
                </div>
            </div>
            <div class="controls-row">
                <MotionButton v-for="control in headerControls" :key="control.id" class="control-btn"
                    :while-hover="{ y: -1, backgroundColor: 'rgba(74, 74, 74, 0.9)', boxShadow: '0 4px 15px rgba(0, 0, 0, 0.3)' }"
                    :while-press="{ scale: 0.96 }" :transition="microTransition"
                    :class="{ selected: control.selected }" @click.stop="$emit('header-control-click', control)">

                    <Icon :src="getIconPath(control.icon)" size="xs" />
                    <span v-if="control.selected">{{ control.label }}</span>

                </MotionButton>
            </div>
        </div>
        <div class="song-list-container">
            <SongList :songs="musicLibrary.songs" @song-select="$emit('song-select', $event)"
                @song-play="$emit('song-play', $event)" />
        </div>

        <!-- 专辑详情卡片 -->
        <AlbumDetailCard :visible="albumDetailVisible" :album="currentAlbumDetail" @close="hideAlbumDetail"
            @play-all="handlePlayAll" @track-select="handleTrackSelect" @track-play="handleTrackPlay" />
    </main>
</template>

<script setup>
import { defineProps, defineEmits, computed, inject, ref } from 'vue'
import SongList from './SongList.vue'
import Icon from './Icon.vue'
import AlbumDetailCard from './AlbumDetailCard.vue'
import MotionTransition from './MotionTransition.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const currentSong = inject('currentSong')

const props = defineProps({
    musicLibrary: {
        type: Object,
        required: true
    },
    headerControls: {
        type: Array,
        default: () => [
            { id: 'all', icon: 'library.svg', label: '全部', selected: true },
            { id: 'system', icon: 'folder.svg', label: '系统音乐目录', selected: false },
            { id: 'local', icon: 'sys_music.svg', label: '本地存储', selected: false },
            { id: 'import', icon: 'ext.svg', label: '外部导入', selected: false },
            { id: 'network', icon: 'cloud.svg', label: '网络', selected: false }
        ]
    }
})

const emit = defineEmits(['header-control-click', 'song-select', 'song-play'])

const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

// 专辑详情状态
const albumDetailVisible = ref(false)
const currentAlbumDetail = ref(null)

const bannerImage = computed(() => {
    return currentSong.value.cover
})

const getIconPath = (iconName) => {
    return `/assets/${iconName}`
}

// 显示专辑详情
const showAlbumDetail = () => {
    currentAlbumDetail.value = {
        title: "はたらく細胞BLACK (Original Soundtrack)",
        artist: "菅野祐悟",
        coverUrl: currentSong.value.cover,
        releaseYear: "2024",
        totalTracks: props.musicLibrary.totalSongs,
        duration: props.musicLibrary.totalDuration,
        genres: ["Electronic", "Ambient", "Pop"],
        description: "A curated collection of modern electronic and ambient music pieces featuring artists from around the world.",
        tracks: props.musicLibrary.songs.map((song, index) => ({
            number: index + 1,
            title: song.title,
            artist: song.artist,
            duration: song.duration,
            url: song.url
        }))
    }
    albumDetailVisible.value = true
}

// 隐藏专辑详情
const hideAlbumDetail = () => {
    albumDetailVisible.value = false
}

// 播放专辑所有歌曲
const handlePlayAll = () => {
    emit('song-play', props.musicLibrary.songs[0])
}

// 选择曲目
const handleTrackSelect = (track) => {
    const song = props.musicLibrary.songs.find(s => s.title === track.title)
    if (song) {
        emit('song-select', song)
    }
}

// 播放指定曲目
const handleTrackPlay = (track) => {
    const song = props.musicLibrary.songs.find(s => s.title === track.title)
    if (song) {
        emit('song-play', song)
    }
}
</script>

<style scoped>
.main-content {
    grid-area: main;
    background: #1e1e1e;
    padding: 20px 50px;
    overflow-y: auto;
    overflow-x: hidden
}


.controls-row {
    margin-top: 15px;
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    position: absolute;
    right: 40px;
    width: fit-content;
    top: 20px;
}

.control-btn {
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #ffffff;
    padding: 8px 12px;
    font-size: 12px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 10px;
    opacity: 0.5;
}

.control-btn.selected {
    font-weight: bold;
    color: rgb(var(--primary-color));
    opacity: 1;
}

/* 库信息 */
.library-info {
    margin-bottom: 24px;
}

.library-stats {
    display: flex;
    align-items: center;
    gap: 16px;
    color: #888;
    font-size: 14px;
}

.stat-icon {
    font-size: 16px;
}

/* 歌曲列表 */
.song-list-container {
    padding: 0;
    margin-top: 60px;
}
</style>
