<template>
  <PageLayout>
    <template #header>
      <div class="music-banner">
        <div class="image-container">
          <MotionTransition variant="banner">
            <img :key="bannerImage" class="background-image" :src="bannerImage" referrerpolicy="no-referrer" />
          </MotionTransition>
        </div>
        <div class="banner-content">
          <div class="title">{{ t('app.name') }}</div>
          <MotionDiv :key="`title-${providerKey}`" class="library-title" :initial="{ opacity: 0, y: 10 }"
            :animate="{ opacity: 1, y: 0 }" :transition="softTransition">
            {{ provider?.name || providerKey }}
          </MotionDiv>
          <MotionDiv :key="`description-${providerKey}`" class="description" :initial="{ opacity: 0, y: 10 }"
            :animate="{ opacity: 1, y: 0 }" :transition="softTransition">
            {{ description }}
          </MotionDiv>
        </div>
        <div v-if="provider?.tagId" class="controls-row">
          <MotionButton type="button" class="control-btn" :while-hover="{ y: -1 }" :while-press="{ scale: 0.96 }"
            :transition="microTransition" @click="openRenameTag">
            <Icon src="/assets/setting.svg" size="xs" />
            <span>{{ t('playlistsPage.rename') }}</span>
          </MotionButton>
          <MotionButton type="button" class="control-btn" :while-hover="{ y: -1 }" :while-press="{ scale: 0.96 }"
            :transition="microTransition" @click="openDeleteTag">
            <Icon src="/assets/delete.svg" size="xs" />
            <span>{{ t('playlistsPage.delete') }}</span>
          </MotionButton>
        </div>
        <div class="banner-status-row">
          <MotionButton v-if="refreshing && progress.jobId" type="button" class="control-btn cancel-btn"
            :while-hover="{ y: -1 }" :while-press="{ scale: 0.96 }" :transition="microTransition"
            @click="cancelRefresh">
            {{ t('tagProvider.cancel') }}
          </MotionButton>
          <AnimatePresence>
            <MotionDiv v-if="statusMessage" :key="statusMessage" class="status" :class="statusClass"
              :initial="{ opacity: 0, y: 8 }" :animate="{ opacity: 1, y: 0 }" :exit="{ opacity: 0, y: -8 }"
              :transition="microTransition">
              {{ statusMessage }}
            </MotionDiv>
          </AnimatePresence>
        </div>
      </div>
    </template>

    <div v-if="error" class="provider-error">{{ error }}</div>

    <div ref="pageRef" class="library-content-with-alphabet">
      <div class="song-list-container">
          <SongList :songs="loading ? [] : listSongs" show-header :primary-action-label="primaryActionLabel"
            :primary-action-clickable="true" :primary-action-disabled="primaryActionDisabled"
            :show-play-all="true" :group-of="groupOf" :sort-value-of="sortValueOf"
            :context-source="contextSource" :song-badge-of="songBadgeOf"
            @primary-action="handlePrimaryAction" @play-all="playAll" @song-play="handleSongPlay"
            @song-context-menu="$emit('song-context-menu', $event)"
            @group-label-click="handleGroupLabelClick" @filter-click="focusSearch">
            <template #primary-icon>
              <Plus v-if="isManual" size="13" />
              <RefreshCw v-else size="12" />
            </template>
            <template #header-extra>
              <MotionButton type="button" class="wiki-btn" :while-hover="{ y: -1 }"
                :while-press="{ scale: 0.96 }" :transition="microTransition" @click="openWiki">
                <BookOpen size="13" />
                <span>{{ t('tagProvider.wiki') }}</span>
              </MotionButton>
            </template>
            <template #header-search>
              <Search :size="14" :stroke-width="1.8" class="header-search-icon" />
              <input ref="searchInput" v-model="search" type="search"
                :placeholder="t('playlistsPage.searchPlaceholder')" />
            </template>
          </SongList>
        <div v-if="loading" class="provider-empty">{{ t('playlistsPage.loadingSongs') }}</div>
        <MotionDiv v-else-if="!listSongs.length" class="provider-empty" :initial="{ opacity: 0, y: 14 }"
          :animate="{ opacity: 1, y: 0 }" :transition="softTransition">
          <h3>{{ search ? t('playlistsPage.emptyAvailable') : (isManual ? t('tagProvider.emptyManual') : t('tagProvider.empty')) }}</h3>
          <p>{{ search ? t('playlistsPage.clearSearch') : description }}</p>
        </MotionDiv>
      </div>
      <AlphabetFilter v-if="!loading && listSongs.length && !groupOf" :active-initial="activeInitial"
        :top-offset="alphabetTopOffset" :available-initials="availableInitials" @select="handleAlphabetSelect" />
    </div>

    <Dialog v-model="isWikiOpen" :width="640" height="min(640px, calc(100dvh - 36px))"
      :close-on-backdrop="!wikiSaving" :close-on-escape="!wikiSaving" :aria-labelledby="'tag-wiki-title'">
      <div class="dialog-content wiki-dialog">
        <header class="dialog-header">
          <h2 :id="'tag-wiki-title'">{{ t('tagProvider.wiki') }} · {{ provider?.name || providerKey }}</h2>
        </header>
        <div v-if="!isManual" class="wiki-tabs" role="tablist" :aria-label="t('tagProvider.wiki')">
          <button id="wiki-tab-original" type="button" role="tab" :aria-selected="wikiTab === 'original'"
            aria-controls="wiki-original" :tabindex="wikiTab === 'original' ? 0 : -1"
            @click="wikiTab = 'original'" @keydown.right.prevent="selectWikiTab('plugin')">
            {{ t('tagProvider.wikiOriginal') }}
          </button>
          <button id="wiki-tab-plugin" type="button" role="tab" :aria-selected="wikiTab === 'plugin'"
            aria-controls="wiki-plugin" :tabindex="wikiTab === 'plugin' ? 0 : -1"
            @click="selectWikiTab('plugin')" @keydown.left.prevent="selectWikiTab('original')">
            {{ t('tagProvider.wikiPluginProvided') }}
          </button>
        </div>
        <div v-show="wikiTab === 'original'" id="wiki-original" class="wiki-panel" role="tabpanel"
          :aria-labelledby="!isManual ? 'wiki-tab-original' : 'tag-wiki-title'">
          <div v-if="wikiLoading" class="wiki-state">{{ t('playlistsPage.loadingSongs') }}</div>
          <textarea v-else-if="wikiEditing" v-model="wikiDraft" class="wiki-editor" maxlength="32768"
            :aria-label="t('tagProvider.wikiOriginal')" :disabled="wikiSaving" />
          <template v-else>
            <MarkdownContent v-if="wikiContent" :source="wikiContent" />
            <div v-else class="wiki-state">{{ wikiError || t('tagProvider.wikiEmpty') }}</div>
          </template>
        </div>
        <div v-show="wikiTab === 'plugin'" id="wiki-plugin" class="wiki-panel" role="tabpanel"
          aria-labelledby="wiki-tab-plugin">
          <div v-if="pluginWikiLoading" class="wiki-state">{{ t('playlistsPage.loadingSongs') }}</div>
          <MarkdownContent v-else-if="pluginWikiContent" :source="pluginWikiContent" />
          <div v-else class="wiki-state">{{ pluginWikiError || t('tagProvider.wikiEmpty') }}</div>
        </div>
        <p v-if="wikiSaveError" class="provider-error">{{ wikiSaveError }}</p>
        <footer v-if="wikiTab === 'original' && provider?.tagId && !wikiLoading && !wikiError" class="wiki-actions">
          <template v-if="wikiEditing">
            <button type="button" :disabled="wikiSaving" @click="wikiEditing = false">{{ t('dialog.actions.cancel') }}</button>
            <button type="button" :disabled="wikiSaving" @click="saveWiki">{{ t('tagProvider.wikiSave') }}</button>
          </template>
          <button v-else type="button" @click="editWiki">{{ t('tagProvider.wikiEdit') }}</button>
        </footer>
      </div>
    </Dialog>

    <Dialog v-model="isAddSongsDialogOpen" :width="520" :aria-labelledby="'tag-add-songs-title'">
      <div class="dialog-content">
        <header class="dialog-header">
          <h2 :id="'tag-add-songs-title'">{{ t('tagProvider.addSongsTitle', { name: provider?.name || '' }) }}</h2>
        </header>
        <input v-model="addSearch" class="dialog-input" type="search"
          :placeholder="t('playlistsPage.searchPlaceholder')" />
        <div class="add-song-list">
          <VirtualList :items="addCandidates" :item-height="48" v-slot="{ item: song }">
            <MotionDiv class="add-song-row"
              :initial="{ opacity: 0, y: 10 }" :animate="{ opacity: 1, y: 0 }"
              :while-hover="{ backgroundColor: 'rgba(var(--surface-color), 0.5)' }" :transition="microTransition">
              <img :src="song.cover" :alt="song.title" loading="lazy" decoding="async" />
              <div class="add-song-meta">
                <strong>{{ song.title }}</strong>
                <small>{{ song.artist || t('player.unknownArtist') }}</small>
              </div>
              <MotionButton type="button" class="add-song-toggle" :class="{ added: isTagged(song.id) }"
                :disabled="mutatingTag" :while-hover="{ y: -1 }" :while-press="{ scale: 0.95 }"
                :transition="microTransition" @click="toggleSong(song)">
                {{ isTagged(song.id) ? t('tagProvider.removeSong') : t('playlistsPage.addSong') }}
              </MotionButton>
            </MotionDiv>
          </VirtualList>
          <p v-if="!addCandidates.length" class="add-song-empty">{{ t('playlistsPage.emptyAvailable') }}</p>
        </div>
      </div>
    </Dialog>

    <Dialog v-model="isRenameTagDialogOpen" width="460" :aria-labelledby="'rename-tag-dialog-title'">
      <form class="dialog-content" @submit.prevent="submitRenameTag">
        <header class="dialog-header">
          <h2 :id="'rename-tag-dialog-title'">{{ t('tagProvider.renameTitle') }}</h2>
        </header>
        <input v-model="renameTagValue" class="dialog-input" type="text"
          :placeholder="t('dialog.tag.createPlaceholder')" :disabled="isRenamingTag" autofocus />
        <p v-if="renameTagError" class="provider-error">{{ renameTagError }}</p>
        <footer class="dialog-actions">
          <button type="button" class="dialog-button secondary" :disabled="isRenamingTag"
            @click="isRenameTagDialogOpen = false">
            {{ t('dialog.actions.cancel') }}
          </button>
          <button type="submit" class="dialog-button primary" :disabled="isRenamingTag || !renameTagValue.trim()">
            {{ t('playlistsPage.renameConfirm') }}
          </button>
        </footer>
      </form>
    </Dialog>

    <Dialog v-model="isDeleteTagDialogOpen" width="460" :aria-labelledby="'delete-tag-dialog-title'"
      :aria-describedby="'delete-tag-dialog-message'">
      <div class="dialog-content">
        <header class="dialog-header">
          <h2 :id="'delete-tag-dialog-title'">{{ t('tagProvider.deleteTitle') }}</h2>
        </header>
        <p :id="'delete-tag-dialog-message'" class="dialog-message">
          {{ provider ? t('tagProvider.deleteMessage', { name: provider.name || providerKey }) : '' }}
        </p>
        <footer class="dialog-actions">
          <button type="button" class="dialog-button secondary" :disabled="isDeletingTag"
            @click="isDeleteTagDialogOpen = false">
            {{ t('dialog.actions.cancel') }}
          </button>
          <button type="button" class="dialog-button danger" :disabled="isDeletingTag" @click="confirmDeleteTag">
            {{ t('playlistsPage.delete') }}
          </button>
        </footer>
      </div>
    </Dialog>
  </PageLayout>
