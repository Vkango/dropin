<template>
    <aside class="sidebar-scroll">
        <div class="sidebar-titlebar-spacer" aria-hidden="true"></div>
        <div class="sidebar">
            <div class="sidebar-main">
                <nav class="nav-menu">
                    <MotionDiv v-for="item in sidebarItems" :key="item.id"
                        :class="['nav-item', { active: item.id === currentPage }]"
                        :aria-current="item.id === currentPage ? 'page' : undefined" :while-hover="{ y: -1 }"
                        :transition="microTransition" @click="$emit('nav-item-click', item)">
                        <MotionDiv v-if="item.id === currentPage" class="active-nav-indicator"
                            layout-id="sidebar-active-indicator" :initial="{ opacity: 0, scale: 0.92 }"
                            :animate="{ opacity: 1, scale: 1 }" :exit="{ opacity: 0, scale: 0.92 }"
                            :transition="activeIndicatorTransition" aria-hidden="true" />
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
                        <Icon class="section-chevron" :class="{ collapsed: !sections.tags }"
                            src="/assets/chevrondown.svg" :color="iconColor" />
                        <Icon class="section-icon" src="/assets/bookmark.svg" :color="iconColor" />
                        <span class="section-title">{{ t('sidebar.myTags') }}</span>
                        <MotionButton class="add-btn" :while-hover="{ color: '#ffffff', scale: 1.08 }"
                            :transition="microTransition" @click.stop="$emit('add-tag')">+</MotionButton>
                    </div>
                    <div v-if="sections.tags" class="section-content">
                        <MotionDiv v-for="tag in tags" :key="tag.id" class="nav-item" :while-hover="{ y: -1 }"
                            :transition="microTransition" @click="$emit('select-tag', tag)">
                            <span class="section-icon">
                                <Icon src="/assets/bookmark-item.svg" :color="iconColor" />
                            </span>
                            <span class="nav-label">{{ tag.label }}</span>
                        </MotionDiv>
                        <MotionDiv class="nav-item" :while-hover="{ y: -1 }" :transition="microTransition"
                            @click="$emit('select-tag', null)">
                            <span class="section-icon">
                                <Icon src="/assets/bookmark-item.svg" :color="iconColor" />
                            </span>
                            <span class="nav-label">{{ t('sidebar.untagged') }}</span>
                        </MotionDiv>
                    </div>
                </div>

                <!-- 播放列表 -->
                <div class="section" :class="{ collapsed: !sections.playlists }">
                    <div class="section-header" role="button" tabindex="0" :aria-expanded="sections.playlists"
                        @click="toggleSection('playlists')" @keydown.enter="toggleSection('playlists')"
                        @keydown.space.prevent="toggleSection('playlists')">
                        <Icon class="section-chevron" :class="{ collapsed: !sections.playlists }"
                            src="/assets/chevrondown.svg" :color="iconColor" />
                        <Icon class="section-icon" src="/assets/playlist.svg" style="scale: 0.8" :color="iconColor" />
                        <span class="section-title">{{ t('sidebar.playlists') }}</span>
                        <MotionButton class="add-btn" :while-hover="{ color: '#ffffff', scale: 1.08 }"
                            :transition="microTransition" @click.stop="$emit('add-playlist')">+</MotionButton>
                    </div>
                    <div v-if="sections.playlists" class="section-content">
                        <MotionDiv v-for="playlist in playlists" :key="playlist.id" class="nav-item"
                            :class="{ active: currentPage === 'playlists' && playlist.id === selectedPlaylistId }"
                            :while-hover="{ y: -1 }" :transition="microTransition"
                            @click="$emit('select-playlist', playlist)">
                            <MotionDiv v-if="currentPage === 'playlists' && playlist.id === selectedPlaylistId"
                                class="active-nav-indicator" layout-id="sidebar-active-indicator"
                                :initial="{ opacity: 0, scale: 0.92 }" :animate="{ opacity: 1, scale: 1 }"
                                :exit="{ opacity: 0, scale: 0.92 }" :transition="activeIndicatorTransition"
                                aria-hidden="true" />
                            <span class="section-icon">
                                <Icon src="/assets/playlist.svg" style="scale: 0.7" :color="iconColor" />
                            </span>
                            <span class="nav-label">{{ playlist.name }}</span>
                        </MotionDiv>
                    </div>
                </div>

                <!-- 扩展插件 -->
                <div class="section" :class="{ collapsed: !sections.plugins }">
                    <div class="section-header" role="button" tabindex="0" :aria-expanded="sections.plugins"
                        @click="toggleSection('plugins')" @keydown.enter="toggleSection('plugins')"
                        @keydown.space.prevent="toggleSection('plugins')">
                        <Icon class="section-chevron" :class="{ collapsed: !sections.plugins }"
                            src="/assets/chevrondown.svg" :color="iconColor" />
                        <Icon class="section-icon" src="/assets/plugin.svg" :color="iconColor" />
                        <span class="section-title">{{ t('sidebar.pluginsSection') }}</span>
                        <MotionButton class="add-btn" :while-hover="{ color: '#ffffff', scale: 1.08 }"
                            :transition="microTransition" @click.stop="$emit('add-plugin')">+</MotionButton>
                    </div>
                    <div v-if="sections.plugins" class="section-content">
                        <MotionDiv v-for="plugin in installedPlugins" :key="plugin.id" class="nav-item"
                            :class="{ active: `plugin:${plugin.id}` === currentPage }" :while-hover="{ y: -1 }"
                            :transition="microTransition" @click="$emit('select-plugin', plugin)">
                            <MotionDiv v-if="`plugin:${plugin.id}` === currentPage" class="active-nav-indicator"
                                layout-id="sidebar-active-indicator" :initial="{ opacity: 0, scale: 0.92 }"
                                :animate="{ opacity: 1, scale: 1 }" :exit="{ opacity: 0, scale: 0.92 }"
                                :transition="activeIndicatorTransition" aria-hidden="true" />
                            <span class="section-icon">
                                <img v-if="plugin.iconDataUrl" :src="plugin.iconDataUrl" alt="" />
                                <Icon v-else src="/assets/plugin.svg" :color="iconColor" />
                            </span>
                            <span class="nav-label">{{ plugin.name }}</span>
                        </MotionDiv>
                        <div v-if="installedPlugins.length === 0" class="plugin-empty">{{ t('plugins.empty') }}</div>
                    </div>

                </div>
            </div>

            <div class="sidebar-footer">
                <MotionDiv v-if="isDrawer" class="nav-item" :while-hover="{ y: -1 }" :transition="microTransition"
                    :aria-label="t('sidebar.collapseMenu')" @click="$emit('collapse')">
                    <span class="nav-icon">
                        <PanelLeftClose :size="16" :stroke-width="1.8" />
                    </span>
                    <span class="nav-label">{{ t('sidebar.collapseMenu') }}</span>
                </MotionDiv>
                <MotionDiv :class="['nav-item', { active: settingsItem.id === currentPage }]"
                    :aria-current="settingsItem.id === currentPage ? 'page' : undefined" :while-hover="{ y: -1 }"
                    :transition="microTransition" @click="$emit('nav-item-click', settingsItem)">
                    <MotionDiv v-if="settingsItem.id === currentPage" class="active-nav-indicator"
                        layout-id="sidebar-active-indicator" :initial="{ opacity: 0, scale: 0.92 }"
                        :animate="{ opacity: 1, scale: 1 }" :exit="{ opacity: 0, scale: 0.92 }"
                        :transition="activeIndicatorTransition" aria-hidden="true" />
                    <span class="nav-icon">
                        <Icon src="/assets/setting.svg" />
                    </span>
                    <span class="nav-label">{{ t('sidebar.settings') }}</span>
                </MotionDiv>
            </div>
        </div>
    </aside>
