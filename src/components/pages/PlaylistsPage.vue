<template>
  <MusicLibrary
    v-if="selectedPlaylist"
    :music-library="playlistLibrary"
    :title="selectedPlaylist.name"
    :description="playlistDescription"
    :header-controls="headerControls"
    :primary-action-label="t('playlistsPage.manageSongsTitle')"
    :primary-action-clickable="true"
    :show-play-all="true"
    @header-control-click="handleHeaderControlClick"
    @primary-action="openSongPicker"
    @play-all="playAll"
    @song-play="playSong"
  />

  <ManageSongsDialog v-if="songPickerContext" v-model:open="isSongPickerDialogOpen"
    v-model="songPickerValue" :playlist-name="songPickerContext.playlist.name"
    :songs="libraryStore.tracks.value" :sources="playlistSources"
    :existing-song-ids="songPickerContext.existingSongIds"
    :current-playlist-id="songPickerContext.playlist.id" :initial-mode="songPickerContext.initialMode"
    :initial-rule="songPickerContext.savedRule" :valid="isSongPickerValueValid" :saving="isSavingSongs"
    @save="saveSongPicker" @order-saved="handleOrderSaved" />

  <Dialog v-model="isModeConfirmDialogOpen" width="460" :aria-labelledby="'playlist-mode-confirm-title'"
    :aria-describedby="'playlist-mode-confirm-message'" :close-on-backdrop="false" @close="cancelModeChange">
    <div class="dialog-content">
      <header class="dialog-header">
        <h2 id="playlist-mode-confirm-title">{{ modeConfirmTitle }}</h2>
      </header>
      <p id="playlist-mode-confirm-message" class="dialog-message">{{ modeConfirmMessage }}</p>
      <footer class="dialog-actions">
        <button type="button" class="dialog-button secondary" @click="cancelModeChange">
          {{ t('dialog.actions.cancel') }}
        </button>
        <button type="button" class="dialog-button primary" @click="confirmModeChange">
          {{ modeConfirmLabel }}
        </button>
      </footer>
    </div>
  </Dialog>

  <Dialog v-model="isRenameDialogOpen" width="460" :aria-labelledby="'rename-playlist-dialog-title'">
    <form class="dialog-content" @submit.prevent="submitRenamePlaylist">
      <header class="dialog-header">
        <h2 id="rename-playlist-dialog-title">{{ t('playlistsPage.renameTitle') }}</h2>
      </header>
      <input v-model="renameValue" class="dialog-input" type="text"
        :placeholder="t('dialog.playlist.createPlaceholder')" :disabled="isRenamingPlaylist" autofocus />
      <footer class="dialog-actions">
        <button type="button" class="dialog-button secondary" :disabled="isRenamingPlaylist"
          @click="isRenameDialogOpen = false">
          {{ t('dialog.actions.cancel') }}
        </button>
        <button type="submit" class="dialog-button primary"
          :disabled="isRenamingPlaylist || !renameValue.trim()">
          {{ t('playlistsPage.renameConfirm') }}
        </button>
      </footer>
    </form>
  </Dialog>

  <Dialog v-model="isDeleteDialogOpen" width="460" :aria-labelledby="'delete-playlist-dialog-title'"
    :aria-describedby="'delete-playlist-dialog-message'">
    <div class="dialog-content">
      <header class="dialog-header">
        <h2 id="delete-playlist-dialog-title">{{ t('playlistsPage.deleteTitle') }}</h2>
      </header>
      <p id="delete-playlist-dialog-message" class="dialog-message">
        {{ selectedPlaylist ? t('playlistsPage.deleteMessage', { name: selectedPlaylist.name }) : '' }}
      </p>
      <footer class="dialog-actions">
        <button type="button" class="dialog-button secondary" :disabled="isDeletingPlaylist"
          @click="isDeleteDialogOpen = false">
          {{ t('dialog.actions.cancel') }}
        </button>
        <button type="button" class="dialog-button danger" :disabled="isDeletingPlaylist"
          @click="confirmDeletePlaylist">
          {{ t('playlistsPage.delete') }}
        </button>
      </footer>
    </div>
  </Dialog>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import ManageSongsDialog from '@/components/library/ManageSongsDialog.vue'
import Dialog from '@/components/ui/Dialog.vue'
import MusicLibrary from '@/components/library/MusicLibrary.vue'
import { useLibraryStore } from '@/stores/libraryStore.js'
import { useI18n } from '@/i18n/index.js'
import { isPlaylistComposerValueValid, sourceKey } from '@/utils/playlistRule.js'

const props = defineProps({
  playlists: {
    type: Array,
    default: () => []
  },
  selectedPlaylistId: {
    type: String,
    default: ''
  }
})

const emit = defineEmits([
  'playlist-select',
  'playlist-song-play',
  'user-playlist-play',
  'navigate'
])