</template>

<script setup>
import { computed, inject, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { Plus, RefreshCw, Search, BookOpen } from '@lucide/vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import MotionTransition from '@/components/ui/MotionTransition.vue'
import Dialog from '@/components/ui/Dialog.vue'
import Icon from '@/components/ui/Icon.vue'
import SongList from '@/components/library/SongList.vue'
import VirtualList from '@/components/ui/VirtualList.vue'
import AlphabetFilter from '@/components/ui/AlphabetFilter.vue'
import { getLocale, useI18n } from '@/i18n/index.js'
import Tip from '@/components/notification/Tip.vue'
import { useLibraryStore } from '@/stores/libraryStore.js'
import { listenToPluginEvents } from '@/services/pluginApi.js'
import { getAvailableInitials } from '@/utils/alphabet.js'
import { useAlphabetNavigation } from '@/utils/useAlphabetNavigation.js'
import MarkdownContent from '@/components/ui/MarkdownContent.vue'
import { isManualTag, formatTagBadge } from '@/utils/tagProvider.js'
import { mediaApi } from '@/services/mediaApi.js'
import { INSTANT_MOTION, MICRO_SPRING, SOFT_SPRING } from '@/utils/motion.js'

const props = defineProps({
  provider: { type: Object, default: null },
  pluginRuntime: { type: Object, required: true }
})
const emit = defineEmits(['song-play', 'song-context-menu', 'navigate'])
const { t } = useI18n()
const currentSong = inject('currentSong')
const notificationRef = inject('notification', null)
const libraryStore = useLibraryStore()
const search = ref('')
const loading = ref(false)
const refreshing = ref(false)
const mutatingTag = ref(false)
const error = ref('')
const progress = ref({ completed: 0, total: 0 })
const songs = ref([])
const searchInput = ref(null)
const isAddSongsDialogOpen = ref(false)
const addSearch = ref('')
const isWikiOpen = ref(false)
const wikiLoading = ref(false)
const wikiContent = ref('')
const wikiError = ref('')
const wikiTab = ref('original')
const wikiDraft = ref('')
const wikiEditing = ref(false)
const wikiSaving = ref(false)
const wikiSaveError = ref('')
const pluginWikiContent = ref('')
const pluginWikiLoading = ref(false)
const pluginWikiError = ref('')
let pluginWikiRequest = 0
let pluginWikiLoaded = false
const providerKey = computed(() => props.provider?.key || '')
const isManual = computed(() => isManualTag(props.provider))
const bannerImage = computed(() => currentSong.value?.cover || '/assets/cover.jpg')
const description = computed(() => props.provider?.help
  || (isManual.value ? t('tagProvider.manualDescription') : t('sidebar.myTags')))
const statusMessage = computed(() => {
  if (isManual.value) return ''
  if (!props.provider?.sourceAvailable || !props.provider?.enabled) return t('tagProvider.unavailable')
  if (props.provider?.stale) return t('tagProvider.stale')
  return ''
})
const statusClass = computed(() => (!props.provider?.sourceAvailable || !props.provider?.enabled ? 'unavailable' : 'stale'))
const listSongs = computed(() => songs.value.map((item) => {
  const rawValue = item.values?.length ? item.values[0].value : item.value
  const number = typeof rawValue === 'boolean' || rawValue === null || rawValue === undefined || rawValue === ''
    ? NaN
    : Number(rawValue)
  return {
    ...item.track,
    tagValue: Number.isFinite(number) ? Math.round(number * 10) / 10 : rawValue,
    tagStatus: item.values?.some((entry) => entry.status !== 'ready') ? 'stale' : 'ready'
  }
}).filter((song) => !search.value.trim() || [song.title, song.artist, song.album]
  .some((field) => String(field || '').toLowerCase().includes(search.value.trim().toLowerCase()))))
const numericBucket = (value) => {
  const number = Number(value)
  if (!Number.isFinite(number)) return String(value ?? '–')
  const start = Math.floor(number / 10) * 10
  return start >= 90 ? '90+' : `${start}–${start + 9}`
}
const groupOf = computed(() => {
  const valueType = props.provider?.valueType
  if (valueType === 'number') return (song) => numericBucket(song.tagValue)
  if (valueType === 'enum' || valueType === 'boolean') return (song) => String(song.tagValue ?? '–')
  return null
})
const sortValueOf = (song) => song.tagValue
const primaryActionLabel = computed(() => {
  if (isManual.value) return t('tagProvider.manageSongs')
  return refreshing.value ? `${progress.value.completed}/${progress.value.total || '?'}` : t('playlistsPage.refresh')
})
const primaryActionDisabled = computed(() => !props.provider || (!isManual.value
  && (refreshing.value || !props.provider.sourceAvailable || !props.provider.enabled)))
const taggedIds = computed(() => new Set(songs.value.map((item) => item.track.id)))
const addCandidates = computed(() => {
  const keyword = addSearch.value.trim().toLowerCase()
  return libraryStore.tracks.value
    .filter((track) => !keyword
      || [track.title, track.artist, track.album].some((field) => String(field || '').toLowerCase().includes(keyword)))
})
let unlistenPluginEvents = () => { }
let loadRequest = 0
let wikiRequest = 0

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const softTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)

