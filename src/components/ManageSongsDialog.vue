<template>
  <Dialog v-model="isOpen" :width="1540" height="min(820px, calc(100dvh - 28px))" max-height="calc(100dvh - 28px)"
    :aria-labelledby="'manage-songs-dialog-title'">
    <div class="manage-songs-dialog">
      <div class="song-picker">
        <div class="picker-layout">
          <aside class="source-panel">
            <div class="picker-header">
              <div class="picker-header-row">
                <h2 id="manage-songs-dialog-title">{{ t('playlistsPage.manageSongsTitle') }}</h2>
                <div class="picker-mode" role="tablist" :aria-label="t('playlistsPage.modeLabel')">
                  <MotionButton type="button" role="tab" :aria-selected="mode === 'static'"
                    :class="{ active: mode === 'static' }" :while-hover="{ y: -1 }" :while-press="{ scale: 0.97 }"
                    :transition="microTransition" @click="setMode('static')">
                    {{ t('playlistsPage.staticMode') }}
                  </MotionButton>
                  <MotionButton type="button" role="tab" :aria-selected="mode === 'dynamic'"
                    :class="{ active: mode === 'dynamic' }" :while-hover="{ y: -1 }" :while-press="{ scale: 0.97 }"
                    :transition="microTransition" @click="setMode('dynamic')">
                    {{ t('playlistsPage.dynamicMode') }}
                  </MotionButton>
                </div>
              </div>
            </div>

            <div class="source-heading">
              {{ mode === 'dynamic' ? t('playlistsPage.sources') : t('playlistsPage.fromSources') }}
            </div>

            <div class="source-list">
              <MotionButton v-for="source in pickerSources" :key="source.key" type="button" class="source-row" :class="{
                active: mode === 'static' && selectedSourceKey === source.key,
                selected: mode === 'dynamic' && ruleSourceKeys.includes(source.key),
                highlighted: mode === 'dynamic' && highlightedSourceKeys.includes(source.key)
              }" :while-press="{ scale: 0.98 }" :transition="microTransition" @click="selectSource(source)">
                <span class="source-icon">
                  <Icon :src="`/assets/${source.icon || 'playlist.svg'}`" size="sm" />
                </span>
                <span class="source-copy">
                  <strong>{{ source.name }}</strong>
                  <small>{{ sourceDescription(source) }}</small>
                </span>
                <span class="source-count">
                  {{ sourceResultCount(source) }}
                </span>
              </MotionButton>
            </div>

            <template v-if="mode === 'dynamic'">
              <div class="operator-heading">{{ t('playlistsPage.operators') }}</div>
              <div class="operator-list">
                <MotionButton v-for="operator in operators" :key="operator.op" type="button" class="operator-button"
                  :disabled="!canAppendOperator(operator.op)" :while-hover="{ y: -1 }" :while-press="{ scale: 0.97 }"
                  :transition="microTransition" @click="appendOperator(operator.op)">
                  <span class="operator-glyph">{{ operator.glyph }}</span>
                  {{ operator.label }}
                </MotionButton>
              </div>

            </template>
          </aside>

          <div class="picker-main">
            <template v-if="mode === 'static'">
              <div class="content-toolbar">
                <label class="song-search">
                  <Search :size="15" :stroke-width="1.8" />
                  <input v-model="query" type="search" :placeholder="t('playlistsPage.searchPlaceholder')" />
                </label>
                <MotionButton type="button" class="select-all-button" :disabled="!filteredSourceSongs.length"
                  :while-hover="{ y: -1 }" :while-press="{ scale: 0.97 }" :transition="microTransition"
                  @click="toggleAllStaticSongs">
                  {{ allVisibleStaticSongsSelected ? t('playlistsPage.clearAll') : t('playlistsPage.selectAll') }}
                </MotionButton>
              </div>
            </template>

            <section class="picker-content">
              <template v-if="mode === 'static'">
                <div v-if="isLoadingSource" class="picker-empty">
                  <p>{{ t('playlistsPage.loadingSongs') }}</p>
                </div>
                <div v-else-if="filteredSourceSongs.length" class="picker-list">
                  <label v-for="song in filteredSourceSongs" :key="song.id" class="picker-row"
                    :class="{ selected: staticSelectedIds.includes(song.id) }">
                    <span class="picker-check">
                      <input v-model="staticSelectedIds" type="checkbox" :value="song.id" />
                      <span aria-hidden="true"></span>
                    </span>
                    <img :src="song.cover" :alt="song.title" class="song-cover" />
                    <span class="song-copy">
                      <strong>{{ song.title }}</strong>
                      <small>{{ song.artist || t('player.unknownArtist') }}<span v-if="song.album"> · {{ song.album
                      }}</span></small>
                    </span>
                  </label>
                </div>
                <div v-else class="picker-empty">
                  <h3>{{ query ? t('playlistsPage.emptyAvailable') : t('playlistsPage.noSourceSongs') }}</h3>
                  <p v-if="query">{{ t('playlistsPage.clearSearch') }}</p>
                </div>
              </template>

              <template v-else>
                <div class="rule-section">
                  <p class="dynamic-tip">
                    <InfoIcon size="16" />{{ t('playlistsPage.dynamicTip') }}
                  </p>
                  <div class="section-label">{{ t('playlistsPage.rule') }}</div>
                  <div v-if="rule.steps.length" class="rule-chips">
                    <template v-for="(step, index) in rule.steps" :key="`${step.type}-${index}`">
                      <span class="rule-chip" :class="step.type">
                        {{ stepLabel(step) }}
                        <input v-if="step.type === 'operator' && step.op === 'randomChoose'" class="random-count-input"
                          type="number" min="1" max="10000" :value="step.count"
                          :aria-label="t('playlistsPage.randomCount')" @click.stop
                          @input="updateRandomCount(index, $event)" />
                        <button type="button" :aria-label="t('playlistsPage.removeRuleStep')"
                          @click="removeRuleStep(index)">×</button>
                      </span>
                    </template>
                  </div>
                  <p v-else class="rule-placeholder">{{ t('playlistsPage.rulePlaceholder') }}</p>
                </div>

                <div class="content-toolbar result-toolbar">
                  <div>
                    <div class="section-label">{{ t('playlistsPage.previewCount', { count: dynamicPreviewSongs.length })
                    }}</div>

                  </div>
                  <MotionButton type="button" class="select-all-button" :disabled="!rule.steps.length"
                    :while-hover="{ y: -1 }" :while-press="{ scale: 0.97 }" :transition="microTransition"
                    @click="refreshPreview">
                    {{ t('playlistsPage.refreshPreview') }}
                  </MotionButton>
                </div>

                <div v-if="isPreviewing" class="picker-empty">
                  <p>{{ t('playlistsPage.previewing') }}</p>
                </div>
                <div v-else-if="dynamicPreviewSongs.length" class="picker-list preview-list">
                  <MotionButton v-for="song in dynamicPreviewSongs" :key="song.id" type="button" class="preview-row"
                    :class="{ highlighted: highlightedSongId === song.id }" :while-press="{ scale: 0.99 }"
                    :transition="microTransition"
                    @click="highlightedSongId = highlightedSongId === song.id ? '' : song.id">
                    <img :src="song.cover" :alt="song.title" class="song-cover" />
                    <span class="song-copy">
                      <strong>{{ song.title }}</strong>
                      <small>{{ song.artist || t('player.unknownArtist') }}<span v-if="song.album"> · {{ song.album
                      }}</span></small>
                    </span>
                    <span class="preview-sources">{{ contributionLabel(song) }}</span>
                  </MotionButton>
                </div>
                <div v-else class="picker-empty">
                  <h3>{{ t('playlistsPage.emptyPreview') }}</h3>

                </div>
              </template>
            </section>
          </div>

          <div class="fab-actions">
            <button type="button" class="fab secondary" :disabled="saving" :aria-label="t('dialog.actions.cancel')"
              @click="close">
              <X :size="18" :stroke-width="2" />
            </button>
            <button type="button" class="fab primary" :disabled="saving || !valid" :aria-label="t('playlistsPage.save')"
              @click="save">
              <Check :size="20" :stroke-width="2.2" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </Dialog>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Check, Search, X } from '@lucide/vue'
