<template>
    <PageLayout>
        <template #header>
            <div class="music-banner">
                <div class="image-container">
                    <MotionTransition variant="banner">
                        <img :key="currentSong.cover" class="background-image" :src="currentSong.cover"
                            referrerpolicy="no-referrer">
                    </MotionTransition>
                </div>
                <div class="banner-content">
                    <div class="title">{{ t('app.name') }}</div>
                    <h2 class="library-title">{{ t('effects.title') }}</h2>
                    <div class="description">{{ t('effects.bannerDescription') }}</div>
                </div>
            </div>
        </template>

        <div class="sound-effects-page">
            <div v-if="effectsRuntime.state.loading" class="settings-note">{{ t('effects.loading') }}</div>
            <section v-else class="settings-row effects-shell">
                <div v-if="!effectsRuntime.state.catalog.length" class="settings-note">{{ t('effects.empty') }}</div>
                <HorizontalMaskScroll class="effect-tabs" :gap="8" padding="0 2px">
                    <button type="button" class="effect-tab" :class="{ active: selectedTab === 'playback' }"
                        :aria-pressed="selectedTab === 'playback'" @click="selectedTab = 'playback'">
                        {{ t('effects.playbackTab') }}
                    </button>
                    <button v-for="group in effectGroups" :key="group.key" type="button" class="effect-tab"
                        :class="{ active: selectedTab === group.key }" :aria-pressed="selectedTab === group.key"
                        @click="selectedTab = group.key">
                        <span>{{ categoryLabel(group.key) }}</span><span v-if="group.enabled" class="tab-check"
                            aria-label="enabled">✓</span>
                    </button>
                </HorizontalMaskScroll>

                <div class="effect-content-grid" :style="contentGridStyle">
                    <main class="tab-panel">
                        <template v-if="selectedTab === 'playback'">
                            <div class="education-copy">
                                <h2>{{ t('effects.playbackTitle') }}</h2>
                                <p>{{ t('effects.playbackDescription') }}</p>
                            </div>
                            <div class="playback-grid">
                                <div class="control-card">
                                    <div class="control-heading"><span>{{ t('effects.volume') }}</span><strong>{{
                                        Math.round(settingsStore.state.volume) }}%</strong></div>
                                    <p>{{ t('effects.volumeHint') }}</p>
                                    <RangeSlider :model-value="settingsStore.state.volume" :min="0" :max="100" :step="1"
                                        :aria-label="t('effects.volume')"
                                        :aria-value-text="`${Math.round(settingsStore.state.volume)}%`"
                                        @update:model-value="handleVolumeChange" />
                                </div>
                                <div class="control-card">
                                    <div class="control-heading"><span>{{ t('effects.speed') }}</span><strong>{{
                                        playback.speed }}%</strong></div>
                                    <p>{{ t('effects.speedHint') }}</p>
                                    <RangeSlider :model-value="playback.speed" :min="-95" :max="1000" :step="1"
                                        :aria-label="t('effects.speed')" :aria-value-text="`${playback.speed}%`"
                                        @update:model-value="updatePlayback({ speed: $event })" />
                                </div>
                                <div class="control-card">
                                    <div class="control-heading"><span>{{ t('effects.reverse') }}</span><strong>{{
                                        playback.reverse ? t('effects.on') : t('effects.off') }}</strong></div>
                                    <p>{{ t('effects.reverseHint') }}</p>
                                    <label class="toggle-row playback-toggle">
                                        <input type="checkbox" :checked="playback.reverse"
                                            :aria-label="t('effects.reverse')"
                                            @change="updatePlayback({ reverse: $event.target.checked })" />
                                        <span>{{ playback.reverse ? t('effects.on') : t('effects.off') }}</span>
                                    </label>
                                </div>
                                <div class="control-card">
                                    <div class="control-heading"><span>{{ t('effects.frequency') }}</span><strong>{{
                                        playback.frequencyRatio.toFixed(2) }}×</strong></div>
                                    <p>{{ t('effects.frequencyHint') }}</p>
                                    <RangeSlider :model-value="playback.frequencyRatio" :min="0.5" :max="2" :step="0.01"
                                        :aria-label="t('effects.frequency')"
                                        :aria-value-text="`${playback.frequencyRatio.toFixed(2)}x`"
                                        @update:model-value="updatePlayback({ frequencyRatio: $event })" />
                                </div>
                                <div class="control-card">
                                    <div class="control-heading"><span>{{ t('effects.pan') }}</span><strong>{{ panLabel
                                            }}</strong></div>
                                    <p>{{ t('effects.panHint') }}</p>
                                    <RangeSlider :model-value="playback.pan" :min="-1" :max="1" :step="0.01"
                                        :aria-label="t('effects.pan')" :aria-value-text="panLabel"
                                        @update:model-value="updatePlayback({ pan: $event })" />
                                </div>
                            </div>
                            <div v-if="playbackRuntime.state.error" class="settings-note error-note">{{
                                playbackRuntime.state.error }}</div>
                        </template>

                        <template v-else-if="selectedGroup">
                            <template v-if="selectedGroup.key === 'equalizer'">
                                <div class="education-copy">
                                    <h2>{{ t('effects.equalizerTitle') }}</h2>
                                    <p>{{ t('effects.equalizerDescription') }}</p>
                                </div>

                                <article class="effect-variant equalizer-variant">
                                    <div class="variant-heading">
                                        <div>
                                            <h3>{{ t('effects.equalizerGraphicTitle') }}</h3>
                                            <small>{{ t('effects.equalizerGraphicSubtitle') }}</small>
                                        </div>
                                        <div class="effect-toolbar">
                                            <label class="toggle-row"><input type="checkbox"
                                                    :checked="isEqualizerEnabled"
                                                    @change="handleEqualizerEnabledChange($event.target.checked)" /><span>{{
                                                        t('effects.enable') }}</span></label>
                                            <button type="button" class="text-button" @click="handleEqualizerReset">{{
                                                t('effects.reset') }}</button>
                                        </div>
                                    </div>

                                    <div class="equalizer-chart-card">
                                        <div class="equalizer-chart-heading">
                                            <span>{{ t('effects.equalizerGainAxis') }}</span>
                                            <strong>{{ t('effects.equalizerZeroDb') }}</strong>
                                        </div>
                                        <div class="equalizer-chart">
                                            <svg ref="equalizerGraph" viewBox="0 0 1000 280" role="img"
                                                :aria-label="t('effects.equalizerChartLabel')"
                                                preserveAspectRatio="none" @pointerdown="startEqualizerGraphDrag">
                                                <line v-for="gain in equalizerGridGains" :key="gain" x1="40"
                                                    :y1="equalizerChartY(gain)" x2="980" :y2="equalizerChartY(gain)"
                                                    :class="{ 'equalizer-zero-line': gain === 0 }"
                                                    class="equalizer-grid-line" />
                                                <line v-for="(_, index) in equalizerBands" :key="'grid-' + index"
                                                    :x1="equalizerChartX(index)" y1="20" :x2="equalizerChartX(index)"
                                                    y2="250" class="equalizer-grid-line vertical" />
                                                <polyline :points="equalizerCurvePoints" class="equalizer-curve" />
                                                <circle v-for="(band, index) in equalizerBands"
                                                    :key="'point-' + band.frequency" :cx="equalizerChartX(index)"
                                                    :cy="equalizerChartY(band.gain)" r="6" class="equalizer-point"
                                                    role="slider" tabindex="0" :aria-valuenow="band.gain"
                                                    aria-valuemin="-15" aria-valuemax="15"
                                                    :aria-label="`${band.frequency} Hz`"
                                                    @keydown="handleEqualizerPointKey(index, $event)" />
                                            </svg>
                                            <div class="equalizer-axis-labels">
                                                <span v-for="gain in equalizerGridGains" :key="gain">{{ gain > 0 ? '+' :
                                                    '' }}{{ gain
                                                    }} dB</span>
                                            </div>
                                        </div>
                                        <div class="equalizer-band-labels">
                                            <div v-for="band in equalizerBands" :key="band.frequency"
                                                class="equalizer-band-label">
                                                <strong>{{ equalizerFrequencyLabel(band.frequency) }}</strong>
                                                <output>{{ equalizerGainLabel(band.gain) }}</output>
                                            </div>
                                        </div>
                                        <p class="equalizer-drag-hint">{{ t('effects.equalizerDragHint') }}</p>
                                    </div>
                                </article>
                            </template>

                            <template v-else>
                                <div class="education-copy">
                                    <h2>{{ categoryLabel(selectedGroup.key) }}</h2>
                                    <p>{{ categoryDescription(selectedGroup.key) }}</p>
                                </div>

                                <div class="variant-list">
                                    <article v-for="descriptor in selectedGroup.descriptors" :key="descriptor.kind"
                                        class="effect-variant">
                                        <div class="variant-heading">
                                            <div>
                                                <h3>{{ effectLabel(descriptor) }}</h3>
                                                <small>{{ descriptor.kind }} · {{ familyLabel(descriptor.family)
                                                    }}</small>
                                            </div>
                                            <div class="effect-toolbar">
                                                <label class="toggle-row"><input type="checkbox"
                                                        :checked="isEnabled(descriptor.kind)"
                                                        @change="handleEnabledChange(descriptor.kind, $event.target.checked)" /><span>{{
                                                            t('effects.enable') }}</span></label>
                                                <button type="button" class="text-button"
                                                    @click="handleReset(descriptor.kind)">{{ t('effects.reset')
                                                    }}</button>
                                            </div>
                                        </div>

                                        <div class="parameter-grid">
                                            <template v-for="parameter in descriptor.parameters" :key="parameter.key">
                                                <div v-if="parameter.type === 'nodes'"
                                                    class="parameter-item nodes-item">
                                                    <div class="parameter-label"><span>{{ parameterLabel(parameter.key)
                                                            }}</span><small>{{ parameter.key }}</small></div>
                                                    <div class="nodes-editor">
                                                        <div v-for="(node, index) in parameterValue(descriptor, parameter).value"
                                                            :key="index" class="node-row">
                                                            <input class="parameter-input" type="number" step="0.01"
                                                                :value="node.pos"
                                                                @change="handleNodeChange(descriptor, index, 'pos', $event.target.value)" />
                                                            <input class="parameter-input" type="number" step="0.01"
                                                                :value="node.val"
                                                                @change="handleNodeChange(descriptor, index, 'val', $event.target.value)" />
                                                            <button type="button" class="icon-text-button"
                                                                @click="removeNode(descriptor, index)">−</button>
                                                        </div>
                                                        <button type="button" class="text-button"
                                                            @click="addNode(descriptor)">＋ {{ t('effects.addNode')
                                                            }}</button>
                                                    </div>
                                                </div>
                                                <div v-else class="parameter-item">
                                                    <div class="parameter-label"><span>{{ parameterLabel(parameter.key)
                                                            }}</span><small>{{ parameter.key }}</small></div>
                                                    <label v-if="parameter.type === 'boolean'"
                                                        class="toggle-row parameter-toggle"><input type="checkbox"
                                                            :checked="Boolean(parameterValue(descriptor, parameter).value)"
                                                            @change="handleParameterChange(descriptor, parameter, $event.target.checked ? 1 : 0)" /><span>{{
                                                                parameterValue(descriptor, parameter).value ?
                                                                    t('effects.on') :
                                                            t('effects.off') }}</span></label>
                                                    <input v-else-if="parameter.type === 'array'"
                                                        class="parameter-input" type="text"
                                                        :value="arrayText(parameterValue(descriptor, parameter).value)"
                                                        @change="handleArrayChange(descriptor, parameter, $event.target.value)" />
                                                    <select v-else-if="parameterOptions(parameter).length"
                                                        class="parameter-input"
                                                        :value="parameterValue(descriptor, parameter).value"
                                                        @change="handleParameterChange(descriptor, parameter, $event.target.value)">
                                                        <option v-for="option in parameterOptions(parameter)"
                                                            :key="option.value" :value="option.value">{{
                                                                optionLabel(parameter, option) }}</option>
                                                    </select>
                                                    <div v-else-if="isSliderParameter(parameter)"
                                                        class="parameter-slider-control">
                                                        <RangeSlider
                                                            :model-value="parameterSliderValue(descriptor, parameter)"
                                                            :min="parameter.min" :max="parameter.max"
                                                            :step="parameter.step || 0.01"
                                                            :aria-label="parameterLabel(parameter.key)"
                                                            :aria-value-text="parameterValueLabel(parameter, parameterValue(descriptor, parameter).value)"
                                                            @update:model-value="handleParameterChange(descriptor, parameter, $event)" />
                                                        <output class="parameter-value">{{
                                                            parameterValueLabel(parameter,
                                                            parameterValue(descriptor, parameter).value) }}</output>
                                                    </div>
                                                    <input v-else class="parameter-input" type="number"
                                                        :min="parameter.min ?? undefined"
                                                        :max="parameter.max ?? undefined"
                                                        :step="parameter.step || 'any'"
                                                        :value="parameterValue(descriptor, parameter).value"
                                                        @change="handleParameterChange(descriptor, parameter, $event.target.value)" />
                                                </div>
                                            </template>
                                        </div>
                                        <div v-if="effectsRuntime.state.errors[descriptor.kind]"
                                            class="settings-note error-note">{{
                                                effectsRuntime.state.errors[descriptor.kind] }}</div>
                                    </article>
                                </div>
                            </template>
                        </template>
                    </main>

                    <div ref="wikiSplitter" class="wiki-resize-handle" role="separator" tabindex="0"
                        :aria-label="t('effects.wikiResize')" :aria-valuenow="wikiWidth" @pointerdown="startWikiResize"
                        @keydown.left.prevent="adjustWikiWidth(-20)" @keydown.right.prevent="adjustWikiWidth(20)"></div>

                    <aside class="wiki-panel">
                        <div class="wiki-heading">
                            <InfoIcon size="13" />
                            <div style="font-size: 13px;">{{ t('effects.wiki') }}</div>
                        </div>
                        <template v-if="selectedTab === 'playback'">
                            <h3>{{ t('effects.playbackTitle') }}</h3>
                            <p>{{ t('effects.playbackWiki') }}</p>
                            <dl class="wiki-facts">
                                <div>
                                    <dt>{{ t('effects.volume') }}</dt>
                                    <dd>{{ t('effects.volumeWiki') }}</dd>
                                </div>
                                <div>
                                    <dt>{{ t('effects.speed') }}</dt>
                                    <dd>{{ t('effects.speedWiki') }}</dd>
                                </div>
                                <div>
                                    <dt>{{ t('effects.reverse') }}</dt>
                                    <dd>{{ t('effects.reverseWiki') }}</dd>
                                </div>
                                <div>
                                    <dt>{{ t('effects.frequency') }}</dt>
                                    <dd>{{ t('effects.frequencyWiki') }}</dd>
                                </div>
                                <div>
                                    <dt>{{ t('effects.pan') }}</dt>
                                    <dd>{{ t('effects.panWiki') }}</dd>
                                </div>
                            </dl>
                            <section v-if="wikiGlossary.length" class="wiki-variant">
                                <h4>{{ t('effects.wikiGlossary') }}</h4>
                                <dl class="wiki-facts">
                                    <div v-for="item in wikiGlossary" :key="item.titleKey">
                                        <dt>{{ t(item.titleKey) }}</dt>
                                        <dd>{{ t(item.descriptionKey) }}</dd>
                                    </div>
                                </dl>
                            </section>
                        </template>
                        <template v-else-if="selectedGroup">
                            <template v-if="selectedGroup.key === 'equalizer'">
                                <h3>{{ t('effects.equalizerTitle') }}</h3>
                                <p>{{ t('effects.equalizerWikiIntro') }}</p>
                                <dl class="wiki-facts">
                                    <div>
                                        <dt>{{ t('effects.equalizerWikiGainTitle') }}</dt>
                                        <dd>{{ t('effects.equalizerWikiGain') }}</dd>
                                    </div>
                                    <div>
                                        <dt>{{ t('effects.equalizerWikiZeroTitle') }}</dt>
                                        <dd>{{ t('effects.equalizerWikiZero') }}</dd>
                                    </div>
                                    <div>
                                        <dt>{{ t('effects.equalizerWikiBandsTitle') }}</dt>
                                        <dd>{{ t('effects.equalizerWikiBands') }}</dd>
                                    </div>
                                </dl>
                                <section class="wiki-variant">
                                    <h4>{{ t('effects.equalizerWikiUsageTitle') }}</h4>
                                    <p class="wiki-tip">{{ t('effects.equalizerWikiUsage') }}</p>
                                </section>
                            </template>
                            <template v-else>
                                <h3>{{ categoryLabel(selectedGroup.key) }}</h3>
                                <p>{{ categoryDescription(selectedGroup.key) }}</p>
                                <section v-for="descriptor in selectedGroup.descriptors" :key="descriptor.kind"
                                    class="wiki-variant">
                                    <h4>{{ effectLabel(descriptor) }} <small>{{ familyLabel(descriptor.family)
                                            }}</small>
                                    </h4>
                                    <p>{{ wikiPurpose(descriptor) }}</p>
                                    <p v-if="wikiQuickStart(descriptor)" class="wiki-tip">{{ wikiQuickStart(descriptor)
                                        }}
                                    </p>
                                    <dl class="wiki-facts">
                                        <div v-for="parameter in descriptor.parameters" :key="parameter.key">
                                            <dt>{{ parameterLabel(parameter.key) }}<small
                                                    v-if="wikiParameter(descriptor, parameter.key)?.range"> · {{
                                                        wikiParameter(descriptor, parameter.key).range }}</small><small
                                                    v-if="wikiParameter(descriptor, parameter.key)?.defaultValue"> · {{
                                                        wikiParameter(descriptor, parameter.key).defaultValue }}</small>
                                            </dt>
                                            <dd>{{ wikiParameter(descriptor, parameter.key).description }}</dd>
                                        </div>
                                    </dl>
                                </section>
                            </template>
                        </template>
                    </aside>
                </div>
            </section>
        </div>
    </PageLayout>