const pageRef = ref(null)
const availableInitials = computed(() => getAvailableInitials(listSongs.value, (song) => song.title))
const { activeInitial, alphabetTopOffset, handleAlphabetSelect, handleGroupLabelClick } = useAlphabetNavigation(
  pageRef,
  availableInitials
)

const toSong = (track) => {
  const source = libraryStore.tracks.value.find((item) => item.id === track.id)
  return {
    ...track,
    cover: source?.cover || '/assets/cover.jpg',
    duration: source?.duration || '00:00',
    url: track.path || track.url || ''
  }
}

const load = async () => {
  if (!providerKey.value) return
  const request = ++loadRequest
  loading.value = true
  error.value = ''
  try {
    const result = await props.pluginRuntime.tagProviderResults(providerKey.value, { limit: 2000 })
    if (request !== loadRequest) return
    const byId = new Map()
    for (const item of result.results || []) {
      const existing = byId.get(item.track.id)
      if (existing) {
        existing.values.push({ key: item.key, value: item.value, status: item.status })
      } else {
        byId.set(item.track.id, {
          ...item,
          values: [{ key: item.key, value: item.value, status: item.status }]
        })
      }
    }
    songs.value = [...byId.values()].map((item) => ({ ...item, track: toSong(item.track) }))
  } catch (cause) {
    if (request !== loadRequest) return
    error.value = cause?.message || String(cause)
    songs.value = []
  } finally {
    if (request === loadRequest) loading.value = false
  }
}

