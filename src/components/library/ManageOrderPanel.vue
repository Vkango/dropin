<template>
  <section class="order-editor" :aria-label="t('playlistsPage.manageOrderTitle')">
    <aside class="order-rule-panel">

      <p class="order-hint">
        <InfoIcon size="14" />{{ t('playlistsPage.manageOrderHint') }}
      </p>

      <div class="rule-field">
        <label for="sort-rule-select">{{ t('playlistsPage.sortRule') }}</label>
        <div class="rule-picker">
          <select id="sort-rule-select" v-model="selectedRuleId" @change="selectRule">
            <option value="">{{ t('playlistsPage.unsavedRule') }}</option>
            <option v-for="sortRule in sortRules" :key="sortRule.id" :value="sortRule.id">
              {{ sortRule.name }}
            </option>
          </select>
          <button v-if="selectedRuleId" type="button" class="icon-button"
            :aria-label="t('playlistsPage.removeSortRule')" :disabled="isRuleSaving" @click="removeRule">
            <Trash2 :size="14" />
          </button>
        </div>
      </div>

      <div class="rule-field">
        <label for="sort-rule-name">{{ t('playlistsPage.ruleName') }}</label>
        <input id="sort-rule-name" v-model="ruleName" type="text"
          :placeholder="t('playlistsPage.ruleNamePlaceholder')" />
      </div>

      <div class="rule-section">
        <div class="rule-section-heading">
          <span>{{ t('playlistsPage.tagWeights') }}</span>
          <button type="button" class="text-button" :disabled="!availableTags.length" @click="addTagWeight">
            <Plus :size="13" />{{ t('playlistsPage.addCondition') }}
          </button>
        </div>
        <div v-if="ruleDraft.tagWeights.length" class="rule-lines">
          <div v-for="(item, index) in ruleDraft.tagWeights" :key="`tag-${index}`" class="rule-line">
            <select v-model="item.tagId" :aria-label="t('playlistsPage.tagCondition')">
              <option v-for="tag in tagOptions(index)" :key="tag.id" :value="tag.id">{{ tag.label }}</option>
            </select>
            <input v-model.number="item.weight" type="number" min="0" max="1000000"
              :aria-label="t('playlistsPage.tagWeight')" />
            <button type="button" class="icon-button" :aria-label="t('playlistsPage.removeCondition')"
              @click="removeTagWeight(index)">
              <Trash2 :size="14" />
            </button>
          </div>
          <label class="direction-field">
            <span>{{ t('playlistsPage.tagScoreDirection') }}</span>
            <select v-model="ruleDraft.tagDirection">
              <option value="desc">{{ t('playlistsPage.descending') }}</option>
              <option value="asc">{{ t('playlistsPage.ascending') }}</option>
            </select>
          </label>
        </div>
        <p v-else class="empty-rule-line">{{ t('playlistsPage.noTagConditions') }}</p>
      </div>

      <div class="rule-section">
        <div class="rule-section-heading">
          <span>{{ t('playlistsPage.metadataOrder') }}</span>
          <button type="button" class="text-button" :disabled="availableFields.length === 0" @click="addField">
            <Plus :size="13" />{{ t('playlistsPage.addCondition') }}
          </button>
        </div>
        <div v-if="ruleDraft.fields.length" class="rule-lines">
          <div v-for="(item, index) in ruleDraft.fields" :key="`field-${index}`" class="rule-line">
            <select v-model="item.field" :aria-label="t('playlistsPage.metadataField')">
              <option v-for="field in fieldOptions" :key="field.value" :value="field.value">{{ field.label }}</option>
            </select>
            <select v-model="item.direction" :aria-label="t('playlistsPage.sortDirection')">
              <option value="asc">{{ t('playlistsPage.ascending') }}</option>
              <option value="desc">{{ t('playlistsPage.descending') }}</option>
            </select>
            <button type="button" class="icon-button" :aria-label="t('playlistsPage.removeCondition')"
              @click="removeField(index)">
              <Trash2 :size="14" />
            </button>
          </div>
        </div>
        <p v-else class="empty-rule-line">{{ t('playlistsPage.noMetadataConditions') }}</p>
      </div>

      <div class="rule-actions">
        <button type="button" class="secondary-action" :disabled="isRuleSaving || !ruleName.trim()" @click="saveRule">
          <Save :size="14" />{{ t('playlistsPage.saveRule') }}
        </button>
        <button type="button" class="primary-action" :disabled="isPreviewing || !currentSongs.length"
          @click="runPreview">
          <RefreshCcw :size="14" />{{ t('playlistsPage.generateOrder') }}
        </button>
      </div>
    </aside>

    <main class="order-main">
      <div class="order-toolbar">
        <div>
          <div class="section-label">{{ t('playlistsPage.currentOrder') }}</div>
          <p>{{ t('playlistsPage.orderCount', { count: workingSongs.length }) }}</p>
        </div>
        <span v-if="hasManualChanges" class="dirty-state">{{ t('playlistsPage.unsavedOrder') }}</span>
      </div>

      <div v-if="isLoading" class="order-empty">
        <p>{{ t('playlistsPage.loadingOrder') }}</p>
      </div>
      <div v-else-if="!workingSongs.length" class="order-empty">
        <h3>{{ t('playlistsPage.emptyPlaylist') }}</h3>
        <p>{{ t('playlistsPage.addSongsHint') }}</p>
      </div>
      <ol v-else class="order-list" @dragover.prevent>
        <li v-for="(song, index) in workingSongs" :key="song.id" class="order-row"
          :class="{ dragging: draggingIndex === index }" draggable="true" @dragstart="startDrag(index, $event)"
          @dragend="endDrag" @drop="dropSong(index, $event)">
          <span class="order-index">{{ index + 1 }}</span>
          <button type="button" class="drag-handle" :aria-label="t('playlistsPage.dragSong', { name: song.title })">
            <GripVertical :size="16" />
          </button>
          <img class="order-cover" :src="song.cover || '/assets/cover.jpg'" :alt="song.title" />
          <span class="order-song-copy">
            <strong>{{ song.title }}</strong>
            <small>{{ song.artist || t('player.unknownArtist') }}<span v-if="song.album"> · {{ song.album
                }}</span></small>
          </span>
          <span class="order-song-duration">{{ song.duration || '--:--' }}</span>
          <div class="row-actions">
            <button type="button" class="icon-button" :disabled="index === 0" :aria-label="t('playlistsPage.moveUp')"
              @click="moveSong(index, -1)">
              <ChevronUp :size="15" />
            </button>
            <button type="button" class="icon-button" :disabled="index === workingSongs.length - 1"
              :aria-label="t('playlistsPage.moveDown')" @click="moveSong(index, 1)">
              <ChevronDown :size="15" />
            </button>
          </div>
        </li>
      </ol>

      <footer class="order-footer">
        <button type="button" class="close-action" :disabled="isSaving" @click="$emit('close')">
          {{ t('dialog.actions.cancel') }}
        </button>
        <div class="footer-actions">
          <button type="button" class="secondary-action" :disabled="isSaving || isLoading" @click="openCloneDialog">
            <Copy :size="14" />{{ t('playlistsPage.saveAsPlaylist') }}
          </button>
          <button type="button" class="primary-action" :disabled="isSaving || isLoading" @click="saveCurrentOrder">
            <Check :size="15" />{{ t('playlistsPage.saveOrder') }}
          </button>
        </div>
      </footer>
    </main>

    <Dialog v-model="isCloneDialogOpen" width="460" :aria-labelledby="'clone-playlist-dialog-title'">
      <form class="clone-dialog" @submit.prevent="clonePlaylist">
        <header class="dialog-header">
          <h2 id="clone-playlist-dialog-title">{{ t('playlistsPage.saveAsPlaylist') }}</h2>
        </header>
        <p class="dialog-message">{{ t('playlistsPage.cloneMessage') }}</p>
        <input v-model="cloneName" class="dialog-input" type="text"
          :placeholder="t('dialog.playlist.createPlaceholder')" :disabled="isCloning" autofocus />
        <footer class="dialog-actions">
          <button type="button" class="dialog-button secondary" :disabled="isCloning"
            @click="isCloneDialogOpen = false">{{ t('dialog.actions.cancel') }}</button>
          <button type="submit" class="dialog-button primary" :disabled="isCloning || !cloneName.trim()">
            {{ t('playlistsPage.cloneConfirm') }}
          </button>
        </footer>
      </form>
    </Dialog>
  </section>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import {
  Check,
  ChevronDown,
  ChevronUp,
  Copy,
  GripVertical,
  Plus,
  RefreshCcw,
  Save,
  Trash2
} from '@lucide/vue'
import Dialog from '@/components/ui/Dialog.vue'
import { useI18n } from '@/i18n/index.js'
import { useLibraryStore } from '@/stores/libraryStore.js'
import { InfoIcon } from '@lucide/vue'