const { t } = useI18n()
const libraryStore = useLibraryStore()
const playlistSongs = ref([])
const isSavingSongs = ref(false)
const isSongPickerDialogOpen = ref(false)
const songPickerContext = ref(null)
const songPickerValue = ref({ mode: 'static', trackIds: [] })
const isModeConfirmDialogOpen = ref(false)
const pendingSongPickerValue = ref(null)
const isRenameDialogOpen = ref(false)
const renameValue = ref('')
const isRenamingPlaylist = ref(false)
const isDeleteDialogOpen = ref(false)
const isDeletingPlaylist = ref(false)
let tracksRequestId = 0

const selectedPlaylist = computed(() =>
  props.playlists.find((playlist) => playlist.id === props.selectedPlaylistId) || null
)

const playlistLibrary = computed(() => ({
  totalSongs: playlistSongs.value.length,
  totalDuration: formatDuration(playlistSongs.value),
  songs: playlistSongs.value
}))

const playlistDescription = computed(() =>
  `${playlistSongs.value.length} songs · ${formatDuration(playlistSongs.value)}`
)

const playlistSources = computed(() => [
  {
    key: sourceKey({ kind: 'library', id: null }),
    kind: 'library',
    id: null,
    name: t('library.title'),
    icon: 'library.svg',
    trackCount: libraryStore.tracks.value.length
  },
  ...props.playlists.map((playlist) => ({
    key: sourceKey({ kind: 'playlist', id: playlist.id }),
    kind: 'playlist',
    id: playlist.id,
    name: playlist.name,
    icon: 'playlist.svg',
    trackCount: Number(playlist.trackCount) || 0,
    type: playlist.type || 'static'
  })),
  ...libraryStore.tags.value.map((tag) => ({
    key: sourceKey({ kind: 'tag', id: tag.id }),
    kind: 'tag',
    id: tag.id,
    name: tag.label,
    icon: 'bookmark.svg',
    trackCount: Number(tag.trackCount) || 0
  }))
])

const headerControls = computed(() => [
  ...(selectedPlaylist.value?.type === 'dynamic'
    ? [{ id: 'refresh', icon: 'restore.svg', label: t('playlistsPage.refresh'), selected: false }]
    : []),
  { id: 'rename', icon: 'setting.svg', label: t('playlistsPage.rename'), selected: false },
  { id: 'delete', icon: 'delete.svg', label: t('playlistsPage.delete'), selected: false }
])

const isDynamicValueValid = (value) => value?.mode === 'dynamic'
  && value.previewReady === true
  && isPlaylistComposerValueValid(value)

const isSongPickerValueValid = computed(() => {
  const context = songPickerContext.value
  const value = songPickerValue.value
  if (!context || !value) return false
  return value.mode === 'static'
    ? (context.initialMode === 'dynamic' ? Array.isArray(value.trackIds) : isPlaylistComposerValueValid(value))
    : isDynamicValueValid(value)
})

const modeConfirmTitle = computed(() => pendingSongPickerValue.value?.mode === 'dynamic'
  ? t('playlistsPage.switchToDynamicTitle')
  : t('playlistsPage.switchToStaticTitle'))

const modeConfirmMessage = computed(() => pendingSongPickerValue.value?.mode === 'dynamic'
  ? t('playlistsPage.switchToDynamicMessage')
  : t('playlistsPage.switchToStaticMessage'))

const modeConfirmLabel = computed(() => pendingSongPickerValue.value?.mode === 'dynamic'
  ? t('playlistsPage.dynamicMode')
  : t('playlistsPage.staticMode'))

const formatDuration = (songs = []) => {
  const durationMs = songs.reduce((total, song) => total + (Number(song.durationMs) || 0), 0)
  const totalSeconds = Math.max(0, Math.floor(durationMs / 1000))
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  return `${hours}h ${minutes}m ${seconds}s`
}

const loadSelectedTracks = async (playlistId) => {
  const requestId = ++tracksRequestId
  playlistSongs.value = []

  if (!playlistId) return

  try {
    const songs = (await libraryStore.playlistOrder(playlistId)).tracks || []
    if (requestId === tracksRequestId && props.selectedPlaylistId === playlistId) {
      playlistSongs.value = songs
    }
  } catch (error) {
    if (requestId === tracksRequestId) {
      playlistSongs.value = []
      console.error('加载播放列表失败:', error)
    }
  }
}

const syncSelectedPlaylist = () => {
  const playlist = selectedPlaylist.value
  if (!playlist) {
    playlistSongs.value = []
    if (props.selectedPlaylistId) {
      emit('playlist-select', null)
      emit('navigate', 'library')
    }
    return
  }

  emit('playlist-select', playlist)
  void loadSelectedTracks(playlist.id)
}

const handleHeaderControlClick = (control) => {
  if (control?.id === 'refresh') return refreshPlaylist()
  if (control?.id === 'rename') return renamePlaylist()
  if (control?.id === 'delete') return deletePlaylist()
}

const refreshPlaylist = async () => {
  if (selectedPlaylist.value?.type !== 'dynamic') return
  await loadSelectedTracks(selectedPlaylist.value.id)
}