const refresh = async () => {
  if (!providerKey.value || refreshing.value || isManual.value) return
  if (props.provider && (!props.provider.sourceAvailable || !props.provider.enabled)) return
  refreshing.value = true
  error.value = ''
  try {
    const job = await props.pluginRuntime.refreshTagProvider(providerKey.value)
    if (refreshing.value) progress.value = { ...progress.value, total: job?.total || progress.value.total, jobId: job?.jobId }
  } catch (cause) {
    error.value = cause?.message || String(cause)
    refreshing.value = false
  }
}

const cancelRefresh = async () => {
  if (!progress.value.jobId) return
  try {
    await props.pluginRuntime.cancelTagProvider(progress.value.jobId)
  } catch (cause) {
    error.value = cause?.message || String(cause)
  }
}

const handlePrimaryAction = () => {
  if (isManual.value) openAddSongs()
  else void refresh()
}

// —— 徽标与右键菜单来源 ——
const songBadgeOf = (song) => formatTagBadge(props.provider?.name || '', song.tagValue)
const contextSource = computed(() => props.provider?.tagId
  ? { type: 'tag', id: props.provider.tagId, name: props.provider?.name || '' }
  : null)

// —— 重命名 / 删除标签（与播放列表页一致）——
const isRenameTagDialogOpen = ref(false)
const renameTagValue = ref('')
const renameTagError = ref('')
const isRenamingTag = ref(false)
const isDeleteTagDialogOpen = ref(false)
const isDeletingTag = ref(false)