const props = defineProps({
  playlistId: { type: String, default: '' },
  playlistName: { type: String, default: '' },
  playlistType: { type: String, default: 'static' }
})

const emit = defineEmits(['close', 'saved'])
const { t } = useI18n()
const libraryStore = useLibraryStore()

const defaultRule = () => ({
  version: 1,
  tagWeights: [],
  tagDirection: 'desc',
  fields: [
    { field: 'year', direction: 'asc' },
    { field: 'album', direction: 'asc' },
    { field: 'discNumber', direction: 'asc' },
    { field: 'trackNumber', direction: 'asc' },
    { field: 'title', direction: 'asc' }
  ]
})

const cloneRule = (rule) => JSON.parse(JSON.stringify(rule || defaultRule()))
const isLoading = ref(true)
const isPreviewing = ref(false)
const isSaving = ref(false)
const isRuleSaving = ref(false)
const workingSongs = ref([])
const persistedSongs = ref([])
const selectedRuleId = ref('')
const ruleName = ref('')
const ruleDraft = ref(defaultRule())
const savedRuleSnapshot = ref(null)
const savedRuleName = ref('')
const draggingIndex = ref(-1)
const hasManualChanges = ref(false)
const hasPersistedManualOrder = ref(false)
const isCloneDialogOpen = ref(false)
const isCloning = ref(false)
const cloneName = ref('')

