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
                        <h2>外观主题</h2>
                    </div>
                    <strong class="status-pill">{{ themeModeLabel }}</strong>
                </div>

                <div class="segmented-control" role="radiogroup" aria-label="外观主题">
                    <button v-for="option in themeModeOptions" :key="option.value" type="button" class="segment-button"
                        :class="{ active: settingsStore.state.themeMode === option.value }"
                        :aria-pressed="settingsStore.state.themeMode === option.value"
                        @click="settingsStore.updateThemeMode(option.value)">
                        {{ option.label }}
                    </button>
                </div>
            </section>

            <section class="settings-row">
                <div class="setting-copy">
                    <div>
                        <h2>主题颜色</h2>
                    </div>
                    <span class="color-status" :style="themeColorStyle"></span>
                </div>

                <label class="toggle-row">
                    <input type="checkbox" :checked="settingsStore.state.autoAlbumTheme"
                        @change="handleAutoAlbumThemeChange" />
                    <span>从专辑封面自动拾取主题颜色</span>
                </label>

                <div v-if="!settingsStore.state.autoAlbumTheme" class="manual-color-control">
                    <label class="color-picker-wrap">
                        <span class="color-preview" :style="themeColorStyle"></span>
                        <input type="color" :value="settingsStore.state.manualThemeColor" aria-label="选择主题颜色"
                            @input="handleColorPickerInput" />
                    </label>
                    <label class="hex-input-wrap">
                        <span>#</span>
                        <input :value="manualColorText.slice(1)" maxlength="6" inputmode="text" aria-label="主题颜色十六进制值"
                            @input="handleManualColorInput" @blur="commitManualColorInput"
                            @keydown.enter="commitManualColorInput" />
                    </label>
                    <span class="hex-value">{{ settingsStore.state.manualThemeColor }}</span>
                </div>
            </section>

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
import { ZapIcon } from '@lucide/vue'
import Icon from './Icon.vue'
import MotionTransition from './MotionTransition.vue'
import PageLayout from './PageLayout.vue'
import RangeSlider from './RangeSlider.vue'
import { frameRateLimits, useAppSettingsStore, THEME_MODES } from '../stores/appSettingsStore.js'
import { LeafIcon } from '@lucide/vue'

const currentSong = inject('currentSong')
const settingsStore = useAppSettingsStore()
const draftFrameRate = ref(settingsStore.state.animationFrameRate ?? frameRateLimits.default)
const manualColorText = ref(settingsStore.state.manualThemeColor)

const themeModeOptions = [
    { value: 'system', label: '跟随系统' },
    { value: 'light', label: '亮色' },
    { value: 'dark', label: '暗色' }
].filter((option) => THEME_MODES.includes(option.value))

const isUnlimited = computed(() => settingsStore.state.animationFrameRate === null)
const frameRateLabel = computed(() => isUnlimited.value ? '无限制' : settingsStore.state.animationFrameRate + ' 帧')
const themeModeLabel = computed(() => themeModeOptions.find((option) => option.value === settingsStore.state.themeMode)?.label || '跟随系统')
const themeColorStyle = computed(() => ({ backgroundColor: settingsStore.state.manualThemeColor }))

watch(() => settingsStore.state.animationFrameRate, (value) => {
    if (value !== null) draftFrameRate.value = value
})

watch(() => settingsStore.state.manualThemeColor, (value) => {
    manualColorText.value = value
})

const handleFrameRateInput = (value) => {
    draftFrameRate.value = value
    if (!isUnlimited.value) settingsStore.updateAnimationFrameRate(value)
}

const handleUnlimitedChange = (event) => {
    settingsStore.updateAnimationFrameRate(event.target.checked ? null : draftFrameRate.value)
}

const handleAutoAlbumThemeChange = (event) => {
    settingsStore.updateAutoAlbumTheme(event.target.checked)
}

const handleColorPickerInput = (event) => {
    settingsStore.updateManualThemeColor(event.target.value)
}

const handleManualColorInput = (event) => {
    const value = `#${event.target.value.replace(/[^0-9a-f]/gi, '').slice(0, 6)}`
    manualColorText.value = value
    if (/^#[0-9a-f]{6}$/i.test(value)) settingsStore.updateManualThemeColor(value)
}

const commitManualColorInput = () => {
    if (!/^#[0-9a-f]{6}$/i.test(manualColorText.value)) {
        manualColorText.value = settingsStore.state.manualThemeColor
        return
    }
    settingsStore.updateManualThemeColor(manualColorText.value)
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

.segmented-control {
    display: inline-flex;
    width: max-content;
    max-width: 100%;
    border: 1px solid rgba(var(--outline-color), 0.28);
    border-radius: 5px;
    overflow: hidden;
}

.segment-button {
    min-width: 88px;
    padding: 7px 12px;
    border: 0;
    border-right: 1px solid rgba(var(--outline-color), 0.2);
    color: rgba(var(--text-color), 0.62);
    background: transparent;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: color 160ms ease, background-color 160ms ease;
}

.segment-button:last-child {
    border-right: 0;
}

.segment-button:hover,
.segment-button.active {
    color: rgb(var(--text-color));
    background: rgba(var(--primary-color), 0.14);
}

.color-status,
.color-preview {
    display: block;
    flex: 0 0 auto;
    width: 18px;
    height: 18px;
    border: 1px solid rgba(var(--text-color), 0.24);
    border-radius: 50%;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.16);
}

.manual-color-control {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 32px;
}

.color-picker-wrap {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
}

.color-picker-wrap input[type='color'] {
    width: 28px;
    height: 24px;
    padding: 0;
    border: 1px solid rgba(var(--outline-color), 0.35);
    border-radius: 3px;
    background: transparent;
    cursor: pointer;
}

.hex-input-wrap {
    display: inline-flex;
    align-items: center;
    height: 28px;
    padding: 0 8px;
    border-bottom: 1px solid rgba(var(--outline-color), 0.34);
    color: rgba(var(--text-color), 0.64);
    font-size: 12px;
}

.hex-input-wrap input {
    width: 54px;
    padding: 0;
    border: 0;
    outline: 0;
    color: rgb(var(--text-color));
    background: transparent;
    font: inherit;
    letter-spacing: 0.04em;
}

.hex-value {
    color: rgba(var(--text-color), 0.48);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
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

    .segmented-control {
        width: 100%;
    }

    .segment-button {
        flex: 1 1 0;
        min-width: 0;
    }

    .manual-color-control {
        flex-wrap: wrap;
    }
}
</style>