const notifyTag = (tip) => {
  void notificationRef?.value?.addNotification(
    t('contextMenu.menuDone'),
    t('notification.source'),
    Tip,
    null,
    { Tip: tip },
    2600
  ).catch(() => undefined)
}

const openRenameTag = () => {
  if (!props.provider?.tagId) return
  renameTagValue.value = props.provider.name || ''
  renameTagError.value = ''
  isRenameTagDialogOpen.value = true
}

const submitRenameTag = async () => {
  const tagId = props.provider?.tagId
  const name = renameTagValue.value.trim()
  if (!tagId || !name || isRenamingTag.value) return
  isRenamingTag.value = true
  renameTagError.value = ''
  try {
    await libraryStore.renameTag(tagId, name)
    isRenameTagDialogOpen.value = false
    await props.pluginRuntime.refresh().catch(() => undefined)
    notifyTag(t('contextMenu.tagRenamed', { name }))
  } catch (cause) {
    renameTagError.value = cause?.message || String(cause)
  } finally {
    isRenamingTag.value = false
  }
}

const openDeleteTag = () => {
  if (!props.provider?.tagId) return
  isDeleteTagDialogOpen.value = true
}

const confirmDeleteTag = async () => {
  const tagId = props.provider?.tagId
  const name = props.provider?.name || providerKey.value
  if (!tagId || isDeletingTag.value) return
  isDeletingTag.value = true
  try {
    await libraryStore.removeTag(tagId)
    isDeleteTagDialogOpen.value = false
    await props.pluginRuntime.refresh().catch(() => undefined)
    notifyTag(t('contextMenu.tagDeleted', { name }))
    emit('navigate', 'library')
  } catch (cause) {
    error.value = cause?.message || String(cause)
  } finally {
    isDeletingTag.value = false
  }
}

