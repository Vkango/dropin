<template>
    <aside class="detail-panel">
        <div class="detail-header">
            <span class="detail-title">Detail</span>
        </div>
        <div class="current-song">
            <img :src="currentSong.cover" :alt="currentSong.title" class="current-cover" />
            <div class="current-info">
                <div class="info-row">
                    <span class="info-label">Title</span>
                    <span class="info-value">{{ currentSong.title }}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">Artist</span>
                    <span class="info-value">{{ currentSong.artist }}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">Album</span>
                    <span class="info-value">{{ currentSong.album }}</span>
                </div>
            </div>
        </div>
    </aside>
</template>

<script setup>
import { defineProps } from 'vue'

const props = defineProps({
    currentSong: {
        type: Object,
        required: true
    }
})
</script>

<style scoped>
.detail-panel {
    grid-area: detail;
    background: rgba(var(--surface-color), 0.8);
    padding: 20px;
    border-left: 1px solid #3a3a3a;
}

.detail-header {
    margin-bottom: 24px;
}

.detail-title {
    font-size: 18px;
    font-weight: 600;
}

.current-cover {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 8px;
    object-fit: cover;
    margin-bottom: 20px;
}

.info-row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
    padding: 8px 0;
}

.info-label {
    color: rgba(var(--text-color), 0.6);
    font-size: 14px;
}

.info-value {
    font-size: 14px;
    text-align: right;
    max-width: 60%;
    word-break: break-word;
}
</style>
