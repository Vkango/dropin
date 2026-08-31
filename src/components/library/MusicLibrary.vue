<template>
    <PageLayout>
        <template #header>
            <!-- Banner 区域 -->
            <div class="music-banner">
                <div class="image-container">
                    <MotionTransition variant="banner">
                        <img :key="bannerImage" class="background-image" :src="bannerImage"
                            referrerpolicy="no-referrer">
                    </MotionTransition>
                </div>
                <div class="banner-content">
                    <div class="title">{{ t('app.name') }}</div>
                    <h2 class="library-title">{{ resolvedTitle }}</h2>
                    <div class="description">{{ resolvedDescription }}</div>
                </div>
                <div class="controls-row">
                    <MotionButton v-for="control in headerControls" :key="control.id" class="control-btn"
                        :while-hover="{ y: -1, backgroundColor: 'rgba(255,255,255, 0.1)', boxShadow: '0 4px 15px rgba(0, 0, 0, 0.3)' }"
                        :while-press="{ scale: 0.96 }" :transition="microTransition"
                        :class="{ selected: control.selected }" @click.stop="$emit('header-control-click', control)">

                        <Icon :src="getIconPath(control.icon)" size="xs" />
                        <span>{{ control.label }}</span>

                    </MotionButton>
                </div>
            </div>
        </template>
        <div ref="pageRef" class="library-content-with-alphabet">
            <div class="song-list-container">
                <SongList :songs="musicLibrary.songs" :primary-action-label="resolvedPrimaryActionLabel"
                    :primary-action-clickable="primaryActionClickable" :show-play-all="showPlayAll"
                    @primary-action="$emit('primary-action')" @play-all="$emit('play-all')"
                    @song-select="$emit('song-select', $event)" @song-play="$emit('song-play', $event)"
                    @group-label-click="handleGroupLabelClick" />
            </div>
            <AlphabetFilter :active-initial="activeInitial" :top-offset="alphabetTopOffset"
                :available-initials="availableInitials"
                @select="handleAlphabetSelect" />
        </div>

    </PageLayout>
</template>

<script setup>
import { defineProps, defineEmits, computed, inject, ref } from 'vue'
import SongList from './SongList.vue'
import AlphabetFilter from '@/components/ui/AlphabetFilter.vue'
import Icon from '@/components/ui/Icon.vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import MotionTransition from '@/components/ui/MotionTransition.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '@/utils/motion.js'
import { getAvailableInitials } from '@/utils/alphabet.js'
import { useAlphabetNavigation } from '@/utils/useAlphabetNavigation.js'
import { useI18n } from '@/i18n/index.js'

const { t } = useI18n()
const currentSong = inject('currentSong')

const props = defineProps({
    musicLibrary: {
        type: Object,
        required: true
    },
    headerControls: {
        type: Array,
        default: () => []
    },
    title: {
        type: String,
        default: ''
    },
    description: {
        type: String,
        default: ''
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

const emit = defineEmits(['header-control-click', 'primary-action', 'play-all', 'song-select', 'song-play'])
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const resolvedTitle = computed(() => props.title || t('library.title'))
const resolvedDescription = computed(() => props.description
    || props.musicLibrary.totalSongs + ' songs • ' + props.musicLibrary.totalDuration)
const resolvedPrimaryActionLabel = computed(() => props.primaryActionLabel || t('library.addToPlaylist'))

const headerControls = computed(() => props.headerControls?.length ? props.headerControls : [
    { id: 'all', icon: 'library.svg', label: t('library.all'), selected: true },
    { id: 'system', icon: 'folder.svg', label: t('library.system'), selected: false },
    { id: 'local', icon: 'sys_music.svg', label: t('library.local'), selected: false },
    { id: 'import', icon: 'ext.svg', label: t('library.import'), selected: false },
    { id: 'network', icon: 'cloud.svg', label: t('library.network'), selected: false }
])

const pageRef = ref(null)
const availableInitials = computed(() => getAvailableInitials(props.musicLibrary.songs, (song) => song.title))
const { activeInitial, alphabetTopOffset, handleAlphabetSelect, handleGroupLabelClick } = useAlphabetNavigation(
    pageRef,
    availableInitials
)

const bannerImage = computed(() => {
    return currentSong.value.cover
})

const getIconPath = (iconName) => {
    return `/assets/${iconName}`
}
</script>

<style scoped>
.controls-row {
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
    color: rgb(var(--primary-color), 0.5);
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
    color: rgba(var(--text-color), 0.6);
    font-size: 14px;
}

.stat-icon {
    font-size: 16px;
}

/* 歌曲列表 */
.song-list-container {
    width: 100%;
}

.library-content-with-alphabet {
    display: flex;
    align-items: flex-start;
    gap: 16px;
}

.library-content-with-alphabet .song-list-container {
    min-width: 0;
    flex: 1;
}

@media (max-width: 768px) {
    .library-content-with-alphabet {
        gap: 4px;
    }
}
</style>