</template>

<script setup>
import { computed, inject, onBeforeUnmount, ref } from 'vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import MotionTransition from '@/components/ui/MotionTransition.vue'
import RangeSlider from '@/components/ui/RangeSlider.vue'
import HorizontalMaskScroll from '@/components/layout/HorizontalMaskScroll.vue'
import { useI18n } from '@/i18n/index.js'
import { useAppSettingsStore } from '@/stores/appSettingsStore.js'
import { getEffectWiki, getWikiGlossary } from '@/services/effectsWiki.js'
import { InfoIcon } from '@lucide/vue'

const props = defineProps({
    effectsRuntime: { type: Object, required: true },
    playbackRuntime: { type: Object, required: true }
})
const { t } = useI18n()
const currentSong = inject('currentSong')
const settingsStore = useAppSettingsStore()
const selectedTab = ref('playback')
const wikiWidth = ref(340)
const wikiSplitter = ref(null)
const equalizerGraph = ref(null)
const wikiGlossary = getWikiGlossary()
let isResizingWiki = false
let isDraggingEqualizer = false
const playback = computed(() => settingsStore.state.playback)
const panLabel = computed(() => playback.value.pan < -0.01 ? `${Math.round(Math.abs(playback.value.pan) * 100)}% ${t('effects.left')}` : playback.value.pan > 0.01 ? `${Math.round(playback.value.pan * 100)}% ${t('effects.right')}` : t('effects.center'))
const equalizerBands = computed(() => {
    const saved = effectConfig('bassFx.peakeq').bands || effectConfig('dx8.parameq').bands
    return Array.isArray(saved) && saved.length ? saved : (props.effectsRuntime.state.equalizerBands || [])
})
const isEqualizerEnabled = computed(() => isEnabled('bassFx.peakeq') || isEnabled('dx8.parameq'))
const equalizerGridGains = [15, 7.5, 0, -7.5, -15]
const equalizerChartX = (index) => {
    const count = Math.max(1, equalizerBands.value.length - 1)
    return 40 + (940 * index / count)
}
const equalizerChartY = (gain) => 135 - (Math.max(-15, Math.min(15, Number(gain) || 0)) * (115 / 15))
const equalizerCurvePoints = computed(() => equalizerBands.value.map((band, index) => `${equalizerChartX(index)},${equalizerChartY(band.gain)}`).join(' '))
const equalizerGainLabel = (gain) => `${Number(gain) > 0 ? '+' : ''}${Number(gain || 0).toFixed(1)} dB`
const equalizerFrequencyLabel = (frequency) => Number(frequency) >= 1000 ? `${Number(frequency) / 1000}k` : `${frequency}`
const equalizerIndexFromPointer = (event) => {
    const graph = equalizerGraph.value
    if (!graph || !equalizerBands.value.length) return 0
    const bounds = graph.getBoundingClientRect()
    const x = Math.max(0, Math.min(1, (event.clientX - bounds.left) / bounds.width))
    return Math.max(0, Math.min(equalizerBands.value.length - 1,
        Math.round(x * (equalizerBands.value.length - 1))))
}
const equalizerGainFromPointer = (event) => {
    const graph = equalizerGraph.value
    if (!graph) return 0
    const bounds = graph.getBoundingClientRect()
    const y = Math.max(0, Math.min(1, (event.clientY - bounds.top) / bounds.height))
    return Math.round((15 - y * 30) * 2) / 2
}
const updateEqualizerGraphDrag = (event) => {
    if (!isDraggingEqualizer) return
    void props.effectsRuntime.setEqualizerBand(equalizerIndexFromPointer(event), equalizerGainFromPointer(event))
}
const stopEqualizerGraphDrag = () => {
    if (!isDraggingEqualizer) return
    isDraggingEqualizer = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    window.removeEventListener('pointermove', updateEqualizerGraphDrag)
    window.removeEventListener('pointerup', stopEqualizerGraphDrag)
    window.removeEventListener('pointercancel', stopEqualizerGraphDrag)
}
const startEqualizerGraphDrag = (event) => {
    if (event.button !== 0) return
    event.preventDefault()
    isDraggingEqualizer = true
    document.body.style.cursor = 'crosshair'
    document.body.style.userSelect = 'none'
    updateEqualizerGraphDrag(event)
    window.addEventListener('pointermove', updateEqualizerGraphDrag)
    window.addEventListener('pointerup', stopEqualizerGraphDrag)
    window.addEventListener('pointercancel', stopEqualizerGraphDrag)
}
const handleEqualizerPointKey = (index, event) => {
    if (!['ArrowUp', 'ArrowDown', 'Home', 'End'].includes(event.key)) return
    event.preventDefault()
    const current = Number(equalizerBands.value[index]?.gain) || 0
    const next = event.key === 'Home' ? -15 : event.key === 'End' ? 15 : current + (event.key === 'ArrowUp' ? 0.5 : -0.5)
    handleEqualizerBandChange(index, Math.max(-15, Math.min(15, next)))
}