import { motion, useReducedMotion } from 'motion-v'
import Dialog from './Dialog.vue'
import Icon from './Icon.vue'
import { useI18n } from '../i18n/index.js'
import { useLibraryStore } from '../stores/libraryStore.js'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'
import {
  clonePlaylistRule,
  emptyPlaylistRule,
  isPlaylistRuleValid,
  operatorStep,
  sourceKey,
  sourceStep
} from '../utils/playlistRule.js'
import { InfoIcon } from '@lucide/vue'
import { RefreshCcwIcon } from '@lucide/vue'

const props = defineProps({
  open: {
    type: Boolean,
    default: false
  },
  modelValue: {
    type: Object,
    default: () => ({ mode: 'static', trackIds: [] })
  },
  playlistName: {
    type: String,
    default: ''
  },
  songs: {
    type: Array,
    default: () => []
  },
  sources: {
    type: Array,
    default: () => []
  },
  existingSongIds: {
    type: Array,
    default: () => []
  },
  currentPlaylistId: {
    type: String,
    default: ''
  },
  initialMode: {
    type: String,
    default: 'static'
  },
  initialRule: {
    type: Object,
    default: null
  },
  initialStaticIds: {
    type: Array,
    default: () => []
  },
  valid: {
    type: Boolean,
    default: false
  },
  saving: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:open', 'update:modelValue', 'save'])
const { t } = useI18n()
const libraryStore = useLibraryStore()
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit('update:open', value)
})

