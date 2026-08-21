<template>
    <PageLayout>
        <template #header>
        <!-- 页面标题 -->
        <div class="music-banner">
            <div class="image-container">
                <MotionTransition variant="banner">
                    <img :key="bannerImage" class="background-image" :src="bannerImage"
                        referrerpolicy="no-referrer">
                </MotionTransition>
            </div>
            <div class="banner-content">
                <div class="title">{{ t('app.name') }}</div>
                <h2 class="library-title">{{ t('plugins.title') }}</h2>
            </div>
        </div>
        </template>
        <div class="plugins-page">
        <!-- 筛选标签 -->
        <div class="filter-tabs">
            <MotionButton v-for="category in categories" :key="category.id" class="filter-tab"
                :while-hover="{ backgroundColor: 'rgba(var(--primary-color), 0.1)', color: 'rgba(var(--primary-color), 0.3)' }"
                :while-press="{ scale: 0.97 }" :transition="microTransition"
                :class="{ active: activeCategory === category.id }" @click="activeCategory = category.id">
                {{ category.label }}
                <span class="count">{{ category.count }}</span>
            </MotionButton>
        </div>

        <!-- 插件列表 -->
        <div class="plugins-list">
            <MotionDiv v-for="plugin in filteredPlugins" :key="plugin.id" class="plugin-card" initial="rest"
                while-hover="hover" :variants="cardVariants" :class="{
                installed: plugin.installed,
                disabled: plugin.disabled
            }">
                <div class="plugin-icon">
                    <img :src="plugin.icon" :alt="plugin.name" />
                    <div v-if="plugin.installed" class="status-badge installed">
                        <Icon src="/assets/task.svg" size="xs" />
                    </div>
                    <div v-else-if="plugin.disabled" class="status-badge disabled">
                        <Icon src="/assets/close.svg" size="xs" />
                    </div>
                </div>

                <div class="plugin-info">
                    <div class="plugin-heading">
                        <h3 class="plugin-name">{{ plugin.name }}</h3>
                        <span class="plugin-meta">v{{ plugin.version }} · {{ statusLabel(plugin) }}</span>
                    </div>
                    <p class="plugin-description">{{ plugin.description }}</p>
                </div>

                <div class="plugin-actions">
                    <MotionButton v-if="!plugin.installed" class="action-btn install"
                        :while-hover="{ y: -1, backgroundColor: 'rgba(var(--primary-color), 0.8)' }"
                        :while-press="{ scale: 0.96 }" :transition="microTransition" @click="installPlugin(plugin)">
                        {{ t('plugins.install') }}
                    </MotionButton>
                    <MotionButton v-else-if="plugin.disabled" class="action-btn enable"
                        :while-hover="{ y: -1, backgroundColor: '#45a049' }" :while-press="{ scale: 0.96 }"
                        :transition="microTransition" @click="enablePlugin(plugin)">
                        {{ t('plugins.enable') }}
                    </MotionButton>
                    <div v-else class="installed-actions">
                        <MotionButton class="action-btn disable"
                            :while-hover="{ backgroundColor: 'rgba(var(--outline-color), 0.2)' }"
                            :while-press="{ scale: 0.96 }" :transition="microTransition" @click="disablePlugin(plugin)">
                            {{ t('plugins.disable') }}
                        </MotionButton>
                        <MotionButton class="action-btn uninstall"
                            :while-hover="{ backgroundColor: 'rgba(244, 67, 54, 0.2)' }"
                            :while-press="{ scale: 0.96 }" :transition="microTransition" @click="uninstallPlugin(plugin)">
                            {{ t('plugins.uninstall') }}
                        </MotionButton>
                    </div>
                </div>
            </MotionDiv>
        </div>

        <!-- 空状态 -->
        <div v-if="filteredPlugins.length === 0" class="empty-state">
            <Icon src="/assets/plugin.svg" size="xl" />
            <h3>{{ t('plugins.empty') }}</h3>
        </div>
        </div>
    </PageLayout>
</template>