const openAddSongs = () => {
  addSearch.value = ''
  isAddSongsDialogOpen.value = true
}

const focusSearch = () => {
  nextTick(() => searchInput.value?.focus())
}

const openWiki = async () => {
  const request = ++wikiRequest
  isWikiOpen.value = true
  wikiLoading.value = true
  wikiContent.value = ''
  wikiError.value = ''
  wikiTab.value = 'original'
  wikiEditing.value = false
  wikiSaveError.value = ''
  pluginWikiRequest += 1
  pluginWikiLoaded = false
  pluginWikiContent.value = ''
  pluginWikiError.value = ''
  pluginWikiLoading.value = false
  try {
    const result = await props.pluginRuntime.tagProviderWiki(providerKey.value, getLocale())
    if (request === wikiRequest) wikiContent.value = String(result?.wiki || '')
  } catch (cause) {
    if (request === wikiRequest) wikiError.value = cause?.message || String(cause)
  } finally {
    if (request === wikiRequest) wikiLoading.value = false
  }
}

const selectWikiTab = async (tab) => {
  wikiTab.value = tab
  await nextTick()
  document.getElementById(tab === 'plugin' ? 'wiki-tab-plugin' : 'wiki-tab-original')?.focus()
  if (tab !== 'plugin' || pluginWikiLoaded || pluginWikiLoading.value) return
  const request = ++pluginWikiRequest
  pluginWikiLoading.value = true
  pluginWikiError.value = ''
  try {
    const result = await props.pluginRuntime.tagProviderWiki(providerKey.value, getLocale(), true)
    if (request === pluginWikiRequest) {
      pluginWikiContent.value = String(result?.wiki || '')
      pluginWikiLoaded = true
    }
  } catch (cause) {
    if (request === pluginWikiRequest) pluginWikiError.value = cause?.message || String(cause)
  } finally {
    if (request === pluginWikiRequest) pluginWikiLoading.value = false
  }
}