const categoryFor = (kind) => {
    if (kind === 'dx8.chorus' || kind === 'bassFx.chorus') return 'chorus'
    if (kind === 'dx8.compressor' || kind === 'bassFx.compressor' || kind === 'bassFx.compressor2') return 'compressor'
    if (kind === 'dx8.distortion' || kind === 'bassFx.distortion') return 'distortion'
    if (kind === 'dx8.echo' || kind === 'bassFx.echo' || kind === 'bassFx.echo2' || kind === 'bassFx.echo3' || kind === 'bassFx.echo4') return 'echo'
    if (kind === 'dx8.flanger' || kind === 'bassFx.flanger') return 'flanger'
    if (kind === 'dx8.i3dl2reverb' || kind === 'dx8.reverb' || kind === 'bassFx.reverb' || kind === 'bassFx.freeverb') return 'reverb'
    if (kind === 'dx8.parameq' || kind === 'bassFx.peakeq') return 'equalizer'
    if (kind === 'bassFx.lowpassfilter' || kind === 'bassFx.allpassfilter' || kind === 'bassFx.biquadfilter') return 'filters'
    if (kind === 'volume' || kind === 'bassFx.volume' || kind === 'bassFx.volumeenvelope') return 'volumeFx'
    if (kind === 'bassFx.pitchshift') return 'pitchshift'
    if (kind === 'dx8.gargle' || kind === 'bassFx.rotate' || kind === 'bassFx.autowah' || kind === 'bassFx.phaser') return 'modulation'
    if (kind === 'bassFx.mix') return 'channel'
    if (kind === 'bassFx.damp') return 'dynamics'
    return 'other'
}
const effectConfig = (kind) => settingsStore.state.effects?.[kind] || {}
const isEnabled = (kind) => Boolean(effectConfig(kind).enabled)
const effectGroups = computed(() => {
    const groups = new Map()
    for (const descriptor of props.effectsRuntime.state.catalog) {
        const key = categoryFor(descriptor.kind)
        if (!groups.has(key)) groups.set(key, { key, descriptors: [], enabled: false })
        const group = groups.get(key)
        group.descriptors.push(descriptor)
        group.enabled = group.enabled || isEnabled(descriptor.kind)
    }
    return [...groups.values()]
})
const selectedGroup = computed(() => effectGroups.value.find((group) => group.key === selectedTab.value))
const contentGridStyle = computed(() => ({ '--wiki-width': `${wikiWidth.value}px` }))
const translated = (key, fallback) => { const value = t(key); return value.startsWith('effects.') ? fallback : value }
const effectLabel = (descriptor) => translated(`effects.names.${descriptor.kind}`, descriptor.kind)
const categoryLabel = (key) => translated(`effects.categories.${key}`, key)
const categoryDescription = (key) => translated(`effects.categoryDescriptions.${key}`, t('effects.bassFxEducation'))
const variantDescription = (descriptor) => translated(`effects.variantDescriptions.${descriptor.kind}`, categoryDescription(categoryFor(descriptor.kind)))
const parameterLabel = (key) => translated(`effects.parameterNames.${key}`, key)
const parameterDescription = (key) => translated(`effects.parameterDescriptions.${key}`, t('effects.parameterWikiFallback'))
const familyLabel = (family) => family === 'dx8' ? t('effects.groupDx8') : family === 'bassFx' ? t('effects.groupBassFx') : t('effects.groupVolume')
const wikiFor = (descriptor) => getEffectWiki(descriptor.kind)
const wikiPurpose = (descriptor) => {
    const wiki = wikiFor(descriptor)
    return wiki ? translated(wiki.purposeKey, variantDescription(descriptor)) : variantDescription(descriptor)
}
const wikiQuickStart = (descriptor) => {
    const wiki = wikiFor(descriptor)
    return wiki ? translated(wiki.quickStartKey, '') : ''
}
const wikiParameter = (descriptor, key) => ({
    ...(wikiFor(descriptor)?.parameters[key] || {}),
    description: parameterDescription(key)
})