const pickerSources = computed(() => props.sources.filter(
  (source) => !(source.kind === 'playlist' && source.id === props.currentPlaylistId)
))

const close = () => {
  isOpen.value = false
}

const save = () => {
  if (props.saving || !props.valid) return
  emit('save')
}

const mode = ref(props.initialMode === 'dynamic' ? 'dynamic' : 'static')
const query = ref('')
const selectedSourceKey = ref(pickerSources.value[0]?.key || '')
const staticSelectedIds = ref(props.initialMode === 'dynamic'
  ? [...props.initialStaticIds]
  : [...props.existingSongIds])
const rule = ref(clonePlaylistRule(props.initialRule || emptyPlaylistRule()))
const sourceSongs = ref([])
const isLoadingSource = ref(false)
const isPreviewing = ref(false)
const dynamicPreviewSongs = ref(Array.isArray(props.modelValue?.tracks) ? [...props.modelValue.tracks] : [])
const previewPending = ref(mode.value === 'dynamic')
const highlightedSongId = ref('')
let sourceRequestId = 0
let previewRequestId = 0
let previewTimer = 0

const operators = computed(() => [
  { op: 'union', glyph: '◉', label: t('playlistsPage.union') },
  { op: 'inter', glyph: '◎', label: t('playlistsPage.inter') },
  { op: 'concatenate', glyph: '◌', label: t('playlistsPage.concatenate') },
  { op: 'subtract', glyph: '⊖', label: t('playlistsPage.subtract') },
  { op: 'randomChoose', glyph: '✦', label: t('playlistsPage.randomChoose') }
])