const editWiki = () => {
  wikiDraft.value = wikiContent.value
  wikiSaveError.value = ''
  wikiEditing.value = true
}

const saveWiki = async () => {
  if (!props.provider?.tagId || wikiSaving.value) return
  const key = providerKey.value
  const draft = wikiDraft.value
  wikiSaving.value = true
  wikiSaveError.value = ''
  try {
    await mediaApi.tagWikiSave(props.provider.tagId, draft)
    if (key === providerKey.value) {
      wikiContent.value = draft
      wikiEditing.value = false
    }
    await props.pluginRuntime.refresh().catch(() => undefined)
  } catch (cause) {
    if (key === providerKey.value) wikiSaveError.value = cause?.message || String(cause)
  } finally {
    wikiSaving.value = false
  }
}

const isTagged = (trackId) => taggedIds.value.has(trackId)

const toggleSong = async (song) => {
  if (mutatingTag.value || !props.provider) return
  mutatingTag.value = true
  error.value = ''
  try {
    if (isTagged(song.id)) {
      await libraryStore.untagTrack(song.id, props.provider.tagId || '')
    } else {
      await libraryStore.tagTrack(song.id, props.provider.name || providerKey.value)
    }
    await load()
  } catch (cause) {
    error.value = cause?.message || String(cause)
  } finally {
    mutatingTag.value = false
  }
}

const playAll = () => {
  if (!listSongs.value.length) return
  emit('song-play', { song: listSongs.value[0], songs: listSongs.value })
}

const handleSongPlay = (song) => {
  if (!song) return
  emit('song-play', { song, songs: listSongs.value })
}

const handlePluginEvent = (name, payload) => {
  if (!name.startsWith('tag/analysis-') || payload?.providerKey !== providerKey.value) return
  if (name === 'tag/analysis-progress') {
    refreshing.value = true
    progress.value = {
      completed: Number(payload?.completed) || 0,
      total: Number(payload?.total) || 0,
      jobId: payload?.jobId || progress.value.jobId
    }
    return
  }
  refreshing.value = false
  if (name === 'tag/analysis-finished') {
    progress.value = { ...progress.value, completed: progress.value.total || progress.value.completed }
    void load()
  } else if (name === 'tag/analysis-error') {
    if (payload?.state !== 'cancelled') {
      error.value = payload?.error?.message || payload?.error || t('tagProvider.analysisFailed')
    }
    void load()
  }
}

watch(providerKey, () => {
  wikiRequest += 1
  pluginWikiRequest += 1
  isWikiOpen.value = false
  isAddSongsDialogOpen.value = false
  search.value = ''
  refreshing.value = false
  progress.value = { completed: 0, total: 0 }
  void load()
})

// 曲库刷新（如右键“从标签移除”）后同步重载当前标签的结果
watch(() => libraryStore.state.tracks, () => {
  if (providerKey.value && !loading.value) void load()
})
onMounted(async () => {
  unlistenPluginEvents = await listenToPluginEvents(handlePluginEvent)
  void load()
})

onBeforeUnmount(() => {
  loadRequest += 1
  wikiRequest += 1
  pluginWikiRequest += 1
  unlistenPluginEvents()
})
</script>