const defaultValue = (parameter) => {
    if (parameter.type === 'nodes') return []
    if (parameter.type === 'array') return [-1]
    if (parameter.default !== null && parameter.default !== undefined) return parameter.default
    if (parameter.type === 'boolean') return 0
    if (parameter.key === 'lChannel') return -1
    if (parameter.key === 'lNodeCount') return 0
    return 0
}
const parameterValue = (descriptor, parameter) => ({ value: effectConfig(descriptor.kind).parameters?.[parameter.key] ?? defaultValue(parameter) })
const parameterOptions = (parameter) => ({
    lWaveform: [0, 1], dwWaveShape: [0, 1], lPhase: [0, 1, 2, 3, 4], lPanDelay: [0, 1],
    lCurve: [0, 1, 2], lChannel: [-1, 0, 1], lFilter: [0, 1, 2, 3, 4, 5, 6], lMode: [0, 1]
}[parameter.key] || []).map((value) => ({ value }))
const optionLabel = (parameter, option) => translated(`effects.options.${parameter.key}.${option.value}`, String(option.value))
const isSliderParameter = (parameter) => {
    if (!['number', 'integer'].includes(parameter.type) || parameterOptions(parameter).length) return false
    return Number.isFinite(Number(parameter.min)) && Number.isFinite(Number(parameter.max))
}
const numericValue = (parameter, value) => {
    const next = Number(value)
    if (!Number.isFinite(next)) return defaultValue(parameter)
    const min = parameter.min !== null && parameter.min !== undefined ? Math.max(parameter.min, next) : next
    return parameter.max !== null && parameter.max !== undefined ? Math.min(parameter.max, min) : min
}
const parameterSliderValue = (descriptor, parameter) => numericValue(parameter, parameterValue(descriptor, parameter).value)
const parameterValueLabel = (parameter, value) => {
    const next = Number(value)
    if (!Number.isFinite(next)) return '—'
    if (parameter.type === 'integer') return String(Math.round(next))
    const step = Number(parameter.step)
    const decimals = Number.isFinite(step) && step > 0 && step < 1
        ? Math.min(3, String(step).split('.')[1]?.length || 2)
        : 0
    return next.toFixed(decimals).replace(/\.0+$/, '').replace(/(\.\d*?)0+$/, '$1')
}
const fullParameters = (descriptor) => Object.fromEntries(descriptor.parameters.map((parameter) => [parameter.key, parameterValue(descriptor, parameter).value]))
const updateParameters = (descriptor, parameters) => void props.effectsRuntime.setEffect(descriptor.kind, { parameters })
const handleParameterChange = (descriptor, parameter, value) => {
    const parameters = fullParameters(descriptor)
    parameters[parameter.key] = parameter.type === 'integer' ? Math.round(numericValue(parameter, value)) : numericValue(parameter, value)
    updateParameters(descriptor, parameters)
}
const arrayText = (value) => Array.isArray(value) ? value.join(', ') : ''
const handleArrayChange = (descriptor, parameter, value) => {
    const parameters = fullParameters(descriptor)
    parameters[parameter.key] = value.split(',').map((entry) => Number(entry.trim())).filter(Number.isFinite).map(Math.round)
    if (!parameters[parameter.key].length) parameters[parameter.key] = [-1]
    updateParameters(descriptor, parameters)
}
const handleNodeChange = (descriptor, index, field, value) => {
    const parameters = fullParameters(descriptor)
    parameters.pNodes = (parameters.pNodes || []).map((node, nodeIndex) => nodeIndex === index ? { ...node, [field]: numericValue({ min: 0 }, value) } : node)
    parameters.lNodeCount = parameters.pNodes.length
    updateParameters(descriptor, parameters)
}
const addNode = (descriptor) => { const parameters = fullParameters(descriptor); parameters.pNodes = [...(parameters.pNodes || []), { pos: 0, val: 1 }]; parameters.lNodeCount = parameters.pNodes.length; updateParameters(descriptor, parameters) }
const removeNode = (descriptor, index) => { const parameters = fullParameters(descriptor); parameters.pNodes = (parameters.pNodes || []).filter((_, nodeIndex) => nodeIndex !== index); parameters.lNodeCount = parameters.pNodes.length; updateParameters(descriptor, parameters) }
const handleEnabledChange = (kind, enabled) => void props.effectsRuntime.setEnabled(kind, enabled)
const handleReset = (kind) => void props.effectsRuntime.resetEffect(kind)
const handleVolumeChange = (value) => { settingsStore.updateVolume(value); void props.effectsRuntime.setVolume(props.effectsRuntime.state.activeChannelId, value) }
const updatePlayback = (patch) => void props.playbackRuntime.update(patch)
const handleEqualizerBandChange = (index, value) => void props.effectsRuntime.setEqualizerBand(index, value)
const handleEqualizerEnabledChange = (enabled) => void props.effectsRuntime.setEqualizerEnabled(enabled)
const handleEqualizerReset = () => void props.effectsRuntime.resetEqualizer()