const openSongPicker = async () => {
  const playlist = selectedPlaylist.value
  if (!playlist || isSavingSongs.value || isSongPickerDialogOpen.value) return

  let savedRule = null
  if (playlist.type === 'dynamic') {
    try {
      savedRule = (await libraryStore.getPlaylistRule(playlist.id))?.rule || null
    } catch (error) {
      console.error('读取动态播放列表规则失败:', error)
      return
    }
  }

  const initialMode = playlist.type === 'dynamic' ? 'dynamic' : 'static'
  songPickerValue.value = initialMode === 'dynamic'
    ? {
        mode: 'dynamic',
        rule: savedRule,
        trackIds: playlistSongs.value.map((song) => song.id),
        tracks: [...playlistSongs.value]
      }
    : { mode: 'static', trackIds: [] }
  songPickerContext.value = {
    playlist,
    initialMode,
    savedRule,
    existingSongIds: playlist.type === 'static' ? playlistSongs.value.map((song) => song.id) : []
  }
  isSongPickerDialogOpen.value = true
}

const saveSongPicker = () => {
  const context = songPickerContext.value
  let value = songPickerValue.value
  if (!context || !isSongPickerValueValid.value || isSavingSongs.value) return

  if (value.mode === 'static' && context.initialMode === 'static') {
    const selectedIds = new Set(value.trackIds || [])
    const currentIds = playlistSongs.value.map((song) => song.id)
    const currentIdSet = new Set(currentIds)
    const newlySelectedIds = (value.trackIds || []).filter((id) => !currentIdSet.has(id))
    value = {
      ...value,
      trackIds: [
        ...currentIds.filter((id) => selectedIds.has(id)),
        ...newlySelectedIds
      ]
    }
    songPickerValue.value = value
  }

  if (value.mode !== context.initialMode) {
    pendingSongPickerValue.value = {
      ...value,
      trackIds: [...(value.trackIds || [])],
      tracks: [...(value.tracks || [])],
      rule: value.rule ? { ...value.rule, steps: value.rule.steps.map((step) => ({ ...step })) } : value.rule
    }
    isSongPickerDialogOpen.value = false
    isModeConfirmDialogOpen.value = true
    return
  }

  isSongPickerDialogOpen.value = false
  void persistSongPicker(value)
}

const persistSongPicker = async (value) => {
  const context = songPickerContext.value
  if (!context || !value) return
  isSavingSongs.value = true
  try {
    let needsReload = true
    if (value.mode === 'dynamic') {
      await libraryStore.savePlaylistRule(context.playlist.id, value.rule)
      needsReload = true
    } else {
      await libraryStore.materializePlaylist(context.playlist.id, value.trackIds)
    }
    if (needsReload) await loadSelectedTracks(context.playlist.id)
  } catch (error) {
    console.error('添加歌曲到播放列表失败:', error)
  } finally {
    isSavingSongs.value = false
    pendingSongPickerValue.value = null
    songPickerContext.value = null
  }
}

const cancelModeChange = () => {
  isModeConfirmDialogOpen.value = false
  pendingSongPickerValue.value = null
}

const confirmModeChange = () => {
  const value = pendingSongPickerValue.value
  if (!value) return
  isModeConfirmDialogOpen.value = false
  void persistSongPicker(value)
}

const playAll = () => {
  if (!selectedPlaylist.value || !playlistSongs.value.length) return
  emit('user-playlist-play', {
    playlist: selectedPlaylist.value,
    songs: [...playlistSongs.value]
  })
}

const playSong = (song) => {
  if (!selectedPlaylist.value || !song) return
  emit('playlist-song-play', {
    playlist: selectedPlaylist.value,
    song,
    songs: [...playlistSongs.value]
  })
}

const handleOrderSaved = () => {
  if (selectedPlaylist.value) void loadSelectedTracks(selectedPlaylist.value.id)
}

const renamePlaylist = () => {
  const playlist = selectedPlaylist.value
  if (!playlist) return
  renameValue.value = playlist.name
  isRenameDialogOpen.value = true
}

const submitRenamePlaylist = async () => {
  const playlist = selectedPlaylist.value
  const name = renameValue.value.trim()
  if (!playlist || !name || name === playlist.name || isRenamingPlaylist.value) return

  isRenamingPlaylist.value = true
  try {
    await libraryStore.renamePlaylist(playlist.id, name)
    isRenameDialogOpen.value = false
  } catch (error) {
    console.error('重命名播放列表失败:', error)
  } finally {
    isRenamingPlaylist.value = false
  }
}

const deletePlaylist = () => {
  const playlist = selectedPlaylist.value
  if (!playlist) return
  isDeleteDialogOpen.value = true
}

const confirmDeletePlaylist = async () => {
  const playlist = selectedPlaylist.value
  if (!playlist || isDeletingPlaylist.value) return

  isDeletingPlaylist.value = true
  try {
    await libraryStore.removePlaylist(playlist.id)
    isDeleteDialogOpen.value = false
    emit('playlist-select', null)
    emit('navigate', 'library')
  } catch (error) {
    console.error('删除播放列表失败:', error)
  } finally {
    isDeletingPlaylist.value = false
  }
}

watch(
  () => [props.selectedPlaylistId, props.playlists.map((playlist) => playlist.id).join('|')],
  syncSelectedPlaylist,
  { immediate: true }
)
</script>

<style scoped></style>