const selectedSource = computed(() => pickerSources.value.find((source) => source.key === selectedSourceKey.value) || null)
const sourceSongMap = computed(() => {
  const map = new Map()
  for (const song of sourceSongs.value) map.set(song.id, song)
  return map
})
const filteredSourceSongs = computed(() => {
  const value = query.value.trim().toLowerCase()
  const songs = sourceSongs.value.filter((song) => !value
    || [song.title, song.artist, song.album]
      .some((field) => String(field || '').toLowerCase().includes(value)))
  const existing = props.existingSongIds
    .map((id) => sourceSongMap.value.get(id))
    .filter(Boolean)
    .filter((song) => !value
      || [song.title, song.artist, song.album]
        .some((field) => String(field || '').toLowerCase().includes(value)))
  const merged = [...existing, ...songs]
  const seen = new Set()
  return merged
    .filter((song) => (seen.has(song.id) ? false : (seen.add(song.id), true)))
    .slice(0, 100)
})
const allVisibleStaticSongsSelected = computed(() =>
  filteredSourceSongs.value.length > 0
  && filteredSourceSongs.value.every((song) => staticSelectedIds.value.includes(song.id))
)
const ruleSourceKeys = computed(() => rule.value.steps
  .filter((step) => step.type === 'source')
  .map(sourceKey))
const highlightedSourceKeys = computed(() => {
  const song = dynamicPreviewSongs.value.find((item) => item.id === highlightedSongId.value)
  return song?.sourceKeys || []
})

const sourceDescription = (source) => {
  const count = Number(source.trackCount) || 0
  return t('playlistsPage.sourceMeta', { songs: count })
}

const sourceResultCount = (source) => {
  if (mode.value === 'dynamic') {
    return dynamicPreviewSongs.value.filter((song) => (song.sourceKeys || []).includes(source.key)).length
  }
  return Number(source.trackCount) || 0
}

const selectedSongSources = (song) => (song.sourceKeys || []).filter((key) => ruleSourceKeys.value.includes(key))

const contributionLabel = (song) => {
  const count = selectedSongSources(song).length
  return count ? t('playlistsPage.sourceCount', { count }) : ''
}

const stepLabel = (step) => {
  if (step.type === 'operator') {
    const operator = operators.value.find((item) => item.op === step.op)
    return operator?.label || step.op
  }
  return pickerSources.value.find((source) => source.key === sourceKey(step))?.name || step.kind
}

const emitValue = () => {
  emit('update:modelValue', mode.value === 'dynamic'
    ? {
      mode: 'dynamic',
      rule: clonePlaylistRule(rule.value),
      trackIds: dynamicPreviewSongs.value.map((song) => song.id),
      tracks: [...dynamicPreviewSongs.value],
      previewReady: !previewPending.value
    }
    : { mode: 'static', trackIds: [...new Set(staticSelectedIds.value)] })
}

const loadSourceSongs = async (source) => {
  const requestId = ++sourceRequestId
  sourceSongs.value = []
  if (!source) return
  isLoadingSource.value = true
  try {
    const songs = source.kind === 'library'
      ? props.songs
      : await libraryStore.sourceTracks(source)
    if (requestId === sourceRequestId) sourceSongs.value = songs
  } catch (error) {
    if (requestId === sourceRequestId) {
      sourceSongs.value = []
      console.error('加载歌曲来源失败:', error)
    }
  } finally {
    if (requestId === sourceRequestId) isLoadingSource.value = false
  }
}

const selectSource = (source) => {
  if (mode.value === 'dynamic') {
    if (source.kind === 'playlist' && source.id === props.currentPlaylistId) return
    const key = source.key
    if (ruleSourceKeys.value.includes(key)) {
      const index = rule.value.steps.findIndex((step) => step.type === 'source' && sourceKey(step) === key)
      if (index >= 0 && (index === 0 || rule.value.steps[index - 1]?.type === 'operator')) removeRuleStep(index)
      return
    }
    if (!rule.value.steps.length || rule.value.steps.at(-1)?.type === 'operator') {
      rule.value.steps.push(sourceStep(source))
      emitValue()
      schedulePreview()
    }
    return
  }

  selectedSourceKey.value = source.key
  query.value = ''
  void loadSourceSongs(source)
}

const setMode = (nextMode) => {
  if (mode.value === nextMode) return
  const previousMode = mode.value
  mode.value = nextMode
  query.value = ''
  if (mode.value === 'static') {
    staticSelectedIds.value = previousMode === 'dynamic'
      ? dynamicPreviewSongs.value.map((song) => song.id)
      : [...props.initialStaticIds]
    void loadSourceSongs(selectedSource.value)
  } else {
    schedulePreview()
  }
  emitValue()
}