const updateWikiWidth = (event) => {
    if (!isResizingWiki || !wikiSplitter.value) return
    const grid = wikiSplitter.value.parentElement
    if (!grid) return
    const bounds = grid.getBoundingClientRect()
    const next = bounds.right - event.clientX
    wikiWidth.value = Math.round(Math.max(260, Math.min(560, next)))
}
const stopWikiResize = () => {
    if (!isResizingWiki) return
    isResizingWiki = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    window.removeEventListener('pointermove', updateWikiWidth)
    window.removeEventListener('pointerup', stopWikiResize)
    window.removeEventListener('pointercancel', stopWikiResize)
}
const startWikiResize = (event) => {
    if (window.matchMedia('(max-width: 980px)').matches) return
    event.preventDefault()
    isResizingWiki = true
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('pointermove', updateWikiWidth)
    window.addEventListener('pointerup', stopWikiResize)
    window.addEventListener('pointercancel', stopWikiResize)
}
const adjustWikiWidth = (amount) => {
    wikiWidth.value = Math.round(Math.max(260, Math.min(560, wikiWidth.value + amount)))
}
onBeforeUnmount(() => {
    stopWikiResize()
    stopEqualizerGraphDrag()
})
</script>

<style scoped>
.sound-effects-page {
    display: grid;
    gap: 0;
    width: 100%;
}

