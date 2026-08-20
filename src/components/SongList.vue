<template>
    <div class="song-list">
        <div class="list-header">
            <div class="col-play">
                <PlayIcon fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))" size="12" />
                添加到播放列表
            </div>
            <div class="col-play">
                <ListFilter fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))"
                    size="12" />
                筛选
            </div>
        </div>

        <div class="songs">
            <template v-for="group in groupedSongs" :key="group.initial">
                <GroupLabel :label="group.initial" @click="$emit('group-label-click', group.initial)" />
                <MotionDiv v-for="song in group.items" :key="song.id" class="song-item"
                    :while-hover="{ backgroundColor: 'rgba(var(--surface-color), 0.5)' }" :transition="microTransition"
                    @click="$emit('song-play', song)">
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
import { groupByInitial } from '../utils/alphabet.js'
import GroupLabel from './GroupLabel.vue'
import { PlayIcon } from '@lucide/vue'
import { ListFilter } from '@lucide/vue'

const props = defineProps({
    songs: {
        type: Array,
        required: true
    }
})

const emit = defineEmits(['song-select', 'song-play', 'group-label-click'])
const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const groupedSongs = computed(() => groupByInitial(props.songs, (song) => song.title))
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