<style scoped>
.banner-status-row { display: flex; align-items: center; gap: 8px; position: absolute; right: 18px; bottom: 16px; }
/* 右上角重命名/删除直接复用全局 .controls-row / .control-btn（见 App.vue） */
.control-btn.cancel-btn { color: #e05b5b; opacity: 1; }
.control-btn:disabled, .secondary-action:disabled { cursor: default; opacity: .55; }
.status { font-size: 11px; }.status.stale { color: #d58b2a; }.status.unavailable { color: #e05b5b; }
.library-content-with-alphabet { display: flex; align-items: flex-start; gap: 16px; }
.library-content-with-alphabet .song-list-container { min-width: 0; flex: 1; }
.header-search-icon { color: rgba(var(--text-color), .55); margin-right: -26px; pointer-events: none; z-index: 1; }
.col-search input { padding-left: 30px; }
.provider-empty, .provider-error { padding: 38px 0; color: rgba(var(--text-color), .58); text-align: center; }.provider-empty h3 { color: rgb(var(--text-color)); }.provider-error { padding: 10px; color: #e05b5b; }
.empty-cta { display: inline-flex; margin-top: 14px; }
.dialog-content { display: flex; flex-direction: column; gap: 12px; }
.dialog-header h2 { margin: 0; font-size: 16px; }
.wiki-dialog { display: flex; flex: 1; flex-direction: column; min-height: 0; gap: 14px; }
.wiki-tabs { display: flex; flex: 0 0 auto; gap: 8px; }
.wiki-tabs button, .wiki-actions button { border: 0; border-radius: 7px; padding: 8px 12px; color: rgb(var(--text-color)); background: rgba(var(--outline-color), .12); cursor: pointer; }
.wiki-tabs button[aria-selected="true"] { background: rgba(var(--primary-color), .24); }
.wiki-panel { flex: 1; min-height: 0; overflow-y: auto; }
.wiki-editor { display: block; box-sizing: border-box; width: 100%; height: 100%; min-height: 180px; resize: none; border: 1px solid rgba(var(--outline-color), .3); border-radius: 8px; padding: 12px; color: rgb(var(--text-color)); background: rgba(var(--surface-color), .3); font-family: monospace; line-height: 1.6; }
.wiki-actions { display: flex; flex: 0 0 auto; justify-content: flex-end; gap: 8px; }
.wiki-actions button:disabled { opacity: .5; cursor: default; }
.wiki-state { padding: 18px 0; color: rgba(var(--text-color), .58); text-align: center; font-size: 12px; }
.wiki-btn { display: flex; align-items: center; gap: 8px; border: 0; padding: 0; background: transparent; color: rgba(var(--text-color), .48); font-size: 12px; cursor: pointer; }
.dialog-input { width: 100%; border: 0; border-radius: 6px; padding: 9px 12px; color: rgb(var(--text-color)); background: rgba(var(--outline-color), .08); }
.add-song-list { display: flex; flex-direction: column; gap: 2px; max-height: 320px; overflow-y: auto; }
.add-song-row { display: flex; align-items: center; gap: 10px; border-radius: 8px; padding: 6px 8px; height: 48px; box-sizing: border-box; }
.add-song-row img { width: 34px; height: 34px; border-radius: 5px; object-fit: cover; }
.add-song-meta { display: flex; flex: 1; min-width: 0; flex-direction: column; }
.add-song-meta strong { font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.add-song-meta small { margin-top: 2px; color: rgba(var(--text-color), .55); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.add-song-toggle { border: 0; border-radius: 5px; padding: 5px 10px; color: rgb(var(--text-color)); background: rgba(var(--outline-color), .1); cursor: pointer; font-size: 11px; }
.add-song-toggle.added { color: #e05b5b; }
.add-song-toggle:disabled { opacity: .55; cursor: default; }
.add-song-empty { padding: 18px 0; color: rgba(var(--text-color), .58); text-align: center; font-size: 12px; }
@media (max-width: 768px) {
  .library-content-with-alphabet { gap: 4px; }
}
</style>