.settings-row {
    display: grid;
    gap: 18px;
    width: 100%;
    padding: 14px 0 16px;
    border-top: 1px solid rgba(var(--outline-color), .12);
    border-bottom: 1px solid rgba(var(--outline-color), .12);
}

.effects-shell {
    gap: 20px;
}

.effect-tabs {
    height: 38px;
}

.effect-tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 auto;
    height: 30px;
    padding: 0 11px;
    border: 1px solid rgba(var(--outline-color), .2);
    border-radius: 6px;
    color: rgba(var(--text-color), .62);
    background: transparent;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: color 160ms ease, background-color 160ms ease, border-color 160ms ease;
}

.effect-tab:hover,
.effect-tab.active {
    border-color: rgba(var(--primary-color), .5);
    color: rgb(var(--text-color));
    background: rgba(var(--primary-color), .14);
}

.tab-check {
    color: rgb(var(--primary-color));
    font-weight: 700;
}

.effect-content-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 8px minmax(260px, var(--wiki-width, 340px));
    column-gap: 0;
    align-items: start;
}

.tab-panel {
    display: grid;
    gap: 22px;
    min-width: 0;
    padding-right: 22px;
}

.education-copy {
    display: grid;
    gap: 8px;
    max-width: 860px;
}

.education-copy h2 {
    margin: 0;
    font-size: 15px;
    line-height: 1.25;
}

