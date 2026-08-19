<template>
    <div class="sound-effects-page">
        <!-- 页面标题 -->
        <div class="music-banner" @click="showAlbumDetail">
            <div class="image-container">
                <Transition name="banner-image">
                    <img :key="currentSong.cover" class="background-image" :src="currentSong.cover"
                        referrerpolicy="no-referrer">
                </Transition>
            </div>
            <div class="banner-content">
                <div class="title">DROPIN MUSIC PLAYER</div>
                <h2 class="library-title">声音效果</h2>
                <div class="description">音效调节实时生效，尽情尝试以达到理想音效！
                </div>
                <div class="header-content">
                    <div class="preset-controls">
                        <select v-model="currentPreset" class="preset-select">
                            <option value="original">Original</option>
                            <option value="rock">Rock</option>
                            <option value="pop">Pop</option>
                            <option value="jazz">Jazz</option>
                            <option value="classical">Classical</option>
                            <option value="electronic">Electronic</option>
                            <option value="vocal">Vocal</option>
                            <option value="custom">Custom</option>
                        </select>
                        <select v-model="viewMode" class="view-select">
                            <option value="custom">Custom</option>
                            <option value="advanced">Advanced</option>
                        </select>
                    </div>
                </div>
            </div>
        </div>
        <div class="page-header">

        </div>

        <!-- 警告提示 -->
        <div class="warning-banner">
            <Icon src="/assets/info.svg" size="sm" />
            <span>自定义调整的文件会被自动保存。你可以点击上方查看。</span>
        </div>

        <!-- 音效控制面板 -->
        <div class="effects-grid">
            <!-- 播放控制 -->
            <div class="effect-section">
                <h3 class="section-title">播放控制</h3>
                <div class="controls-group">
                    <div class="control-item">
                        <label class="control-label">
                            <input type="checkbox" v-model="effects.playback.reverse" class="checkbox" />
                            反转播放
                        </label>
                    </div>
                    <div class="control-item">
                        <label class="control-label">速度</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.playback.speed" min="-100" max="100" step="1"
                                class="slider" />
                            <span class="value">{{ effects.playback.speed }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">音调</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.playback.pitch" min="-100" max="100" step="1"
                                class="slider" />
                            <span class="value">{{ effects.playback.pitch }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">频率</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.playback.freq" min="0" max="2" step="0.1"
                                class="slider" />
                            <span class="value">{{ effects.playback.freq }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">音量</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.playback.volume" min="0" max="3000" step="10"
                                class="slider" />
                            <span class="value">{{ effects.playback.volume }}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 均衡器 -->
            <div class="effect-section">
                <h3 class="section-title">
                    均衡器
                    <span class="section-subtitle">提升/削减特定频率以实现音调平衡。</span>
                </h3>
                <div class="controls-group">
                    <div class="control-item">
                        <label class="control-label">
                            <input type="checkbox" v-model="effects.equalizer.enable" class="checkbox" />
                            启用
                        </label>
                    </div>
                    <div class="eq-container">
                        <div class="eq-bands">
                            <div v-for="(band, index) in effects.equalizer.bands" :key="band.freq" class="eq-band">
                                <div class="eq-slider-wrapper">
                                    <input type="range" v-model="band.gain" min="-20" max="20" step="0.5"
                                        class="eq-slider" :style="{
                                            background: `linear-gradient(to top, rgba(var(--primary-color), 0.3) 0%, rgba(var(--primary-color), 0.3) ${(band.gain + 20) * 2.5}%, rgba(var(--outline-color), 0.2) ${(band.gain + 20) * 2.5}%, rgba(var(--outline-color), 0.2) 100%)`
                                        }" />
                                </div>
                                <div class="eq-label">
                                    <span class="eq-freq">{{ band.freq }}</span>
                                    <span class="eq-gain">{{ band.gain > 0 ? '+' : '' }}{{ band.gain }}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 相位器 -->
            <div class="effect-section">
                <h3 class="section-title">
                    相位器
                    <span class="section-subtitle">创建旋转效果，使用反馈和速率。</span>
                </h3>
                <div class="controls-group">
                    <div class="control-item">
                        <label class="control-label">
                            <input type="checkbox" v-model="effects.phaser.enable" class="checkbox" />
                            启用
                        </label>
                    </div>
                    <div class="control-item">
                        <label class="control-label">干信号</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.phaser.dry" min="0" max="1000" step="1"
                                class="slider" />
                            <span class="value">{{ effects.phaser.dry }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">湿信号</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.phaser.wet" min="-500" max="500" step="1"
                                class="slider" />
                            <span class="value">{{ effects.phaser.wet }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">反馈</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.phaser.feedback" min="0" max="1000" step="1"
                                class="slider" />
                            <span class="value">{{ effects.phaser.feedback }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">比率</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.phaser.rate" min="0" max="10" step="0.1"
                                class="slider" />
                            <span class="value">{{ effects.phaser.rate }}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 混响 -->
            <div class="effect-section">
                <h3 class="section-title">
                    混响
                    <span class="section-subtitle">增加空间深度，使用混合（湿/干平衡）和时间。</span>
                </h3>
                <div class="controls-group">
                    <div class="control-item">
                        <label class="control-label">
                            <input type="checkbox" v-model="effects.reverb.enable" class="checkbox" />
                            启用
                        </label>
                    </div>
                    <div class="control-item">
                        <label class="control-label">混合</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.reverb.mix" min="0" max="100" step="1"
                                class="slider" />
                            <span class="value">{{ effects.reverb.mix }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">时间</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.reverb.time" min="0" max="1000" step="10"
                                class="slider" />
                            <span class="value">{{ effects.reverb.time }}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 回声 -->
            <div class="effect-section">
                <h3 class="section-title">
                    回声
                    <span class="section-subtitle">生成带有反馈和延迟时间的重复延迟。</span>
                </h3>
                <div class="controls-group">
                    <div class="control-item">
                        <label class="control-label">
                            <input type="checkbox" v-model="effects.echo.enable" class="checkbox" />
                            启用
                        </label>
                    </div>
                    <div class="control-item">
                        <label class="control-label">Feedback</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.echo.feedback" min="0" max="100" step="1"
                                class="slider" />
                            <span class="value">{{ effects.echo.feedback }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">Delay</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.echo.delay" min="0" max="1000" step="10"
                                class="slider" />
                            <span class="value">{{ effects.echo.delay }}</span>
                        </div>
                    </div>
                    <div class="control-item">
                        <label class="control-label">Mix</label>
                        <div class="slider-container">
                            <input type="range" v-model="effects.echo.mix" min="0" max="100" step="1" class="slider" />
                            <span class="value">{{ effects.echo.mix }}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- 重置按钮 -->
        <div class="reset-section">
            <button @click="resetToDefaults" class="reset-btn">
                <Icon src="/assets/restore.svg" size="sm" />
                Reset to Defaults
            </button>
        </div>
    </div>
</template>

<script setup>
import { ref, reactive, watch, inject } from 'vue'
import Icon from './Icon.vue'
const currentSong = inject('currentSong')
const emit = defineEmits(['effects-change'])

const currentPreset = ref('original')
const viewMode = ref('custom')

// 音效参数
const effects = reactive({
    playback: {
        reverse: false,
        speed: 0,
        pitch: 0,
        freq: 1,
        volume: 2000
    },
    equalizer: {
        enable: true,
        bands: [
            { freq: 'Gain', gain: 0 },
            { freq: '100', gain: 0 },
            { freq: '200', gain: 0 },
            { freq: '400', gain: 0 },
            { freq: '600', gain: 0 },
            { freq: '1K', gain: 0 },
            { freq: '3K', gain: 0 },
            { freq: '6K', gain: 0 },
            { freq: '12K', gain: 0 },
            { freq: '14K', gain: 0 }
        ]
    },
    phaser: {
        enable: false,
        dry: 999,
        wet: -101,
        feedback: 200,
        rate: 2
    },
    reverb: {
        enable: false,
        mix: 14,
        time: 300
    },
    echo: {
        enable: false,
        feedback: 0,
        delay: 0,
        mix: 0
    }
})

// 预设配置
const presets = {
    original: {
        playback: { reverse: false, speed: 0, pitch: 0, freq: 1, volume: 2000 },
        equalizer: { enable: true, bands: effects.equalizer.bands.map(b => ({ ...b, gain: 0 })) },
        phaser: { enable: false, dry: 999, wet: -101, feedback: 200, rate: 2 },
        reverb: { enable: false, mix: 14, time: 300 },
        echo: { enable: false, feedback: 0, delay: 0, mix: 0 }
    },
    rock: {
        playback: { reverse: false, speed: 0, pitch: 0, freq: 1, volume: 2200 },
        equalizer: {
            enable: true,
            bands: [
                { freq: 'Gain', gain: 0 },
                { freq: '100', gain: 3 },
                { freq: '200', gain: 2 },
                { freq: '400', gain: -1 },
                { freq: '600', gain: 1 },
                { freq: '1K', gain: 2 },
                { freq: '3K', gain: 4 },
                { freq: '6K', gain: 3 },
                { freq: '12K', gain: 2 },
                { freq: '14K', gain: 1 }
            ]
        },
        phaser: { enable: false, dry: 999, wet: -101, feedback: 200, rate: 2 },
        reverb: { enable: true, mix: 25, time: 400 },
        echo: { enable: false, feedback: 0, delay: 0, mix: 0 }
    },
    electronic: {
        playback: { reverse: false, speed: 0, pitch: 0, freq: 1.2, volume: 2400 },
        equalizer: {
            enable: true,
            bands: [
                { freq: 'Gain', gain: 2 },
                { freq: '100', gain: 4 },
                { freq: '200', gain: 2 },
                { freq: '400', gain: 0 },
                { freq: '600', gain: 1 },
                { freq: '1K', gain: -1 },
                { freq: '3K', gain: 2 },
                { freq: '6K', gain: 5 },
                { freq: '12K', gain: 6 },
                { freq: '14K', gain: 4 }
            ]
        },
        phaser: { enable: true, dry: 800, wet: 200, feedback: 400, rate: 3.5 },
        reverb: { enable: true, mix: 35, time: 500 },
        echo: { enable: true, feedback: 30, delay: 250, mix: 20 }
    }
}

// 监听预设变化
watch(currentPreset, (newPreset) => {
    if (presets[newPreset]) {
        Object.assign(effects, presets[newPreset])
    }
})

// 监听音效参数变化
watch(effects, () => {
    emit('effects-change', { ...effects })
}, { deep: true })

// 重置到默认值
const resetToDefaults = () => {
    currentPreset.value = 'original'
    Object.assign(effects, presets.original)
}
</script>

<style scoped>
.sound-effects-page {
    padding: 20px 50px;
    height: 100%;
    overflow-y: auto;
    background: #1e1e1e;
}

/* 页面标题 */
.page-header {
    margin-bottom: 24px;
}

.header-content {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
}

.title-section {
    flex: 1;
}

.app-title {
    font-size: 12px;
    font-weight: 600;
    color: rgba(var(--text-color), 0.6);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
}

.page-title {
    font-size: 48px;
    font-weight: 700;
    color: rgb(var(--text-color));
    margin-bottom: 12px;
}

.page-description {
    font-size: 16px;
    color: rgba(var(--text-color), 0.7);
    line-height: 1.5;
}

.preset-controls {
    display: flex;
    gap: 12px;
    align-items: center;
}

.preset-select,
.view-select {
    background: rgba(var(--surface-color), 0.1);
    border: 1px solid rgba(var(--outline-color), 0.2);
    border-radius: 8px;
    padding: 8px 12px;
    color: rgb(var(--text-color));
    font-size: 14px;
    cursor: pointer;
    min-width: 120px;
}

.preset-select:focus,
.view-select:focus {
    outline: none;
    border-color: rgba(var(--primary-color), 0.3);
}

/* 警告横幅 */
.warning-banner {
    background: rgba(var(--primary-color), 0.1);
    /* border: 1px solid rgba(255, 152, 0, 0.3); */
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 32px;
    display: flex;
    align-items: center;
    gap: 12px;
    color: rgb(var(--primary-color));
    /* color: #ff9800; */
    font-size: 14px;
}

/* 音效网格 */
.effects-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 32px;
    margin-bottom: 32px;
}

.effect-section {
    background: rgba(var(--surface-color), 0.05);
    border: 1px solid rgba(var(--outline-color), 0.1);
    border-radius: 12px;
    padding: 24px;
}

.section-title {
    font-size: 20px;
    font-weight: 600;
    color: rgb(var(--text-color));
    margin-bottom: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.section-subtitle {
    font-size: 14px;
    font-weight: 400;
    color: rgba(var(--text-color), 0.6);
}

.controls-group {
    margin-top: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.control-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.control-label {
    font-size: 14px;
    font-weight: 500;
    color: rgb(var(--text-color));
    display: flex;
    align-items: center;
    gap: 8px;
}

.checkbox {
    width: 16px;
    height: 16px;
    accent-color: rgba(var(--primary-color), 0.3);
}

.slider-container {
    display: flex;
    align-items: center;
    gap: 12px;
}

.slider {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: rgba(var(--outline-color), 0.2);
    outline: none;
    appearance: none;
    cursor: pointer;
}

.slider::-webkit-slider-thumb {
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: rgba(var(--primary-color), 0.3);
    cursor: pointer;
    border: 2px solid white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: rgba(var(--primary-color), 0.3);
    cursor: pointer;
    border: 2px solid white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.value {
    font-size: 14px;
    font-weight: 500;
    color: rgb(var(--text-color));
    min-width: 40px;
    text-align: right;
    font-variant-numeric: tabular-nums;
}

/* 均衡器 */
.eq-container {
    margin-top: 16px;
}

.eq-bands {
    display: flex;
    gap: 12px;
    align-items: end;
    justify-content: space-between;
}

.eq-band {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    flex: 1;
}

.eq-slider-wrapper {
    height: 120px;
    display: flex;
    align-items: center;
}

.eq-slider {
    width: 6px;
    height: 120px;
    border-radius: 3px;
    outline: none;
    appearance: none;
    cursor: pointer;
    writing-mode: bt-lr;
    -webkit-appearance: slider-vertical;
}

.eq-slider::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: rgba(var(--primary-color), 0.3);
    cursor: pointer;
    border: 1px solid white;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
}

.eq-label {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    text-align: center;
}

.eq-freq {
    font-size: 12px;
    color: rgba(var(--text-color), 0.7);
}

.eq-gain {
    font-size: 11px;
    color: rgba(var(--primary-color), 0.3);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
}

/* 重置按钮 */
.reset-section {
    text-align: center;
    padding-top: 16px;
    border-top: 1px solid rgba(var(--outline-color), 0.1);
}

.reset-btn {
    background: rgba(var(--primary-color), 0.1);
    border: 1px solid rgba(var(--primary-color), 0.3);
    border-radius: 8px;
    padding: 12px 24px;
    color: rgba(var(--primary-color), 0.3);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 auto;
}

.reset-btn:hover {
    background: rgba(var(--primary-color), 0.2);
    transform: translateY(-1px);
}

/* 滚动条样式 */
.sound-effects-page::-webkit-scrollbar {
    width: 4px;
}

.sound-effects-page::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
}

.sound-effects-page::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.sound-effects-page::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
}

/* 响应式设计 */
@media (max-width: 1200px) {
    .effects-grid {
        grid-template-columns: 1fr;
        gap: 24px;
    }
}

@media (max-width: 768px) {
    .sound-effects-page {
        padding: 20px 24px;
    }

    .page-title {
        font-size: 36px;
    }

    .header-content {
        flex-direction: column;
        gap: 20px;
    }

    .preset-controls {
        align-self: flex-start;
    }

    .eq-bands {
        gap: 8px;
    }

    .eq-slider-wrapper {
        height: 80px;
    }

    .eq-slider {
        height: 80px;
    }
}
</style>
