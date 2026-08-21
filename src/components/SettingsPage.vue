<template>
    <PageLayout>
        <template #header>
            <!-- 页面标题 -->
            <div class="music-banner">
                <div class="image-container">
                    <MotionTransition variant="banner">
                        <img :key="currentSong.cover" class="background-image" :src="currentSong.cover"
                            referrerpolicy="no-referrer">
                    </MotionTransition>
                </div>
                <div class="banner-content">
                    <div class="title">DROPIN MUSIC PLAYER</div>
                    <h2 class="library-title">设置</h2>
                </div>
            </div>
        </template>
        <div class="settings-page">
            <section class="settings-row">
                <div class="setting-copy">
                    <div>
                        <h2>动画帧率上限</h2>
                    </div>
                    <strong class="status-pill">{{ frameRateLabel }}</strong>
                </div>
                <div class="settings-note">
                    <Icon src="/assets/info.svg" size="sm" />
                    <span>更高的帧率带来更平滑的画面，但会造成更高的性能开销。</span>
                </div>

                <div style="display: grid; grid-template-columns: 1fr auto; gap: 20px">
                    <div class="frame-rate-control" :class="{ disabled: isUnlimited }">
                        <div class="frame-rate-labels">
                            <div class="frame-rate-mark">
                                <LeafIcon :size="14" />
                                <span>{{ frameRateLimits.min }} 帧</span>
                            </div>
                            <div class="frame-rate-mark">
                                <ZapIcon :size="14" />
                                <span>{{ frameRateLimits.max }} 帧</span>
                            </div>
                        </div>
                        <RangeSlider v-model="draftFrameRate" :min="frameRateLimits.min" :max="frameRateLimits.max"
                            :step="1" aria-label="动画帧率上限" :aria-value-text="draftFrameRate + ' 帧'"
                            @input="handleFrameRateInput" @change="handleFrameRateInput" />
                    </div>
                    <label class="toggle-row">
                        <input type="checkbox" :checked="isUnlimited" @change="handleUnlimitedChange" />
                        <span>不限制帧率</span>
                    </label>
                </div>
            </section>
        </div>
    </PageLayout>
</template>

<script setup>
import { computed, inject, ref, watch } from 'vue'
import { GaugeIcon, SnailIcon, ZapIcon } from '@lucide/vue'
import Icon from './Icon.vue'
import MotionTransition from './MotionTransition.vue'
import PageLayout from './PageLayout.vue'
import RangeSlider from './RangeSlider.vue'
import { frameRateLimits, useAppSettingsStore } from '../stores/appSettingsStore.js'
import { LeafIcon } from '@lucide/vue'

const currentSong = inject('currentSong')
const settingsStore = useAppSettingsStore()
const draftFrameRate = ref(settingsStore.state.animationFrameRate ?? frameRateLimits.default)

const isUnlimited = computed(() => settingsStore.state.animationFrameRate === null)
const frameRateLabel = computed(() => isUnlimited.value ? '无限制' : settingsStore.state.animationFrameRate + ' 帧')

watch(() => settingsStore.state.animationFrameRate, (value) => {
    if (value !== null) draftFrameRate.value = value
})

const handleFrameRateInput = (value) => {
    draftFrameRate.value = value
    if (!isUnlimited.value) settingsStore.updateAnimationFrameRate(value)
}

const handleUnlimitedChange = (event) => {
    settingsStore.updateAnimationFrameRate(event.target.checked ? null : draftFrameRate.value)
}
</script>

<style scoped>
.settings-page {
    display: grid;
    gap: 0;
    width: 100%;
}

.settings-row {
    display: grid;
    gap: 14px;
    width: 100%;
    padding: 14px 0 16px;
    border-top: 1px solid rgba(var(--outline-color), 0.12);
    border-bottom: 1px solid rgba(var(--outline-color), 0.12);
}

.setting-copy {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 18px;
}

.setting-copy h2 {
    margin: 0 0 5px;
    font-size: 15px;
    line-height: 1.25;
}

.setting-copy p {
    max-width: 560px;
    margin: 0;
    color: rgba(var(--text-color), 0.58);
    font-size: 12px;
    line-height: 1.55;
}

.status-pill {
    flex: 0 0 auto;
    padding: 2px 0;
    color: rgb(var(--text-color));
    background: transparent;
    font-size: 12px;
    font-weight: 600;
}

.toggle-row {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    width: max-content;
    color: rgb(var(--text-color));
    font-size: 12px;
    cursor: pointer;
}

.toggle-row input {
    width: 16px;
    height: 16px;
    accent-color: rgb(var(--primary-color));
}

.frame-rate-control {
    display: grid;
    gap: 10px;
    transition: opacity 160ms ease;
}

.frame-rate-control.disabled {
    opacity: 0.42;
}

.frame-rate-labels {
    display: flex;
    justify-content: space-between;
    color: rgba(var(--text-color), 0.58);
    font-size: 11px;
}

.frame-rate-mark {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    white-space: nowrap;
}

.frame-rate-mark.current {
    color: rgb(var(--text-color));
}

.settings-note {
    display: flex;
    align-items: center;
    gap: 10px;
    padding-top: 2px;
    color: rgba(var(--text-color), 0.72);
    background: transparent;
    font-size: 12px;
    line-height: 1.5;
}

@media (max-width: 720px) {
    .setting-copy {
        flex-direction: column;
    }
}
</style>