.education-copy p,
.control-card p,
.wiki-panel p {
    margin: 0;
    color: rgba(var(--text-color), .58);
    font-size: 12px;
    line-height: 1.7;
}

.playback-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px 22px;
}

.control-card {
    display: grid;
    gap: 10px;
    padding: 14px 0;
    border-top: 1px solid rgba(var(--outline-color), .12);
    border-bottom: 1px solid rgba(var(--outline-color), .12);
}

.control-heading,
.variant-heading,
.wiki-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
}

.control-heading strong {
    color: rgb(var(--text-color));
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
}

.variant-list {
    display: grid;
    gap: 22px;
}

.effect-variant {
    display: grid;
    gap: 16px;
    padding: 0 0 20px;
    border-bottom: 1px solid rgba(var(--outline-color), .12);
}

.variant-heading {
    align-items: flex-start;
}

.variant-heading h3 {
    margin: 0 0 5px;
    font-size: 14px;
}

.variant-heading small,
.wiki-variant h4 small {
    color: rgba(var(--text-color), .42);
    font-size: 10px;
    font-weight: 400;
}

.effect-toolbar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 14px;
}

.toggle-row {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: rgba(var(--text-color), .72);
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
}

.toggle-row input {
    width: 15px;
    height: 15px;
    accent-color: rgb(var(--primary-color));
}

.text-button,
.icon-text-button {
    padding: 4px 0;
    border: 0;
    color: rgba(var(--text-color), .62);
    background: transparent;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
}

.text-button:hover,
.icon-text-button:hover {
    color: rgb(var(--text-color));
}

.parameter-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 12px 24px;
}

.parameter-item {
    display: grid;
    grid-template-columns: minmax(100px, 1fr) minmax(90px, 1fr);
    align-items: center;
    gap: 12px;
    min-height: 34px;
}

.nodes-item {
    grid-column: 1 / -1;
    align-items: start;
}

.parameter-label {
    display: grid;
    gap: 2px;
    color: rgba(var(--text-color), .72);
    font-size: 12px;
}

.parameter-label small {
    color: rgba(var(--text-color), .42);
    font-size: 10px;
}

.parameter-input {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    padding: 6px 8px;
    border: 0;
    border-bottom: 1px solid rgba(var(--outline-color), .28);
    outline: 0;
    color: rgb(var(--text-color));
    background: transparent;
    font: inherit;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
}

.parameter-input:focus {
    border-bottom-color: rgb(var(--primary-color));
}

.parameter-slider-control {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
}

.parameter-slider-control .range-slider {
    min-width: 70px;
}

.parameter-value {
    min-width: 46px;
    color: rgba(var(--text-color), .72);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-align: right;
}

.equalizer-variant {
    gap: 18px;
}

.equalizer-chart-card {
    display: grid;
    gap: 14px;
    padding: 18px 0 4px;
}

.equalizer-chart-heading {
    display: flex;
    justify-content: space-between;
    color: rgba(var(--text-color), .52);
    font-size: 11px;
}

