<template>
    <div class="plugins-page">
        <!-- 页面标题 -->
        <div class="page-header">
            <h1 class="page-title">扩展插件</h1>
            <div class="page-actions">
                <MotionButton class="install-btn" :while-hover="{ y: -1, backgroundColor: 'rgba(var(--primary-color), 0.2)' }"
                    :while-press="{ scale: 0.96 }" :transition="microTransition" @click="showInstallDialog">
                    <Icon src="/assets/ext.svg" size="sm" />
                    安装插件
                </MotionButton>
                <MotionButton class="refresh-btn" :while-hover="{ y: -1, backgroundColor: 'rgba(var(--primary-color), 0.2)' }"
                    :while-press="{ scale: 0.96 }" :transition="microTransition" @click="refreshPlugins">
                    <Icon src="/assets/restore.svg" size="sm" />
                    刷新
                </MotionButton>
            </div>
        </div>

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
        <div class="plugins-grid">
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
                    <h3 class="plugin-name">{{ plugin.name }}</h3>
                    <p class="plugin-author">{{ plugin.author }}</p>
                    <p class="plugin-description">{{ plugin.description }}</p>

                    <div class="plugin-meta">
                        <span class="version">v{{ plugin.version }}</span>
                        <span class="downloads">{{ formatNumber(plugin.downloads) }} 下载</span>
                        <div class="rating">
                            <div class="stars">
                                <span v-for="i in 5" :key="i" class="star" :class="{ filled: i <= plugin.rating }">
                                    ★
                                </span>
                            </div>
                            <span class="rating-text">({{ plugin.reviews }})</span>
                        </div>
                    </div>

                    <div class="plugin-tags">
                        <span v-for="tag in plugin.tags" :key="tag" class="tag">
                            {{ tag }}
                        </span>
                    </div>
                </div>

                <div class="plugin-actions">
                    <MotionButton v-if="!plugin.installed" class="action-btn install"
                        :while-hover="{ y: -1, backgroundColor: 'rgba(var(--primary-color), 0.8)' }"
                        :while-press="{ scale: 0.96 }" :transition="microTransition" @click="installPlugin(plugin)">
                        安装
                    </MotionButton>
                    <MotionButton v-else-if="plugin.disabled" class="action-btn enable"
                        :while-hover="{ y: -1, backgroundColor: '#45a049' }" :while-press="{ scale: 0.96 }"
                        :transition="microTransition" @click="enablePlugin(plugin)">
                        启用
                    </MotionButton>
                    <div v-else class="installed-actions">
                        <MotionButton class="action-btn disable"
                            :while-hover="{ backgroundColor: 'rgba(var(--outline-color), 0.2)' }"
                            :while-press="{ scale: 0.96 }" :transition="microTransition" @click="disablePlugin(plugin)">
                            禁用
                        </MotionButton>
                        <MotionButton class="action-btn uninstall"
                            :while-hover="{ backgroundColor: 'rgba(244, 67, 54, 0.2)' }"
                            :while-press="{ scale: 0.96 }" :transition="microTransition" @click="uninstallPlugin(plugin)">
                            卸载
                        </MotionButton>
                    </div>
                </div>
            </MotionDiv>
        </div>

        <!-- 空状态 -->
        <div v-if="filteredPlugins.length === 0" class="empty-state">
            <Icon src="/assets/plugin.svg" size="xl" />
            <h3>没有找到插件</h3>
            <p>尝试切换到其他分类或刷新插件列表</p>
        </div>
    </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import Icon from './Icon.vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'
const emit = defineEmits(['plugin-install', 'plugin-uninstall', 'plugin-enable', 'plugin-disable'])

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const cardVariants = {
    rest: { y: 0, boxShadow: '0 0 0 rgba(0, 0, 0, 0)', borderColor: 'rgba(var(--outline-color), 0.1)' },
    hover: { y: -2, boxShadow: '0 8px 24px rgba(0, 0, 0, 0.1)', borderColor: 'rgba(var(--primary-color), 0.2)' }
}

const activeCategory = ref('all')

const categories = ref([
    { id: 'all', label: '全部', count: 12 },
    { id: 'effects', label: '音效', count: 4 },
    { id: 'visualizer', label: '可视化', count: 3 },
    { id: 'utility', label: '工具', count: 3 },
    { id: 'theme', label: '主题', count: 2 }
])

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