<script setup>
import { ref, computed, inject } from 'vue'
import Icon from './Icon.vue'
import MotionTransition from './MotionTransition.vue'
import PageLayout from './PageLayout.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()
const emit = defineEmits(['plugin-install', 'plugin-uninstall', 'plugin-enable', 'plugin-disable'])

const currentSong = inject('currentSong')
const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const bannerImage = computed(() => currentSong.value?.cover || '/assets/cover.jpg')
const cardVariants = {
    rest: { backgroundColor: 'rgba(var(--surface-color), 0)' },
    hover: { backgroundColor: 'rgba(var(--primary-color), 0.04)' }
}

const activeCategory = ref('all')

const categoryOptions = [
    { id: 'all', label: t('plugins.categoryAll') },
    { id: 'effects', label: t('plugins.categoryEffects') },
    { id: 'visualizer', label: t('plugins.categoryVisualizer') },
    { id: 'utility', label: t('plugins.categoryUtility') },
    { id: 'theme', label: t('plugins.categoryTheme') }
]

const plugins = ref([
    {
        id: 1,
        name: 'Advanced Equalizer',
        author: 'AudioTech',
        description: '专业级均衡器，支持31段EQ调节和多种预设模式',
        version: '2.1.0',
        downloads: 15420,
        rating: 5,
        reviews: 128,
        icon: '/assets/effect.svg',
        category: 'effects',
        tags: ['音效', '均衡器', '专业'],
        installed: true,
        disabled: false
    },
    {
        id: 2,
        name: 'Spectrum Visualizer',
        author: 'VisualMusic',
        description: '实时频谱可视化插件，支持多种显示模式和自定义颜色',
        version: '1.5.2',
        downloads: 8930,
        rating: 4,
        reviews: 67,
        icon: '/assets/spa.svg',
        category: 'visualizer',
        tags: ['可视化', '频谱', '实时'],
        installed: true,
        disabled: true
    },
    {
        id: 3,
        name: 'Lyrics Sync',
        author: 'MusicTools',
        description: '自动同步歌词显示，支持在线搜索和手动调整',
        version: '3.0.1',
        downloads: 23100,
        rating: 5,
        reviews: 256,
        icon: '/assets/reply.svg',
        category: 'utility',
        tags: ['歌词', '同步', '在线'],
        installed: false,
        disabled: false
    },
    {
        id: 4,
        name: 'Night Theme',
        author: 'ThemeStudio',
        description: '深色主题包，包含多种夜间模式和护眼配色',
        version: '1.2.0',
        downloads: 5670,
        rating: 4,
        reviews: 45,
        icon: '/assets/personalize.svg',
        category: 'theme',
        tags: ['主题', '深色', '护眼'],
        installed: false,
        disabled: false
    },
    {
        id: 5,
        name: 'Bass Booster',
        author: 'AudioEnhance',
        description: '低音增强插件，提供多级低音增强和虚拟环绕音效',
        version: '1.8.3',
        downloads: 12800,
        rating: 4,
        reviews: 89,
        icon: '/assets/effect.svg',
        category: 'effects',
        tags: ['低音', '增强', '环绕'],
        installed: false,
        disabled: false
    },
    {
        id: 6,
        name: 'Waveform Display',
        author: 'VisualAudio',
        description: '波形显示插件，实时显示音频波形和播放进度',
        version: '2.0.0',
        downloads: 7200,
        rating: 5,
        reviews: 34,
        icon: '/assets/spa.svg',
        category: 'visualizer',
        tags: ['波形', '显示', '进度'],
        installed: false,
        disabled: false
    }
])

const filteredPlugins = computed(() => {
    if (activeCategory.value === 'all') {
        return plugins.value
    }
    return plugins.value.filter(plugin => plugin.category === activeCategory.value)
})

const categories = computed(() => categoryOptions.map((category) => ({
    ...category,
    count: category.id === 'all'
        ? plugins.value.length
        : plugins.value.filter((plugin) => plugin.category === category.id).length
})))

const statusLabel = (plugin) => {
    if (!plugin.installed) return t('plugins.statusAvailable')
    return plugin.disabled ? t('plugins.statusDisabled') : t('plugins.statusInstalled')
}