const sortRules = computed(() => libraryStore.sortRules.value || [])
const tags = computed(() => libraryStore.tags.value || [])
const availableTags = computed(() => tags.value.filter((tag) =>
  !ruleDraft.value.tagWeights.some((item) => item.tagId === tag.id)
))
const fieldOptions = computed(() => [
  { value: 'title', label: t('playlistsPage.fieldTitle') },
  { value: 'artist', label: t('playlistsPage.fieldArtist') },
  { value: 'album', label: t('playlistsPage.fieldAlbum') },
  { value: 'albumArtist', label: t('playlistsPage.fieldAlbumArtist') },
  { value: 'composer', label: t('playlistsPage.fieldComposer') },
  { value: 'year', label: t('playlistsPage.fieldYear') },
  { value: 'discNumber', label: t('playlistsPage.fieldDiscNumber') },
  { value: 'trackNumber', label: t('playlistsPage.fieldTrackNumber') },
  { value: 'durationMs', label: t('playlistsPage.fieldDuration') },
  { value: 'addedAt', label: t('playlistsPage.fieldAddedAt') },
  { value: 'lastPlayedAt', label: t('playlistsPage.fieldLastPlayedAt') }
])
const availableFields = computed(() => fieldOptions.value.filter((field) =>
  !ruleDraft.value.fields.some((item) => item.field === field.value)
))
const currentSongs = computed(() => workingSongs.value)
const ruleIsDirty = computed(() => Boolean(selectedRuleId.value && savedRuleSnapshot.value
  && (JSON.stringify(ruleDraft.value) !== JSON.stringify(savedRuleSnapshot.value)
    || ruleName.value.trim() !== savedRuleName.value)))