const canAppendOperator = (op) => {
  if (mode.value !== 'dynamic' || !rule.value.steps.length) return false
  const last = rule.value.steps.at(-1)
  if (last.type !== 'source') return false
  return op !== 'randomChoose' || !rule.value.steps.some((step) => step.type === 'operator' && step.op === 'randomChoose')
}

const appendOperator = (op) => {
  if (!canAppendOperator(op)) return
  rule.value.steps.push(operatorStep(op, op === 'randomChoose' ? 20 : null))
  emitValue()
  schedulePreview()
}

const removeRuleStep = (index) => {
  if (index < 0 || index >= rule.value.steps.length) return
  const step = rule.value.steps[index]
  const removeAt = new Set([index])
  if (step.type === 'source') {
    if (index === 0 && rule.value.steps[index + 1]?.type === 'operator') {
      removeAt.add(index + 1)
    } else if (rule.value.steps[index - 1]?.type === 'operator') {
      removeAt.add(index - 1)
    }
  } else if (step.op !== 'randomChoose' && rule.value.steps[index + 1]?.type === 'source') {
    removeAt.add(index + 1)
  }
  rule.value.steps = rule.value.steps.filter((_, stepIndex) => !removeAt.has(stepIndex))
  emitValue()
  schedulePreview()
}

const updateRandomCount = (index, event) => {
  const step = rule.value.steps[index]
  if (step?.type !== 'operator' || step.op !== 'randomChoose') return
  step.count = Math.min(10000, Math.max(1, Math.floor(Number(event.target.value) || 1)))
  emitValue()
  schedulePreview()
}

const toggleAllStaticSongs = () => {
  const visibleIds = filteredSourceSongs.value.map((song) => song.id)
  if (allVisibleStaticSongsSelected.value) {
    staticSelectedIds.value = staticSelectedIds.value.filter((id) => !visibleIds.includes(id))
  } else {
    staticSelectedIds.value = [...new Set([...staticSelectedIds.value, ...visibleIds])]
  }
  emitValue()
}

const evaluatePreview = async () => {
  if (mode.value !== 'dynamic' || !isPlaylistRuleValid(rule.value) || !props.currentPlaylistId) {
    dynamicPreviewSongs.value = []
    previewPending.value = false
    emitValue()
    return
  }
  const requestId = ++previewRequestId
  isPreviewing.value = true
  try {
    const result = await libraryStore.evaluatePlaylist(props.currentPlaylistId, rule.value)
    if (requestId === previewRequestId) {
      dynamicPreviewSongs.value = result.tracks || []
      previewPending.value = false
      emitValue()
    }
  } catch (error) {
    if (requestId === previewRequestId) {
      dynamicPreviewSongs.value = []
      previewPending.value = false
      emitValue()
      console.error('预览动态播放列表失败:', error)
    }
  } finally {
    if (requestId === previewRequestId) isPreviewing.value = false
  }
}

const schedulePreview = () => {
  if (previewTimer) window.clearTimeout(previewTimer)
  previewPending.value = true
  emitValue()
  previewTimer = window.setTimeout(() => {
    previewTimer = 0
    void evaluatePreview()
  }, 180)
}

const refreshPreview = () => {
  if (previewTimer) window.clearTimeout(previewTimer)
  previewTimer = 0
  previewPending.value = true
  emitValue()
  void evaluatePreview()
}

watch(staticSelectedIds, emitValue, { deep: true })
watch(pickerSources, (sources) => {
  if (!sources.some((source) => source.key === selectedSourceKey.value)) {
    selectedSourceKey.value = sources[0]?.key || ''
  }
  if (mode.value === 'static') void loadSourceSongs(selectedSource.value)
}, { immediate: true })
watch(() => props.modelValue, (value) => {
  if (value?.mode === 'dynamic' && value.rule) rule.value = clonePlaylistRule(value.rule)
}, { deep: true })

onMounted(() => {
  if (mode.value === 'dynamic') refreshPreview()
})

