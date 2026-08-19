<template>
    <div class="song-list">
        <!-- <div class="list-header">
            <div class="col-play">▶️</div>
            <div class="col-title"></div>
            <div class="col-equals">=</div>
            <div class="col-artist">音源</div>
        </div> -->

        <div class="songs">
            <template v-for="(song, index) in songs" :key="song.id">
                <!-- 分组标签 -->
                <div v-if="index === 0" class="group-label">&</div>
                <div v-if="index === 1" class="group-label">#</div>
                <div v-if="index === 2" class="group-label">A</div>
                <div v-if="index === 4" class="group-label">A</div>
                <MotionDiv class="song-item" :while-hover="{ backgroundColor: '#2a2a2a' }"
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
    background: #242424;
    border-radius: 8px;
    overflow: hidden;
}

.list-header {
    display: grid;
    grid-template-columns: 60px 1fr 30px 200px;
    padding: 16px 20px;
    background: #2a2a2a;
    font-weight: 600;
    border-bottom: 1px solid #3a3a3a;
}

.group-label {
    background: #2a2a2a;
    padding: 12px 20px;
    font-size: 18px;
    font-weight: 700;
    border-bottom: 1px solid #3a3a3a;
}

.song-item {
    display: grid;
    grid-template-columns: 1fr 200px 80px;
    padding: 12px 20px;
    border-bottom: 1px solid #333;
    cursor: pointer;
}

.col-play {
    display: flex;
    align-items: center;
}

.play-btn {
    background: none;
    border: none;
    color: #888;
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
    font-weight: 500;
    margin-bottom: 4px;
}

.song-artist {
    font-size: 12px;
    color: #888;
}

.col-album {
    display: flex;
    align-items: center;
    font-size: 13px;
    color: #888;
}

.col-duration {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    font-size: 13px;
    color: #888;
}
</style>