const tagOptions = (index) => tags.value.filter((tag) =>
  !ruleDraft.value.tagWeights.some((item, itemIndex) => itemIndex !== index && item.tagId === tag.id)
)

const loadSelectedRule = () => {
  const sortRule = sortRules.value.find((item) => item.id === selectedRuleId.value)
  if (!sortRule) {
    ruleDraft.value = defaultRule()
    ruleName.value = ''
    savedRuleSnapshot.value = null
    savedRuleName.value = ''
    return
  }
  ruleDraft.value = cloneRule(sortRule.rule)
  ruleName.value = sortRule.name
  savedRuleSnapshot.value = cloneRule(sortRule.rule)
  savedRuleName.value = sortRule.name
}

const selectRule = () => loadSelectedRule()

const loadOrder = async () => {
  if (!props.playlistId) return
  isLoading.value = true
  try {
    const result = await libraryStore.playlistOrder(props.playlistId)
    const tracks = result.tracks || []
    workingSongs.value = [...tracks]
    persistedSongs.value = [...tracks]
    selectedRuleId.value = result.sortRuleId || ''
    loadSelectedRule()
    hasManualChanges.value = false
    hasPersistedManualOrder.value = Boolean(result.hasManualOrder)
  } catch (error) {
    console.error('加载曲目编排失败:', error)
    workingSongs.value = []
    persistedSongs.value = []
  } finally {
    isLoading.value = false
  }
}

const addTagWeight = () => {
  const tag = availableTags.value[0]
  if (!tag) return
  ruleDraft.value.tagWeights.push({
    tagId: tag.id,
    weight: Math.max(0, 100 - ruleDraft.value.tagWeights.length * 10)
  })
}

const removeTagWeight = (index) => ruleDraft.value.tagWeights.splice(index, 1)

const addField = () => {
  const field = availableFields.value[0]
  if (!field) return
  ruleDraft.value.fields.push({ field: field.value, direction: 'asc' })
}

const removeField = (index) => ruleDraft.value.fields.splice(index, 1)

const saveRule = async () => {
  const name = ruleName.value.trim()
  if (!name || isRuleSaving.value) return
  isRuleSaving.value = true
  try {
    const result = await libraryStore.saveSortRule(selectedRuleId.value || null, name, cloneRule(ruleDraft.value))
    selectedRuleId.value = result.id
    ruleName.value = result.name
    savedRuleSnapshot.value = cloneRule(result.rule)
    savedRuleName.value = result.name
  } catch (error) {
    console.error('保存排序规则失败:', error)
  } finally {
    isRuleSaving.value = false
  }
}

const removeRule = async () => {
  if (!selectedRuleId.value || isRuleSaving.value) return
  if (!window.confirm(t('playlistsPage.removeSortRuleConfirm'))) return
  isRuleSaving.value = true
  try {
    await libraryStore.removeSortRule(selectedRuleId.value)
    selectedRuleId.value = ''
    loadSelectedRule()
  } catch (error) {
    console.error('删除排序规则失败:', error)
  } finally {
    isRuleSaving.value = false
  }
}

const runPreview = async () => {
  if (!props.playlistId || !currentSongs.value.length || isPreviewing.value) return
  if ((hasManualChanges.value || hasPersistedManualOrder.value || selectedRuleId.value)
    && !window.confirm(t('playlistsPage.replaceOrderConfirm'))) return
  isPreviewing.value = true
  try {
    const result = await libraryStore.previewPlaylistOrder(
      props.playlistId,
      selectedRuleId.value || null,
      cloneRule(ruleDraft.value)
    )
    workingSongs.value = [...(result.tracks || [])]
    hasManualChanges.value = false
  } catch (error) {
    console.error('生成曲目编排失败:', error)
  } finally {
    isPreviewing.value = false
  }
}