.equalizer-chart-heading strong {
    color: rgba(var(--text-color), .72);
    font-weight: 500;
}

.equalizer-chart {
    position: relative;
    min-height: 230px;
    padding-right: 48px;
}

.equalizer-chart svg {
    display: block;
    width: 100%;
    overflow: visible;
    cursor: crosshair;
    touch-action: none;
}

.equalizer-grid-line {
    stroke: rgba(var(--outline-color), .16);
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
}

.equalizer-grid-line.vertical {
    stroke: rgba(var(--outline-color), .1);
    stroke-dasharray: 2 6;
}

.equalizer-zero-line {
    stroke: rgba(var(--primary-color), .36);
    stroke-width: 1.5;
}

.equalizer-curve {
    fill: none;
    stroke: rgb(var(--primary-color));
    stroke-width: 3;
    stroke-linecap: round;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
}

.equalizer-point {
    fill: rgb(var(--primary-color));
    stroke: rgb(var(--text-color));
    stroke-width: 2;
    cursor: ns-resize;
    vector-effect: non-scaling-stroke;
}

.equalizer-axis-labels {
    position: absolute;
    top: 8px;
    right: 0;
    bottom: 8px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    color: rgba(var(--text-color), .45);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
}

.equalizer-band-labels {
    display: grid;
    grid-template-columns: repeat(10, minmax(42px, 1fr));
    gap: 8px;
    min-width: 580px;
}

.equalizer-band-label {
    display: grid;
    justify-items: center;
    gap: 4px;
    color: rgba(var(--text-color), .58);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
}

.equalizer-band-label strong {
    color: rgba(var(--text-color), .68);
    font-size: 11px;
    font-weight: 500;
}

.equalizer-band-label output {
    min-height: 14px;
    color: rgba(var(--text-color), .76);
    font-size: 10px;
}

.equalizer-drag-hint {
    margin: 0;
    color: rgba(var(--text-color), .45);
    font-size: 11px;
    text-align: center;
}

.parameter-toggle {
    justify-content: flex-end;
}

.nodes-editor {
    display: grid;
    gap: 8px;
}

.node-row {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 8px;
}

.wiki-resize-handle {
    position: relative;
    align-self: stretch;
    min-height: 220px;
    cursor: col-resize;
    touch-action: none;
}

.wiki-resize-handle::before {
    content: '';
    position: absolute;
    inset: 0 3px;
    border-left: 1px solid rgba(var(--outline-color), .14);
    border-right: 1px solid transparent;
    transition: border-color 160ms ease, background-color 160ms ease;
}

.wiki-resize-handle:hover::before,
.wiki-resize-handle:focus-visible::before {
    border-left-color: rgba(var(--primary-color), .65);
    background: rgba(var(--primary-color), .08);
}

.wiki-panel {
    position: sticky;
    top: 18px;
    display: grid;
    gap: 12px;
    padding-left: 22px;
    color: rgba(var(--text-color), .72);
}

.wiki-heading {
    justify-content: flex-start;
    padding-bottom: 10px;
    border-bottom: 1px solid rgba(var(--outline-color), .12);
}

.wiki-heading h2 {
    margin: 0;
    font-size: 14px;
}

.wiki-heading small {
    margin-left: auto;
    color: rgba(var(--text-color), .4);
    font-size: 10px;
}

.wiki-mark {
    display: inline-grid;
    width: 22px;
    height: 22px;
    place-items: center;
    border: 1px solid rgba(var(--outline-color), .28);
    border-radius: 50%;
    color: rgb(var(--primary-color));
    font-size: 11px;
    font-weight: 700;
}

.wiki-panel h3 {
    margin: 4px 0 0;
    color: rgb(var(--text-color));
    font-size: 14px;
}

.wiki-variant {
    display: grid;
    gap: 7px;
    padding-top: 12px;
    border-top: 1px solid rgba(var(--outline-color), .1);
}

.wiki-variant h4 {
    margin: 0;
    color: rgb(var(--text-color));
    font-size: 12px;
}

.wiki-tip {
    padding-left: 10px;
    border-left: 2px solid rgba(var(--primary-color), .45);
}

.wiki-facts {
    display: grid;
    gap: 9px;
    margin: 0;
}

.wiki-facts div {
    display: grid;
    gap: 2px;
}

.wiki-facts dt {
    color: rgba(var(--text-color), .76);
    font-size: 11px;
}

.wiki-facts dd {
    margin: 0;
    color: rgba(var(--text-color), .5);
    font-size: 11px;
    line-height: 1.55;
}

.settings-note {
    padding: 14px 0;
    color: rgba(var(--text-color), .62);
    font-size: 12px;
}

.error-note {
    color: rgb(214, 103, 103);
}

@media (max-width: 980px) {
    .effect-content-grid {
        grid-template-columns: 1fr;
    }

    .tab-panel {
        padding-right: 0;
    }

    .wiki-resize-handle {
        display: none;
    }

    .wiki-panel {
        position: static;
        padding: 18px 0 0;
        border-top: 1px solid rgba(var(--outline-color), .16);
    }
}

@media (max-width: 720px) {
    .playback-grid {
        grid-template-columns: 1fr;
    }

    .parameter-grid {
        grid-template-columns: 1fr;
    }

    .variant-heading {
        display: grid;
    }

    .effect-toolbar {
        justify-content: flex-start;
    }
}
</style>