</template>

<script setup>
import { defineProps, defineEmits, computed, reactive } from 'vue'
import { PanelLeftClose } from '@lucide/vue'
import Icon from '@/components/ui/Icon.vue';
import { motion, useReducedMotion } from 'motion-v'
import { APPLE_SPRING, INSTANT_MOTION, MICRO_SPRING } from '@/utils/motion.js'
import { useI18n } from '@/i18n/index.js'

const { t } = useI18n()

const props = defineProps({
    sidebarItems: {
        type: Array,
        required: true
    },
    currentPage: {
        type: String,
        required: true
    },
    searchQuery: {
        type: String,
        default: ''
    },
    isDark: {
        type: Boolean,
        default: false
    },
    playlists: {
        type: Array,
        default: () => []
    },
    selectedPlaylistId: {
        type: String,
        default: ''
    },
    tags: {
        type: Array,
        default: () => []
    },
    installedPlugins: {
        type: Array,
        default: () => []
    },
    isDrawer: {
        type: Boolean,
        default: false
    }
})

const emit = defineEmits([
    'search-update',
    'nav-item-click',
    'add-tag',
    'add-playlist',
    'add-plugin',
    'select-playlist',
    'select-tag',
    'select-plugin',
    'collapse'
])

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const activeIndicatorTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const iconColor = computed(() => props.isDark ? 'rgba(255, 255, 255)' : 'rgba(0, 0, 0)')
const settingsItem = computed(() => ({ id: 'settings', icon: 'setting.svg', label: t('sidebar.settings') }))
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
.sidebar-scroll {
    width: 100%;
    height: 100%;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
    background: transparent;
}