onBeforeUnmount(() => {
  if (previewTimer) window.clearTimeout(previewTimer)
})
</script>

<style scoped>
.manage-songs-dialog {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: rgb(var(--text-color));
}

/* Picker fills the dialog. */
.song-picker {
  --picker-panel: color-mix(in srgb, rgb(var(--surface-color)) 72%, rgb(var(--global-color)) 28%);
  --picker-raised: color-mix(in srgb, var(--picker-panel) 92%, rgb(var(--primary-color)) 8%);
  --picker-divider: rgba(var(--outline-color), 0.12);
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: rgb(var(--text-color));
}

.picker-layout {
  position: relative;
  display: grid;
  flex: 1 1 auto;
  grid-template-columns: minmax(220px, 300px) minmax(0, 1fr);
  width: 100%;
  min-height: 0;
  overflow: hidden;
}

.source-panel {
  display: grid;
  align-content: start;
  gap: 6px;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  padding: 10px 16px;
  background-color: color-mix(in srgb, rgba(var(--global-color), 0.05) 25%, rgba(var(--primary-color), 0.05) 75%)
}

.picker-header {
  margin-bottom: 12px;
}

.picker-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-width: 0;
}

.picker-header h2 {
  margin: 0;
  color: rgb(var(--text-color));
  font-size: 15px;
  font-weight: 750;
  letter-spacing: -0.01em;
  line-height: 1.2;
}

.picker-mode {
  display: inline-flex;
  width: fit-content;
  padding: 3px;
  border: 1px solid rgba(var(--outline-color), 0.17);
  border-radius: 8px;
  background: rgba(var(--global-inverse-color), 0.05);
}

.picker-mode button,
.select-all-button,
.operator-button,
.source-row,
.preview-row,
.rule-chip button {
  border: 0;
  font: inherit;
  cursor: pointer;
}

.picker-mode button {
  min-height: 28px;
  padding: 0 12px;
  border-radius: 6px;
  color: rgba(var(--text-color), 0.5);
  background: transparent;
  font-size: 12px;
  font-weight: 650;
}

.picker-mode button.active {
  color: rgb(var(--text-color));
  background: rgba(var(--global-inverse-color), 0.14);
}