const installPlugin = (plugin) => {
    plugin.installed = true
    emit('plugin-install', plugin)
    console.log('安装插件:', plugin.name)
}

const uninstallPlugin = (plugin) => {
    plugin.installed = false
    plugin.disabled = false
    emit('plugin-uninstall', plugin)
    console.log('卸载插件:', plugin.name)
}

const enablePlugin = (plugin) => {
    plugin.disabled = false
    emit('plugin-enable', plugin)
    console.log('启用插件:', plugin.name)
}

const disablePlugin = (plugin) => {
    plugin.disabled = true
    emit('plugin-disable', plugin)
    console.log('禁用插件:', plugin.name)
}
</script>

<style scoped>
.plugins-page {
    width: 100%;
}

.filter-tabs {
    display: flex;
    gap: 6px;
    margin-bottom: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid rgba(var(--outline-color), 0.12);
    overflow-x: auto;
}

.filter-tab {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 5px 8px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    color: rgba(var(--text-color), 0.7);
}

.filter-tab.active {
    background: rgba(var(--primary-color), 0.14);
    color: rgb(var(--text-color));
}

.count {
    color: rgba(var(--text-color), 0.46);
    font-size: 11px;
    font-weight: 600;
}

.plugins-list {
    display: grid;
    width: 100%;
}

.plugin-card {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    padding: 11px 0;
    border-bottom: 1px solid rgba(var(--outline-color), 0.1);
}

.plugin-card.installed {
    border-color: rgba(var(--primary-color), 0.16);
}

.plugin-card.disabled {
    opacity: 0.56;
}

.plugin-icon {
    position: relative;
    width: 34px;
    height: 34px;
    flex-shrink: 0;
}

.plugin-icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 6px;
}

.status-badge {
    position: absolute;
    top: -3px;
    right: -3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--md-sys-color-background);
}

.status-badge.installed {
    background: #4CAF50;
    color: white;
}

.status-badge.disabled {
    background: #f44336;
    color: white;
}

.plugin-info {
    flex: 1;
    min-width: 0;
}

.plugin-heading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    min-width: 0;
}

.plugin-name {
    margin: 0;
    min-width: 0;
    color: rgb(var(--text-color));
    font-size: 14px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.plugin-meta {
    flex: 0 0 auto;
    color: rgba(var(--text-color), 0.5);
    font-size: 11px;
    font-weight: 500;
}

.plugin-description {
    margin: 3px 0 0;
    color: rgba(var(--text-color), 0.58);
    font-size: 12px;
    line-height: 1.45;
}

.plugin-actions {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-width: 116px;
}

.action-btn {
    border: none;
    border-radius: 5px;
    padding: 5px 10px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
}

.action-btn.install {
    background: rgba(var(--primary-color), 0.18);
    color: rgb(var(--text-color));
}

.action-btn.enable {
    background: rgba(76, 175, 80, 0.16);
    color: rgb(var(--text-color));
}

.action-btn.disable {
    background: rgba(var(--outline-color), 0.1);
    color: rgba(var(--text-color), 0.7);
}

.action-btn.uninstall {
    background: rgba(244, 67, 54, 0.1);
    color: #f44336;
}

.installed-actions {
    display: flex;
    gap: 6px;
}

.empty-state {
    text-align: center;
    padding: 28px 0;
    color: rgba(var(--text-color), 0.6);
}

.empty-state h3 {
    margin: 10px 0 0;
    color: rgba(var(--text-color), 0.8);
    font-size: 14px;
}

.plugins-page::-webkit-scrollbar {
    width: 4px;
}

.plugins-page::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
}

.plugins-page::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.plugins-page::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
}

@media (max-width: 768px) {
    .plugin-card {
        align-items: flex-start;
        flex-wrap: wrap;
    }

    .plugin-info {
        flex-basis: calc(100% - 46px);
    }

    .plugin-heading {
        align-items: flex-start;
        flex-direction: column;
        gap: 3px;
    }

    .plugin-actions {
        width: 100%;
        min-width: 0;
        justify-content: flex-start;
    }
}
</style>