.sidebar {
    display: flex;
    flex-direction: column;
    width: 100%;
    min-height: calc(100% - 104px);
    padding: 0 20px 0;
    margin-top: 10px;
}

.sidebar-main {
    flex: 0 0 auto;
}

.sidebar-footer {
    margin-top: auto;
    padding-top: 28px;
}

.section-icon img {
    width: 16px;
    height: 16px;
    border-radius: 4px;
    object-fit: cover;
}

.plugin-empty {
    padding: 6px 0 6px 28px;
    color: rgba(var(--text-color), 0.42);
    font-size: 11px;
}

.sidebar-titlebar-spacer {
    width: 100%;
    height: 64px;
    min-height: 64px;
    pointer-events: none;
}

.sidebar-scroll::-webkit-scrollbar {
    width: 4px;
}

.sidebar-scroll::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
    margin-top: 64px;
    margin-bottom: 8px;
}

.sidebar-scroll::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.sidebar-scroll::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
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
    position: relative;
    isolation: isolate;
    z-index: 0;
    display: flex;
    align-items: center;
    padding: 8px 18px;
    border-radius: 6px;
    cursor: pointer;
}

.nav-item:hover {
    background-color: rgba(var(--text-color), 0.06);
}

.nav-item.active,
.nav-item.active:hover {
    background-color: transparent;
    z-index: 1;
}

.active-nav-indicator {
    position: absolute;
    z-index: 0;
    inset: 0;
    border-radius: inherit;
    background: linear-gradient(105deg, rgba(var(--global-inverse-color), 0.18), rgba(var(--global-inverse-color), 0.08));
    box-shadow: inset 0 0 0 1px rgba(var(--global-inverse-color), 0.04), 0 5px 16px rgba(var(--global-inverse-color), 0.04);
    pointer-events: none;
    will-change: transform, opacity;
}

.nav-icon,
.nav-label,
.section-icon {
    position: relative;
    z-index: 1;
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
    scale: 0.9;
    opacity: 0.5;
}

.section-chevron {
    flex: 0 0 18px;
    margin-right: 8px;
    transition: transform 180ms ease;
    opacity: 0.5;
}

.section-chevron.collapsed {
    transform: rotate(-90deg);
}


.section-title {
    flex: 1;
    font-size: 13px;
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
