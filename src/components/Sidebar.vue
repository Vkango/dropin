<template>
    <aside class="sidebar">
        <div class="search-container">
            <input type="text" placeholder="Search Everywhere" class="search-input" :value="searchQuery"
                @input="$emit('search-update', $event.target.value)" />
        </div>

        <nav class="nav-menu">
            <MotionDiv v-for="item in sidebarItems" :key="item.id" :class="['nav-item', { active: item.active }]"
                :while-hover="{ backgroundColor: 'rgba(var(--text-color), 0.06)' }" :transition="microTransition"
                @click="$emit('nav-item-click', item)">
                <span class="nav-icon">
                    <Icon :src="'/assets/' + item.icon" />
                </span>
                <span class="nav-label">{{ item.label }}</span>
            </MotionDiv>
        </nav>

        <!-- 我的标签 -->
        <div class="section" :class="{ collapsed: !sections.tags }">
            <div class="section-header" role="button" tabindex="0" :aria-expanded="sections.tags"
                @click="toggleSection('tags')" @keydown.enter="toggleSection('tags')"
                @keydown.space.prevent="toggleSection('tags')">
                <Icon class="section-chevron" :class="{ collapsed: !sections.tags }" src="/assets/chevrondown.svg"
                    size="sm" :color="iconColor" />
                <Icon class="section-icon" src="/assets/bookmark.svg" size="sm" :color="iconColor" />
                <span class="section-title">我的标签</span>
                <MotionButton class="add-btn" :while-hover="{ color: '#ffffff', scale: 1.08 }"
                    :transition="microTransition" @click.stop="$emit('add-tag')">+</MotionButton>
            </div>
            <div v-if="sections.tags" class="section-content">
                <MotionDiv class="nav-item" :while-hover="{ backgroundColor: 'rgba(var(--text-color), 0.06)' }"
                    :transition="microTransition">
                    <span class="nav-icon">
                        <Icon src="/assets/bookmark-item.svg" size="sm" :color="iconColor" />
                    </span>
                    <span class="nav-label">未分类标签001</span>
                </MotionDiv>
            </div>
        </div>

        <!-- 播放列表 -->
        <div class="section" :class="{ collapsed: !sections.playlists }">
            <div class="section-header" role="button" tabindex="0" :aria-expanded="sections.playlists"
                @click="toggleSection('playlists')" @keydown.enter="toggleSection('playlists')"
                @keydown.space.prevent="toggleSection('playlists')">
                <Icon class="section-chevron" :class="{ collapsed: !sections.playlists }" src="/assets/chevrondown.svg"
                    size="sm" :color="iconColor" />
                <Icon class="section-icon" src="/assets/playlist.svg" size="sm" :color="iconColor" />
                <span class="section-title">播放列表</span>
                <MotionButton class="add-btn" :while-hover="{ color: '#ffffff', scale: 1.08 }"
                    :transition="microTransition" @click.stop="$emit('add-playlist')">+</MotionButton>
            </div>
        </div>

        <!-- 扩展插件 -->
        <div class="section" :class="{ collapsed: !sections.plugins }">
            <div class="section-header" role="button" tabindex="0" :aria-expanded="sections.plugins"
                @click="toggleSection('plugins')" @keydown.enter="toggleSection('plugins')"
                @keydown.space.prevent="toggleSection('plugins')">
                <Icon class="section-chevron" :class="{ collapsed: !sections.plugins }" src="/assets/chevrondown.svg"
                    size="sm" :color="iconColor" />
                <Icon class="section-icon" src="/assets/plugin.svg" size="sm" :color="iconColor" />
                <span class="section-title">扩展插件</span>
                <MotionButton class="add-btn" :while-hover="{ color: '#ffffff', scale: 1.08 }"
                    :transition="microTransition" @click.stop="$emit('add-plugin')">+</MotionButton>
            </div>
            <div v-if="sections.plugins" class="section-content">
                <MotionDiv class="nav-item" :while-hover="{ backgroundColor: 'rgba(var(--text-color), 0.06)' }"
                    :transition="microTransition">
                    <span class="nav-icon">👣</span>
                    <span class="nav-label">我的足迹</span>
                </MotionDiv>
                <MotionDiv class="nav-item" :while-hover="{ backgroundColor: 'rgba(var(--text-color), 0.06)' }"
                    :transition="microTransition">
                    <span class="nav-icon">⏰</span>
                    <span class="nav-label">定时停止</span>
                </MotionDiv>
            </div>

        </div>
    </aside>
</template>

<script setup>
import { defineProps, defineEmits, computed, reactive } from 'vue'
import Icon from './Icon.vue';
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
    sidebarItems: {
        type: Array,
        required: true
    },
    searchQuery: {
        type: String,
        default: ''
    },
    isDark: {
        type: Boolean,
        default: false
    }
})

const emit = defineEmits([
    'search-update',
    'nav-item-click',
    'add-tag',
    'add-playlist',
    'add-plugin'
])

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const iconColor = computed(() => props.isDark ? '#e8edf0' : '#565b5f')
const sections = reactive({
    tags: true,
    playlists: true,
    plugins: true
})

const toggleSection = (sectionName) => {
    sections[sectionName] = !sections[sectionName]
}
</script>

<style scoped>
.sidebar {
    grid-area: sidebar;
    padding: 20px;
    overflow-y: auto;
}

.search-container {
    margin-bottom: 24px;
}

.search-input {
    width: 100%;
    padding: 10px 12px;
    background: rgba(var(--surface-color), 0.72);
    border: none;
    border-radius: 6px;
    color: rgb(var(--text-color));
    font-size: 14px;
}

.search-input::placeholder {
    color: rgba(var(--text-color), 0.58);
}

.nav-menu {
    margin-bottom: 24px;
    display: flex;
    /* gap: 5px; */
    flex-direction: column;
}

.nav-item {
    display: flex;
    align-items: center;
    padding: 8px 18px;
    border-radius: 6px;
    cursor: pointer;
}

.nav-item.active {
    background: rgba(var(--text-color), 0.06);
    border-radius: 10px;
    box-shadow: 0 0 10px rgba(var(--text-color), 0.08);
}

.nav-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    margin-right: 18px;
    font-size: 16px;
}

.nav-label {
    font-size: 13px;
    color: rgb(var(--text-color));
}

.nav-menu .nav-icon {
    width: auto;
}

.section {
    margin-bottom: 24px;
}

.section.collapsed {
    margin-bottom: 14px;
}

.section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    padding: 0 12px;
    cursor: pointer;
    user-select: none;
}

.section-icon {
    flex: 0 0 22px;
    margin-right: 14px;
}

.section-chevron {
    flex: 0 0 18px;
    margin-right: 8px;
    transition: transform 180ms ease;
}

.section-chevron.collapsed {
    transform: rotate(-90deg);
}

.section-content {
    overflow: hidden;
}

.section-title {
    flex: 1;
    font-size: 14px;
    font-weight: 600;
}

.add-btn {
    background: none;
    border: none;
    color: rgba(var(--text-color), 0.58);
    cursor: pointer;
    font-size: 18px;
    padding: 0;
    width: 20px;
    height: 20px;
}
</style>
