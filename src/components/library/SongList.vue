<template>
    <div class="song-list">
        <div v-if="showHeader" class="list-header">
            <MotionButton v-if="primaryActionClickable" class="col-play list-header-action" type="button"
                :disabled="primaryActionDisabled"
                :while-hover="{ y: -1 }" :while-press="{ scale: 0.96 }" :transition="microTransition"
                @click="$emit('primary-action')">
                <slot name="primary-icon">
                    <ListMusic size="13" />
                </slot>
                {{ resolvedPrimaryActionLabel }}
            </MotionButton>
            <div v-else class="col-play">
                <PlayIcon fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))" size="12" />
                {{ resolvedPrimaryActionLabel }}
            </div>
            <MotionButton v-if="showPlayAll" class="col-play list-header-action" type="button" :while-hover="{ y: -1 }"
                :disabled="!songs.length" :while-press="{ scale: 0.96 }" :transition="microTransition" @click="$emit('play-all')">
                <PlayIcon fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))" size="12" />
                {{ t('library.playAll') }}
            </MotionButton>
            <MotionButton class="col-play list-header-action" @click="$emit('filter-click')">
                <ListFilter fill="rgb(var(--global-inverse-color))" color="rgb(var(--global-inverse-color))"
                    size="12" />
                {{ t('library.filter') }}
            </MotionButton>
            <div v-if="$slots['header-extra']" class="col-extra">
                <slot name="header-extra" />
            </div>
            <div v-if="$slots['header-search']" class="col-search">
                <slot name="header-search" />
            </div>
        </div>

        <div class="songs">
            <template v-for="group in groupedSongs" :key="group.initial">
                <GroupLabel :label="group.initial" @click="$emit('group-label-click', group.initial)" />
                <VirtualList :items="group.items" :item-height="64" v-slot="{ item: song }">
                <MotionDiv class="song-item"
                    :while-hover="{ backgroundColor: 'rgba(var(--surface-color), 0.5)' }" :transition="microTransition"
                    @click="$emit('song-play', song)"
                    @contextmenu.prevent.stop="$emit('song-context-menu', { song, x: $event.clientX, y: $event.clientY, source: contextSource })">
                    <div class="col-info">
                        <img :src="song.cover" :alt="song.title" class="song-cover" loading="lazy" decoding="async" />
                        <div class="song-details">
                            <div class="song-title">{{ song.title }}</div>
                            <div class="song-artist">
                                <span v-if="songBadgeOf" class="song-tag-badge">{{ songBadgeOf(song) }}</span>
                                <span>{{ song.artist }}</span>
                            </div>
                        </div>
                    </div>
                    <div class="col-album">{{ song.album }}</div>
                    <div class="col-duration">{{ song.duration }}</div>
                </MotionDiv>
                </VirtualList>
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
import VirtualList from '@/components/ui/VirtualList.vue'
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
    primaryActionDisabled: {
        type: Boolean,
        default: false
    },
    showPlayAll: {
        type: Boolean,
        default: false
    },
    groupOf: {
        type: Function,
        default: null
    },
    sortValueOf: {
        type: Function,
        default: null
    },
    // 右键菜单来源描述（如 { type: 'playlist', id, name }），透传给菜单处理方
    contextSource: {
        type: Object,
        default: null
    },
    // 歌曲行的徽标文案（如 My Tag 页的 “Energy 30”），显示在艺术家之前
    songBadgeOf: {
        type: Function,
        default: null
    }
})

const emit = defineEmits(['primary-action', 'play-all', 'song-select', 'song-play', 'song-context-menu', 'group-label-click', 'filter-click'])
const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

const groupByValue = (items, getKey) => {
    const groups = new Map()
    items.forEach((item) => {
        const key = String(getKey(item) ?? '').trim() || '–'
        if (!groups.has(key)) groups.set(key, [])
        groups.get(key).push(item)
    })
    const keys = [...groups.keys()]
    if (keys.every((key) => /^\d+/.test(key))) {
        keys.sort((left, right) => (parseInt(left, 10) || 0) - (parseInt(right, 10) || 0))
    } else {
        keys.sort((left, right) => left.localeCompare(right))
    }
    const sortKeyOf = (item) => {
        const value = props.sortValueOf ? props.sortValueOf(item) : null
        if (value === null || value === undefined || value === '') return null
        const number = typeof value === 'number' ? value : Number(value)
        return Number.isFinite(number) ? number : null
    }
    const groupsArray = keys.map((key) => ({ initial: key, items: groups.get(key) }))
    if (props.groupOf && props.sortValueOf) {
        for (const group of groupsArray) {
            group.items.sort((left, right) => {
                const leftKey = sortKeyOf(left)
                const rightKey = sortKeyOf(right)
                if (leftKey === null && rightKey === null) return 0
                if (leftKey === null) return 1
                if (rightKey === null) return -1
                return leftKey - rightKey
            })
        }
    }
    return groupsArray
}

const groupedSongs = computed(() => props.groupOf
    ? groupByValue(props.songs, props.groupOf)
    : groupByInitial(props.songs, (song) => song.title))

const resolvedPrimaryActionLabel = computed(() => props.primaryActionLabel || t('library.addToPlaylist'))
</script>

<style scoped>
.song-list {
    border-radius: 8px;
    overflow: hidden;
}

.list-header {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    padding: 16px;
}

.col-search {
    display: flex;
    flex: 1;
    min-width: 160px;
    max-width: 320px;
    margin-left: auto;
    align-items: center;
}

.col-extra {
    display: flex;
    align-items: center;
    gap: 8px;
}

.col-search input,
.col-search :slotted(input) {
    flex: 1;
    min-width: 0;
    border: 0;
    border-radius: 6px;
    padding: 7px 11px;
    color: rgb(var(--text-color));
    background: rgba(var(--outline-color), 0.08);
    font-size: 12px;
}

.song-item {
    height: 64px;
    box-sizing: border-box;
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

.list-header-action:disabled { cursor: default; opacity: .3; }

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
    display: flex;
    align-items: center;
    min-width: 0;
    gap: 8px;
    font-size: 12px;
    color: rgba(var(--text-color), 0.6);
}

.song-tag-badge {
    flex: 0 0 auto;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 1px 7px;
    border-radius: 5px;
    background: rgba(var(--primary-color), 0.16);
    color: rgb(var(--text-color));
    font-size: 11px;
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
