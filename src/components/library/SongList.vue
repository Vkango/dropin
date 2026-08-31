<template>
    <div class="song-list">
        <div v-if="showHeader" class="list-header">
            <MotionButton v-if="primaryActionClickable" class="col-play list-header-action" type="button"
                :while-hover="{ y: -1 }" :while-press="{ scale: 0.96 }" :transition="microTransition"
                @click="$emit('primary-action')">
                <ListMusic size="13" />
                {{ resolvedPrimaryActionLabel }}
            </MotionButton>
            <div v-else class="col-play">
                <PlayIcon fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))" size="12" />
                {{ resolvedPrimaryActionLabel }}
            </div>
            <MotionButton v-if="showPlayAll" class="col-play list-header-action" type="button" :while-hover="{ y: -1 }"
                :while-press="{ scale: 0.96 }" :transition="microTransition" @click="$emit('play-all')">
                <PlayIcon fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))" size="12" />
                {{ t('library.playAll') }}
            </MotionButton>
            <MotionButton class="col-play list-header-action">
                <ListFilter fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))"
                    size="12" />
                {{ t('library.filter') }}
            </MotionButton>
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
import { INSTANT_MOTION, MICRO_SPRING } from '@/utils/motion.js'
import { groupByInitial } from '@/utils/alphabet.js'
import GroupLabel from './GroupLabel.vue'
import { ListFilter, ListMusic, PlayIcon } from '@lucide/vue'
import { useI18n } from '@/i18n/index.js'

const { t } = useI18n()

const props = defineProps({
    songs: {
        type: Array,
        required: true
    },
    showHeader: {
        type: Boolean,
        default: true
    },
    primaryActionLabel: {
        type: String,
        default: ''
    },
    primaryActionClickable: {
        type: Boolean,
        default: false
    },
    showPlayAll: {
        type: Boolean,
        default: false
    }
})

const emit = defineEmits(['primary-action', 'play-all', 'song-select', 'song-play', 'group-label-click'])
const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const groupedSongs = computed(() => groupByInitial(props.songs, (song) => song.title))
const resolvedPrimaryActionLabel = computed(() => props.primaryActionLabel || t('library.addToPlaylist'))
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

.list-header-action {
    margin: 0;
    border: 0;
    padding: 0;
    color: inherit;
    background: transparent;
    font: inherit;
    cursor: pointer;
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