const startDrag = (index, event) => {
  draggingIndex.value = index
  event.dataTransfer?.setData('text/plain', String(index))
  if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move'
}

const endDrag = () => {
  draggingIndex.value = -1
}

const dropSong = (targetIndex, event) => {
  const sourceIndex = Number(event.dataTransfer?.getData('text/plain'))
  if (!Number.isInteger(sourceIndex) || sourceIndex < 0 || sourceIndex === targetIndex) return
  const [song] = workingSongs.value.splice(sourceIndex, 1)
  workingSongs.value.splice(sourceIndex < targetIndex ? targetIndex - 1 : targetIndex, 0, song)
  hasManualChanges.value = true
  draggingIndex.value = -1
}

const moveSong = (index, offset) => {
  const targetIndex = index + offset
  if (targetIndex < 0 || targetIndex >= workingSongs.value.length) return
  const [song] = workingSongs.value.splice(index, 1)
  workingSongs.value.splice(targetIndex, 0, song)
  hasManualChanges.value = true
}

const saveCurrentOrder = async () => {
  if (!props.playlistId || isSaving.value) return
  isSaving.value = true
  try {
    await libraryStore.savePlaylistOrder(
      props.playlistId,
      workingSongs.value.map((song) => song.id),
      selectedRuleId.value || null
    )
    persistedSongs.value = [...workingSongs.value]
    hasManualChanges.value = false
    hasPersistedManualOrder.value = props.playlistType === 'dynamic'
    emit('saved')
  } catch (error) {
    console.error('保存曲目编排失败:', error)
  } finally {
    isSaving.value = false
  }
}

const openCloneDialog = () => {
  cloneName.value = `${props.playlistName} ${t('playlistsPage.copySuffix')}`.trim()
  isCloneDialogOpen.value = true
}

const clonePlaylist = async () => {
  const name = cloneName.value.trim()
  if (!name || isCloning.value) return
  isCloning.value = true
  try {
    await libraryStore.clonePlaylist(
      props.playlistId,
      name,
      null,
      workingSongs.value.map((song) => song.id)
    )
    isCloneDialogOpen.value = false
    emit('saved')
  } catch (error) {
    console.error('克隆播放列表失败:', error)
  } finally {
    isCloning.value = false
  }
}

onMounted(loadOrder)
watch(() => props.playlistId, loadOrder)
</script>

<style scoped>
.order-editor {
  --picker-panel: color-mix(in srgb, rgb(var(--surface-color)) 72%, rgb(var(--global-color)) 28%);
  --picker-raised: color-mix(in srgb, var(--picker-panel) 92%, rgb(var(--primary-color)) 8%);
  display: grid;
  flex: 1 1 auto;
  grid-template-columns: minmax(255px, 310px) minmax(0, 1fr);
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: rgb(var(--text-color));
}

.order-rule-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  padding: 18px 18px 22px;
  background: color-mix(in srgb, rgba(var(--global-color), 0.05) 25%, rgba(var(--primary-color), 0.05) 75%);
}

.order-panel-header,
.order-toolbar,
.rule-section-heading,
.order-footer,
.rule-actions,
.direction-field,
.footer-actions {
  display: flex;
  align-items: center;
}

.order-panel-header,
.order-toolbar,
.order-footer,
.rule-section-heading {
  justify-content: space-between;
  gap: 12px;
}