const formatNumber = (num) => {
    if (num >= 1000000) {
        return (num / 1000000).toFixed(1) + 'M'
    }
    if (num >= 1000) {
        return (num / 1000).toFixed(1) + 'K'
    }
    return num.toString()
}

const showInstallDialog = () => {
    console.log('显示安装对话框')
}

const refreshPlugins = () => {
    console.log('刷新插件列表')
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
    padding: 20px 50px;
    height: 100%;
    overflow-y: auto;
}

/* 页面标题 */
.page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
}

.page-title {
    font-size: 48px;
    font-weight: 700;
    color: rgb(var(--text-color));
}

.page-actions {
    display: flex;
    gap: 12px;
}

.install-btn,
.refresh-btn {
    background: rgba(var(--primary-color), 0.1);
    border: 1px solid rgba(var(--primary-color), 0.3);
    border-radius: 8px;
    padding: 10px 16px;
    color: rgba(var(--primary-color), 0.3);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
}

/* 筛选标签 */
.filter-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 30px;
    background: rgba(var(--surface-color), 0.05);
    border-radius: 12px;
    padding: 4px;
}

.filter-tab {
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 10px 16px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    color: rgba(var(--text-color), 0.7);
}

.filter-tab.active {
    background: rgba(var(--primary-color), 0.3);
    color: white;
}

.count {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 10px;
    padding: 2px 6px;
    font-size: 12px;
    font-weight: 600;
}

.filter-tab.active .count {
    background: rgba(255, 255, 255, 0.3);
}

/* 插件网格 */
.plugins-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: 24px;
}

.plugin-card {
    background: rgba(var(--surface-color), 0.05);
    border: 1px solid rgba(var(--outline-color), 0.1);
    border-radius: 16px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.plugin-card.installed {
    border-color: rgba(var(--primary-color), 0.3);
    background: rgba(var(--primary-color), 0.02);
}

.plugin-card.disabled {
    opacity: 0.6;
    border-color: rgba(var(--outline-color), 0.2);
}

.plugin-icon {
    position: relative;
    width: 48px;
    height: 48px;
    flex-shrink: 0;
}

.plugin-icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 8px;
}

.status-badge {
    position: absolute;
    top: -4px;
    right: -4px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--md-sys-color-background);
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
}

.plugin-name {
    font-size: 18px;
    font-weight: 600;
    color: rgb(var(--text-color));
    margin-bottom: 4px;
}

.plugin-author {
    font-size: 14px;
    color: rgba(var(--primary-color), 0.3);
    margin-bottom: 8px;
}

.plugin-description {
    font-size: 14px;
    color: rgba(var(--text-color), 0.7);
    line-height: 1.4;
    margin-bottom: 12px;
}

.plugin-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    font-size: 12px;
    color: rgba(var(--text-color), 0.6);
}

.version {
    background: rgba(var(--outline-color), 0.1);
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 500;
}

.rating {
    display: flex;
    align-items: center;
    gap: 4px;
}

.stars {
    color: #FFC107;
}

.star {
    font-size: 12px;
}

.star.filled {
    color: #FFC107;
}

.star:not(.filled) {
    color: rgba(var(--outline-color), 0.3);
}

.plugin-tags {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 16px;
}

.tag {
    background: rgba(var(--primary-color), 0.1);
    color: rgba(var(--primary-color), 0.3);
    padding: 4px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
}

.plugin-actions {
    margin-top: auto;
}

.action-btn {
    border: none;
    border-radius: 8px;
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    margin-right: 8px;
}

.action-btn.install {
    background: rgba(var(--primary-color), 0.3);
    color: white;
}

.action-btn.enable {
    background: #4CAF50;
    color: white;
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
    gap: 8px;
}

/* 空状态 */
.empty-state {
    text-align: center;
    padding: 60px 20px;
    color: rgba(var(--text-color), 0.6);
}

.empty-state h3 {
    font-size: 20px;
    margin: 16px 0 8px;
    color: rgba(var(--text-color), 0.8);
}

.empty-state p {
    font-size: 14px;
    line-height: 1.5;
}

/* 滚动条样式 */
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

/* 响应式设计 */
@media (max-width: 768px) {
    .plugins-page {
        padding: 20px 24px;
    }

    .page-title {
        font-size: 36px;
    }

    .page-header {
        flex-direction: column;
        gap: 20px;
        align-items: flex-start;
    }

    .plugins-grid {
        grid-template-columns: 1fr;
        gap: 16px;
    }

    .filter-tabs {
        overflow-x: auto;
        padding-bottom: 4px;
    }
}
</style>