.source-heading,
.operator-heading {
  padding: 4px 6px;
  color: rgba(var(--text-color), 0.5);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.source-list,
.operator-list {
  display: grid;
  gap: 3px;
}

.source-row {
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  min-width: 0;
  min-height: 48px;
  padding: 7px 10px;
  border: 1px solid transparent;
  border-radius: 10px;
  color: rgb(var(--text-color));
  background: transparent;
  text-align: left;
}

.source-row:hover,
.source-row.active,
.source-row.selected {
  border-color: rgba(var(--primary-color), 0.12);
  background: var(--picker-raised);
}

.source-row.highlighted {
  border-color: rgba(var(--primary-color), 0.4);
  box-shadow: inset 0 0 0 1px rgba(var(--primary-color), 0.22);
}

.source-row:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.source-icon {
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  color: rgb(var(--primary-color));
  background: rgba(var(--primary-color), 0.12);
}

.source-copy,
.song-copy {
  display: grid;
  min-width: 0;
  gap: 2px;
}

.source-copy strong,
.source-copy small,
.song-copy strong,
.song-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-copy strong {
  font-size: 12.5px;
  font-weight: 650;
}

.source-copy small,
.source-count,
.preview-sources,
.result-toolbar small {
  color: rgba(var(--text-color), 0.54);
  font-size: 11px;
}

.source-count {
  min-width: 22px;
  padding: 2px 6px;
  border-radius: 999px;
  background: rgba(var(--global-inverse-color), 0.08);
  font-weight: 600;
  text-align: center;
}

.operator-heading {
  margin-top: 12px;
}

.operator-list {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.operator-button {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  padding: 6px 8px;
  border-radius: 8px;
  color: rgba(var(--text-color), 0.76);
  background: transparent;
  font-size: 11.5px;
  font-weight: 600;
  text-align: left;
}

.operator-button:last-child {
  grid-column: 1 / -1;
}

.operator-button:hover:not(:disabled) {
  background: rgba(var(--primary-color), 0.09);
}

.operator-button:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.operator-glyph {
  display: inline-grid;
  place-items: center;
  width: 20px;
  height: 20px;
  border: 1px solid rgba(var(--text-color), 0.32);
  border-radius: 50%;
  font-size: 11px;
}

.picker-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  padding: 12px 28px 20px;
  padding-bottom: 0px;
  gap: 10px;
}

.picker-content {
  display: grid;
  align-content: start;
  gap: 10px;
  grid-template-rows: auto auto 1fr;
  min-width: 0;
  min-height: 0;
  height: 100%;
}

.fab-actions {
  position: absolute;
  right: 22px;
  bottom: 18px;
  z-index: 6;
  display: flex;
  align-items: flex-end;
  gap: 12px;
  pointer-events: none;
}

.fab-actions .fab {
  pointer-events: auto;
}

.fab {
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 50%;
  cursor: pointer;
  color: rgb(var(--text-color));
  box-shadow:
    0 6px 18px rgba(0, 0, 0, 0.22),
    0 2px 6px rgba(0, 0, 0, 0.14);
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.fab:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow:
    0 10px 26px rgba(0, 0, 0, 0.28),
    0 4px 10px rgba(0, 0, 0, 0.16);
}

.fab:active:not(:disabled) {
  transform: translateY(0) scale(0.96);
}

.fab:disabled {
  opacity: 0.45;
  cursor: not-allowed;
  box-shadow: none;
}

.fab.secondary {
  width: 56px;
  height: 56px;
  color: rgb(var(--text-color));
  background: color-mix(in srgb, rgb(var(--global-color)) 78%, rgb(var(--primary-color)) 22%);
  backdrop-filter: blur(10px);
}

.fab.primary {
  width: 56px;
  height: 56px;
  color: rgb(var(--global-inverse-color));
  background: color-mix(in srgb, rgb(var(--global-color)) 60%, rgb(var(--primary-color)) 40%);
}

.content-toolbar,
.result-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.result-toolbar {
  align-items: end;
}

.result-toolbar>div {
  display: grid;
  gap: 3px;
}

.song-search {
  display: flex;
  align-items: center;
  flex: 1 1 auto;
  gap: 8px;
  min-width: 0;
  min-height: 38px;
  padding: 0 12px;
  border: 1px solid rgba(var(--outline-color), 0.16);
  border-radius: 10px;
  color: rgba(var(--text-color), 0.48);
  background: rgba(var(--global-inverse-color), 0.05);
}

.song-search input {
  width: 100%;
  min-width: 0;
  border: 0;
  outline: 0;
  color: rgb(var(--text-color));
  background: transparent;
  font-size: 13px;
}

.select-all-button {
  flex: 0 0 auto;
  min-height: 34px;
  padding: 0 12px;
  border: 1px solid rgba(var(--primary-color), 0.18);
  border-radius: 9px;
  color: rgb(var(--primary-color));
  background: rgba(var(--primary-color), 0.11);
  font-size: 11.5px;
  font-weight: 650;
}

.select-all-button:disabled {
  opacity: 0.38;
  cursor: not-allowed;
}

.picker-list {
  display: grid;
  gap: 2px;
  min-height: 0;
  height: 100%;
  overflow-y: auto;
}

.picker-row,
.preview-row {
  min-width: 0;
  border: 1px solid transparent;
  border-radius: 9px;
}

.picker-row {
  display: grid;
  grid-template-columns: 20px 40px minmax(0, 1fr);
  align-items: center;
  gap: 10px;
  min-height: 52px;
  padding: 5px 8px;
  cursor: pointer;
}

.picker-row:hover,
.picker-row.selected,
.preview-row:hover,
.preview-row.highlighted {
  border-color: rgba(var(--primary-color), 0.2);
  background: rgba(var(--primary-color), 0.09);
}

.picker-check {
  position: relative;
  width: 20px;
  height: 20px;
}

.picker-check input {
  position: absolute;
  inset: 0;
  z-index: 1;
  width: 100%;
  height: 100%;
  margin: 0;
  opacity: 0;
  cursor: pointer;
}

.picker-check span {
  display: block;
  width: 20px;
  height: 20px;
  border: 1px solid rgba(var(--text-color), 0.32);
  border-radius: 6px;
  background: rgba(var(--global-inverse-color), 0.05);
}

.picker-check input:checked+span {
  border-color: rgb(var(--primary-color));
  background: rgb(var(--primary-color));
  box-shadow: inset 0 0 0 3px var(--picker-panel);
}

.song-cover {
  width: 40px;
  height: 40px;
  border-radius: 7px;
  object-fit: cover;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.song-copy strong {
  color: rgb(var(--text-color));
  font-size: 13px;
  font-weight: 600;
}

.song-copy small {
  color: rgba(var(--text-color), 0.56);
  font-size: 11px;
}

.picker-empty {
  display: grid;
  place-items: center;
  gap: 4px;
  min-height: 180px;
  color: rgba(var(--text-color), 0.52);
  text-align: center;
}

.picker-empty h3,
.picker-empty p {
  margin: 0;
}

.picker-empty h3 {
  color: rgb(var(--text-color));
  font-size: 12px;
  font-weight: 600;
}

.picker-empty p {
  font-size: 11px;
}

.rule-section {
  display: grid;
  min-height: 62px;
  grid-template-rows: auto auto auto;
  gap: 10px;
}

.section-label {
  color: rgba(var(--text-color), 0.48);
  font-size: 14px;
  font-weight: 750;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.rule-chips {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 5px;
}

.rule-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  max-width: 100%;
  padding: 4px 7px 4px 9px;
  border: 1px solid rgba(var(--primary-color), 0.24);
  border-radius: 999px;
  color: rgb(var(--text-color));
  background: rgba(var(--primary-color), 0.1);
  font-size: 11px;
  font-weight: 650;
}

.rule-chip.operator {
  color: rgba(var(--text-color), 0.65);
  border-color: rgba(var(--outline-color), 0.15);
  background: rgba(var(--outline-color), 0.08);
}

.rule-chip button {
  width: 14px;
  height: 14px;
  padding: 0;
  border-radius: 50%;
  color: inherit;
  background: transparent;
  line-height: 1;
}

.rule-chip button:hover {
  background: rgba(var(--text-color), 0.12);
}

.random-count-input {
  width: 36px;
  padding: 1px 3px;
  border: 0;
  border-radius: 5px;
  color: inherit;
  background: rgba(var(--surface-color), 0.6);
  font: inherit;
  text-align: center;
}

.rule-placeholder {
  margin: 0;
  color: rgba(var(--text-color), 0.42);
  font-size: 11px;
}

.preview-row {
  display: grid;
  grid-template-columns: 40px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  min-height: 52px;
  padding: 5px 8px;
  color: inherit;
  background: transparent;
  text-align: left;
}

.preview-sources {
  white-space: nowrap;
}

.dynamic-tip {
  margin-bottom: 10px;
  color: rgba(var(--primary-color), 0.75);
  font-size: 11px;
  display: flex;
  gap: 10px;
}

@media (max-width: 900px) {
  .picker-layout {
    grid-template-columns: minmax(220px, 0.8fr) minmax(0, 1.4fr);
  }

  .source-panel {
    padding-right: 12px;
    padding-left: 12px;
  }

  .picker-content {
    padding-right: 20px;
    padding-left: 20px;
  }
}

@media (max-width: 640px) {
  .picker-layout {
    grid-template-columns: 1fr;
    overflow-y: auto;
  }

  .source-panel {
    overflow: visible;
    border-right: 0;
    border-bottom: 1px solid var(--picker-divider);
  }

  .picker-content {
    overflow: visible;
    padding: 16px 18px;
  }
}
</style>