.order-panel-header h3 {
  max-width: 190px;
  margin: 4px 0 0;
  overflow: hidden;
  font-size: 15px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.panel-eyebrow,
.section-label {
  color: rgba(var(--text-color), 0.5);
  font-size: 10px;
  font-weight: 750;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.playlist-type,
.dirty-state {
  flex: 0 0 auto;
  padding: 4px 7px;
  border-radius: 999px;
  color: rgb(var(--primary-color));
  background: rgba(var(--primary-color), 0.1);
  font-size: 10px;
  font-weight: 700;
}

.dirty-state {
  color: rgba(var(--text-color), 0.68);
  background: rgba(var(--outline-color), 0.1);
}

.order-hint,
.order-toolbar p,
.dialog-message {
  margin: 0;
  color: rgba(var(--text-color), 0.55);
  font-size: 11px;
  line-height: 1.55;
}

.order-hint {
  display: grid;
  grid-template-columns: 24px 1fr;
}

.rule-field {
  display: grid;
  gap: 6px;
}

.rule-field label,
.direction-field span {
  color: rgba(var(--text-color), 0.55);
  font-size: 11px;
  font-weight: 650;
}

.rule-field input,
.rule-field select,
.rule-line select,
.rule-line input,
.direction-field select,
.clone-dialog .dialog-input {
  min-width: 0;
  min-height: 34px;
  border: 1px solid rgba(var(--outline-color), 0.16);
  border-radius: 8px;
  outline: 0;
  padding: 0 9px;
  color: rgb(var(--text-color));
  background: rgba(var(--global-inverse-color), 0.05);
  font: inherit;
  font-size: 12px;
}

.rule-picker {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 26px;
  align-items: center;
  gap: 5px;
}

.rule-field input:focus,
.rule-field select:focus,
.rule-line select:focus,
.rule-line input:focus,
.direction-field select:focus {
  border-color: rgba(var(--primary-color), 0.58);
}

.rule-section {
  display: grid;
  gap: 9px;
}

.rule-section-heading {
  color: rgba(var(--text-color), 0.58);
  font-size: 11px;
  font-weight: 750;
}

.text-button,
.secondary-action,
.primary-action,
.close-action,
.icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 0;
  cursor: pointer;
  font: inherit;
}

.text-button {
  padding: 2px 0;
  color: rgb(var(--primary-color));
  background: transparent;
  font-size: 10px;
  font-weight: 700;
}

.text-button:disabled,
.secondary-action:disabled,
.primary-action:disabled,
.close-action:disabled,
.icon-button:disabled {
  opacity: 0.36;
  cursor: not-allowed;
}

.rule-lines {
  display: grid;
  gap: 6px;
}

.rule-line {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 66px 26px;
  align-items: center;
  gap: 5px;
}

.rule-line input {
  width: 66px;
  padding: 0 5px;
  text-align: center;
}

.rule-line select,
.direction-field select {
  min-height: 30px;
  padding: 0 5px;
  font-size: 11px;
}

.direction-field {
  justify-content: space-between;
  gap: 8px;
  margin-top: 3px;
}

.icon-button {
  width: 26px;
  height: 28px;
  padding: 0;
  border-radius: 7px;
  color: rgba(var(--text-color), 0.56);
  background: transparent;
}

.icon-button:hover:not(:disabled) {
  color: rgb(var(--text-color));
  background: rgba(var(--primary-color), 0.1);
}

.empty-rule-line {
  margin: 0;
  color: rgba(var(--text-color), 0.38);
  font-size: 10px;
}

.rule-actions {
  flex-wrap: wrap;
  gap: 7px;
  margin-top: auto;
}

.secondary-action,
.primary-action,
.close-action {
  min-height: 34px;
  padding: 0 11px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
}

.secondary-action {
  color: rgba(var(--text-color), 0.78);
  background: rgba(var(--outline-color), 0.1);
}

.primary-action {
  color: rgb(var(--global-inverse-color));
  background: rgba(var(--primary-color), 0.5);
}

.order-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  padding: 18px 28px 20px;
  gap: 12px;
}

