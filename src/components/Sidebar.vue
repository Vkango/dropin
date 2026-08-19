<template>
    <aside class="sidebar">
        <div class="search-container">
            <input type="text" placeholder="Search Everywhere" class="search-input" :value="searchQuery"
                @input="$emit('search-update', $event.target.value)" />
        </div>

        <nav class="nav-menu">
            <div v-for="item in sidebarItems" :key="item.id" :class="['nav-item', { active: item.active }]"
                @click="$emit('nav-item-click', item)">
                <span class="nav-icon">
                    <Icon :src="'/assets/' + item.icon" />
                </span>
                <span class="nav-label">{{ item.label }}</span>
            </div>
        </nav>

        <!-- 我的标签 -->
        <div class="section">
            <div class="section-header">
                <span class="section-icon">📁</span>
                <span class="section-title">我的标签</span>
                <button class="add-btn" @click="$emit('add-tag')">+</button>
            </div>
            <div class="nav-item">
                <span class="nav-icon">🏷️</span>
                <span class="nav-label">未分类标签001</span>
            </div>
        </div>

        <!-- 播放列表 -->
        <div class="section">
            <div class="section-header">
                <span class="section-icon">▶️</span>
                <span class="section-title">播放列表</span>
                <button class="add-btn" @click="$emit('add-playlist')">+</button>
            </div>
        </div>

        <!-- 扩展插件 -->
        <div class="section">
            <div class="section-header">
                <span class="section-icon">🧩</span>
                <span class="section-title">扩展插件</span>
                <button class="add-btn" @click="$emit('add-plugin')">+</button>
            </div>
            <div class="nav-item">
                <span class="nav-icon">👣</span>
                <span class="nav-label">我的足迹</span>
            </div>
            <div class="nav-item">
                <span class="nav-icon">⏰</span>
                <span class="nav-label">定时停止</span>
            </div>

        </div>
    </aside>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue'
import Icon from './Icon.vue';

const props = defineProps({
    sidebarItems: {
        type: Array,
        required: true
    },
    searchQuery: {
        type: String,
        default: ''
    }
})

const emit = defineEmits([
    'search-update',
    'nav-item-click',
    'add-tag',
    'add-playlist',
    'add-plugin'
])
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
    background: #3a3a3a;
    border: none;
    border-radius: 6px;
    color: #ffffff;
    font-size: 14px;
}

.search-input::placeholder {
    color: #888;
}

.nav-menu {
    margin-bottom: 24px;
    display: flex;
    gap: 5px;
    flex-direction: column;
}

.nav-item {
    display: flex;
    align-items: center;
    padding: 10px 12px;
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.2s;
}

.nav-item:hover {
    background: rgba(255, 255, 255, 0.05);
}

.nav-item.active {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
}

.nav-icon {
    margin-right: 18px;
    font-size: 16px;
}

.nav-label {
    font-size: 14px;
}

.section {
    margin-bottom: 24px;
}

.section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    padding: 0 12px;
}

.section-icon {
    margin-right: 8px;
}

.section-title {
    flex: 1;
    font-size: 14px;
    font-weight: 600;
}

.add-btn {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 18px;
    padding: 0;
    width: 20px;
    height: 20px;
}

.add-btn:hover {
    color: #ffffff;
}
</style>