<template>
    <div class="song-list">
        <div class="list-header">
            <div class="col-play">
                <PlayIcon fill="white" color="white" size="12" />
                添加到播放列表
            </div>
            <div class="col-play">
                <ListFilter fill="white" color="white" size="12" />
                筛选
            </div>
        </div>

        <div class="songs">
            <template v-for="(song, index) in songs" :key="song.id">
                <!-- 分组标签 -->
                <div v-if="index === 0" class="group-label">&</div>
                <div v-if="index === 1" class="group-label">#</div>
                <div v-if="index === 2" class="group-label">A</div>
                <div v-if="index === 4" class="group-label">A</div>
                <MotionDiv class="song-item" :while-hover="{ backgroundColor: 'rgba(var(--surface-color), 0.5)' }"
                    :transition="microTransition" @click="$emit('song-select', song)">

                    <div class="col-info">
                        <img :src="song.cover" :alt="song.title" class="song-cover" />
                        <div class="song-details">
                            <div class="song-title">{{ song.title }}</div>
                            <div class="song-artist">{{ song.artist }}</div>
                        </div>
                    </div>
                    <div class="col-album">{{ song.album }}</div>
                    <div class="col-duration">{{ song.duration }}</div>
                </MotionDiv>
            </template>
        </div>
    </div>
</template>

<script setup>
import { defineProps, defineEmits, computed } from 'vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'
import { PlayIcon } from '@lucide/vue'
import { ListFilter } from '@lucide/vue'

const props = defineProps({
    songs: {
        type: Array,
        required: true
    }
})

const emit = defineEmits(['song-select', 'song-play'])
const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
</script>

<style scoped>
.song-list {
    border-radius: 8px;
    overflow: hidden;
}

.list-header {
    display: flex;
    gap: 20px;
    padding: 16px;
}

.group-label {
    background: rgba(var(--surface-color), 0.5);
    width: fit-content;
    padding: 6px 10px;
    font-size: 13px;
    font-weight: 700;
    border-radius: 10px;
    margin-bottom: 10px;
    margin-top: 20px;
    margin-left: 16px;
}

.song-item {
    display: grid;
    grid-template-columns: 1fr 200px 80px;
    padding: 12px 20px;
    cursor: pointer;
    border-radius: 10px;
}

.col-play {
    display: flex;
    align-items: center;
    gap: 10px;
    opacity: 0.5;
    font-size: 12px;
}

.play-btn {
    background: none;
    border: none;
    color: rgba(var(--text-color), 0.6);
    cursor: pointer;
    font-size: 14px;
}

.col-info {
    display: flex;
    align-items: center;
    gap: 12px;
}

.song-cover {
    width: 40px;
    height: 40px;
    border-radius: 4px;
    object-fit: cover;
}

.song-title {
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 4px;
}

.song-artist {
    font-size: 12px;
    color: rgba(var(--text-color), 0.6);
}

.col-album {
    display: flex;
    align-items: center;
    font-size: 13px;
    color: rgba(var(--text-color), 0.6);
}

.col-duration {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    font-size: 13px;
    color: rgba(var(--text-color), 0.6);
}
</style>