.order-toolbar {
  flex: 0 0 auto;
}

.order-toolbar p {
  margin-top: 4px;
}

.order-list {
  display: grid;
  align-content: start;
  gap: 3px;
  min-width: 0;
  min-height: 0;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  list-style: none;
}

.order-row {
  display: grid;
  grid-template-columns: 28px 26px 40px minmax(0, 1fr) 54px auto;
  align-items: center;
  gap: 9px;
  min-height: 56px;
  padding: 5px 9px;
  border: 1px solid transparent;
  border-radius: 10px;
  background: transparent;
  transition: background-color 160ms ease, opacity 160ms ease;
}

.order-row:hover,
.order-row.dragging {
  background: rgba(var(--primary-color), 0.09);
}

.order-row.dragging {
  opacity: 0.52;
}

.order-index {
  color: rgba(var(--text-color), 0.42);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.drag-handle {
  display: grid;
  place-items: center;
  width: 26px;
  height: 28px;
  padding: 0;
  border: 0;
  color: rgba(var(--text-color), 0.36);
  background: transparent;
  cursor: grab;
}

.drag-handle:active {
  cursor: grabbing;
}

.order-cover {
  width: 40px;
  height: 40px;
  border-radius: 6px;
  object-fit: cover;
}

.order-song-copy {
  display: grid;
  min-width: 0;
  gap: 3px;
}

.order-song-copy strong,
.order-song-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.order-song-copy strong {
  color: rgb(var(--text-color));
  font-size: 13px;
  font-weight: 650;
}

.order-song-copy small,
.order-song-duration {
  color: rgba(var(--text-color), 0.54);
  font-size: 11px;
}

.order-song-duration {
  text-align: right;
}

.row-actions {
  display: flex;
  gap: 2px;
}

.order-empty {
  display: grid;
  flex: 1 1 auto;
  place-content: center;
  gap: 5px;
  min-height: 160px;
  color: rgba(var(--text-color), 0.5);
  text-align: center;
}

.order-empty h3,
.order-empty p {
  margin: 0;
}

.order-empty h3 {
  color: rgb(var(--text-color));
  font-size: 13px;
}

.order-empty p {
  font-size: 11px;
}

.order-footer {
  flex: 0 0 auto;
  padding-top: 10px;
  border-top: 1px solid rgba(var(--outline-color), 0.12);
}

.close-action {
  color: rgba(var(--text-color), 0.58);
  background: transparent;
}

.footer-actions {
  justify-content: flex-end;
  gap: 8px;
}

.clone-dialog {
  display: grid;
  gap: 14px;
  padding: 20px;
}

.clone-dialog .dialog-header h2 {
  margin: 0;
  color: rgb(var(--text-color));
  font-size: 16px;
}

.clone-dialog .dialog-input {
  width: 100%;
  min-height: 40px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.dialog-button {
  min-height: 34px;
  padding: 0 13px;
  border: 0;
  border-radius: 8px;
  font: inherit;
  font-size: 12px;
  font-weight: 650;
  cursor: pointer;
}

.dialog-button.secondary {
  color: rgba(var(--text-color), 0.68);
  background: rgba(var(--outline-color), 0.1);
}

.dialog-button.primary {
  color: rgb(var(--global-inverse-color));
  background: rgb(var(--primary-color));
}

@media (max-width: 840px) {
  .order-editor {
    grid-template-columns: minmax(220px, 0.8fr) minmax(0, 1.4fr);
  }

  .order-main {
    padding-right: 18px;
    padding-left: 18px;
  }

  .order-row {
    grid-template-columns: 24px 24px 36px minmax(0, 1fr) auto;
  }

  .order-song-duration {
    display: none;
  }
}

@media (max-width: 640px) {
  .order-editor {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .order-rule-panel {
    overflow: visible;
  }

  .order-main {
    min-height: 420px;
    padding: 16px 18px 18px;
  }
}
</style>
