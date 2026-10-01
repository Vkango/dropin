<template>
    <PageLayout flush>
        <div class="command-console">
            <div ref="scrollRef" class="console-results">
                <div class="results-inner">
                    <AnimatePresence mode="popLayout">
                        <MotionDiv :key="resultKind" class="console-view" :initial="viewInitial" :animate="viewAnimate"
                            :exit="viewExit" :transition="softTransition">
                            <div v-if="!result" class="console-idle">
                                <h1 class="idle-greeting">{{ greeting }}</h1>
                                <p class="idle-subtitle">{{ t('home.command.idleHint') }}</p>
                                <p v-if="!library.tracks.value.length" class="idle-empty">{{ t('player.importHint') }}
                                </p>
                                <div class="command-chips">
                                    <MotionButton v-for="(chip, index) in chips" :key="chip.name" type="button"
                                        class="command-chip" :initial="{ opacity: 0, y: 12 }"
                                        :animate="{ opacity: 1, y: 0 }" :transition="chipTransition(index)"
                                        :while-hover="{ y: -2 }" :while-press="{ scale: 0.97 }" @click="runChip(chip)">
                                        <span class="chip-usage">{{ chip.usage }}</span>
                                        <span class="chip-desc">{{ chip.description }}</span>
                                    </MotionButton>
                                </div>
                            </div>

                            <div v-else-if="result.kind === 'error'" class="console-error" role="alert">
                                <span class="error-mark">✕</span>{{ result.message }}
                            </div>

                            <div v-else-if="result.kind === 'hint'" class="console-hint">{{ result.message }}</div>

                            <div v-else-if="result.kind === 'help'" class="console-help">
                                <h2 class="help-title">{{ t('home.command.helpTitle') }}</h2>
                                <div v-for="command in helpCommands" :key="command.name" class="help-row">
                                    <code class="help-usage">{{ command.usage }}</code>
                                    <span class="help-desc">{{ command.description }}</span>
                                </div>
                            </div>

                            <div v-else-if="result.kind === 'songs'" class="console-result">
                                <div class="result-summary">
                                    <div class="result-meta">
                                        <span class="result-title">{{ result.title }}</span>
                                        <span class="result-count">{{ t('player.songCount', {
                                            count:
                                                result.tracks.length
                                        })
                                            }}</span>
                                    </div>
                                    <div class="result-actions">
                                        <span class="result-hint">
                                            <CornerDownLeft :size="12" />{{ result.autoPlay ?
                                                t('home.command.playHint') : t('home.command.saveHint') }}
                                        </span>
                                        <button type="button" class="result-action" :disabled="!result.tracks.length"
                                            @click="playAll">
                                            <Play :size="14" />{{ t('library.playAll') }}
                                        </button>
                                        <button type="button" class="result-action primary"
                                            :disabled="!result.tracks.length" @click="openSaveDialog">
                                            <ListPlus :size="14" />{{ t('home.command.saveAsPlaylist') }}
                                        </button>
                                    </div>
                                </div>
                                <SongList v-if="result.tracks.length" :songs="result.tracks" :show-header="false"
                                    @song-play="onSongPlay" />
                                <div v-else class="result-empty">{{ t('home.command.emptyResult') }}</div>
                            </div>
                        </MotionDiv>
                    </AnimatePresence>
                </div>
            </div>

            <div class="console-dock">
                <div class="dock-scrim" aria-hidden="true"></div>
                <div class="dock-inner">
                    <AnimatePresence>
                        <MotionDiv v-if="suggestions.length" key="suggestion-panel" class="suggestion-list"
                            role="listbox" :initial="{ opacity: 0, y: 10, scale: 0.985 }"
                            :animate="{ opacity: 1, y: 0, scale: 1 }" :exit="{ opacity: 0, y: 6, scale: 0.985 }"
                            :transition="softTransition">
                            <button v-for="(item, index) in suggestions" :key="`${item.value}-${index}`" type="button"
                                class="suggestion-item" :class="{ active: index === activeSuggestion }" role="option"
                                :aria-selected="index === activeSuggestion" @mouseenter="activeSuggestion = index"
                                @mousedown.prevent="completeSuggestion(item)">
                                <span class="suggestion-usage">{{ item.usage }}</span>
                                <span class="suggestion-desc">{{ item.description }}</span>
                            </button>
                        </MotionDiv>
                    </AnimatePresence>
                    <div class="input-row" :class="{ 'command-mode': isCommandMode }">
                        <Terminal :size="16" class="input-icon" />
                        <input ref="inputRef" v-model="input" class="console-input" type="text"
                            :placeholder="t('home.command.placeholder')" aria-autocomplete="list"
                            :aria-label="t('home.command.placeholder')" @keydown="onKeydown" />
                    </div>
                </div>
            </div>
        </div>

        <Dialog v-model="isSaveDialogOpen" :aria-labelledby="'console-save-playlist-title'">
            <form class="dialog-content" @submit.prevent="submitSavePlaylist">
                <header class="dialog-header">
                    <div>
                        <h2 id="console-save-playlist-title">{{ t('dialog.playlist.createTitle') }}</h2>
                    </div>
                </header>
                <p class="dialog-message">{{ saveDialogMessage }}</p>
                <input v-model="playlistName" class="dialog-input" type="text"
                    :placeholder="t('dialog.playlist.createPlaceholder')" :disabled="isSavingPlaylist" />
                <footer class="dialog-actions">
                    <button type="button" class="dialog-button secondary" :disabled="isSavingPlaylist"
                        @click="isSaveDialogOpen = false">
                        {{ t('dialog.actions.cancel') }}
                    </button>
                    <button type="submit" class="dialog-button primary"
                        :disabled="isSavingPlaylist || !playlistName.trim()">
                        {{ t('dialog.playlist.createConfirm') }}
                    </button>
                </footer>
            </form>
        </Dialog>
    </PageLayout>
</template>

<script setup>
import { computed, inject, nextTick, onActivated, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import SongList from '@/components/library/SongList.vue'
import Dialog from '@/components/ui/Dialog.vue'
import Tip from '@/components/notification/Tip.vue'
import { CornerDownLeft, ListPlus, Play, Terminal } from '@lucide/vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, SOFT_SPRING } from '@/utils/motion.js'
import { useLibraryStore } from '@/stores/libraryStore.js'
import { useI18n } from '@/i18n/index.js'
import { execute, listCommands, parseInput, suggest } from '@/services/commandEngine.js'

const emit = defineEmits(['song-play'])

const { t } = useI18n()
const library = useLibraryStore()
const notification = inject('notification', null)

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()

const inputRef = ref(null)
const scrollRef = ref(null)
const input = ref('')
const result = ref(null)
const suggestions = ref([])
const activeSuggestion = ref(-1)
const commandHistory = ref([])
const historyIndex = ref(-1)
const draftInput = ref('')
const isSaveDialogOpen = ref(false)
const playlistName = ref('')
const isSavingPlaylist = ref(false)

let queryTimer = null
let liveRequestId = 0

const greeting = computed(() => {
    const hour = new Date().getHours()
    if (hour < 12) return t('home.goodMorning')
    if (hour < 18) return t('home.goodAfternoon')
    return t('home.goodEvening')
})

const isCommandMode = computed(() => parseInput(input.value).type === 'command')

const resultKind = computed(() => (result.value ? result.value.kind : 'idle'))

const softTransition = computed(() => (reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING))
const viewInitial = computed(() => (
    reducedMotion.value ? { opacity: 0 } : { opacity: 0, y: 16, filter: 'blur(6px)' }
))
const viewAnimate = computed(() => (
    reducedMotion.value ? { opacity: 1 } : { opacity: 1, y: 0, filter: 'blur(0px)' }
))
const viewExit = computed(() => (
    reducedMotion.value ? { opacity: 0 } : { opacity: 0, y: -8, filter: 'blur(4px)' }
))
const chipTransition = (index) => (
    reducedMotion.value ? INSTANT_MOTION : { ...SOFT_SPRING, delay: 0.08 + index * 0.05 }
)

const helpCommands = computed(() => listCommands())

const chips = computed(() => {
    const wanted = ['random', 'recent', 'all', 'help']
    return wanted
        .map((name) => helpCommands.value.find((command) => command.name === name))
        .filter(Boolean)
})

const buildCtx = () => ({
    tracks: library.tracks.value,
    albums: library.albums.value,
    artists: library.artists.value,
    tags: library.tags.value,
    history: library.state.history,
    tracksByTag: (tagId) => library.tracksByTag(tagId)
})

const refreshSuggestions = () => {
    const parsed = parseInput(input.value)
    if (parsed.type !== 'command') {
        suggestions.value = []
        activeSuggestion.value = -1
        return
    }
    suggestions.value = suggest(input.value, buildCtx())
    activeSuggestion.value = suggestions.value.length ? 0 : -1
}

const scrollToResultsTop = () => {
    nextTick(() => {
        if (scrollRef.value) scrollRef.value.scrollTop = 0
    })
}

const runLive = async (text) => {
    const requestId = ++liveRequestId
    const outcome = await execute(text, buildCtx())
    if (requestId !== liveRequestId) return
    result.value = outcome?.kind === 'idle' ? null : outcome
    scrollToResultsTop()
}

const recordHistory = (raw) => {
    const text = String(raw || '').trim()
    if (!text || commandHistory.value[0] === text) return
    commandHistory.value.unshift(text)
    if (commandHistory.value.length > 50) commandHistory.value.pop()
    historyIndex.value = -1
}

const runChip = (chip) => {
    const text = `/${chip.name}`
    recordHistory(text)
    if (input.value === text) {
        void runLive(text)
        return
    }
    input.value = text
}

const completeSuggestion = (item) => {
    if (!item) return
    input.value = item.value
    inputRef.value?.focus?.()
}

const navigateHistory = (step) => {
    const history = commandHistory.value
    if (!history.length) return
    if (step > 0) {
        if (historyIndex.value === -1) {
            draftInput.value = input.value
            historyIndex.value = 0
        } else if (historyIndex.value < history.length - 1) {
            historyIndex.value += 1
        } else return
    } else {
        if (historyIndex.value === -1) return
        if (historyIndex.value === 0) {
            historyIndex.value = -1
            input.value = draftInput.value
            return
        }
        historyIndex.value -= 1
    }
    input.value = history[historyIndex.value] ?? ''
}

const onKeydown = (event) => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        event.preventDefault()
        if (suggestions.value.length) {
            const count = suggestions.value.length
            const delta = event.key === 'ArrowDown' ? 1 : -1
            activeSuggestion.value = (activeSuggestion.value + delta + count) % count
        } else {
            navigateHistory(event.key === 'ArrowUp' ? 1 : -1)
        }
        return
    }
    if (event.key === 'Tab') {
        if (suggestions.value.length) {
            event.preventDefault()
            completeSuggestion(suggestions.value[Math.max(0, activeSuggestion.value)])
        }
        return
    }
    if (event.key === 'Escape') {
        if (input.value) {
            input.value = ''
        } else if (result.value) {
            result.value = null
        }
        return
    }
    if (event.key === 'Enter') {
        event.preventDefault()
        const parsed = parseInput(input.value)
        if (parsed.type === 'command') recordHistory(input.value)
        const songs = result.value?.kind === 'songs' ? result.value : null
        if (!songs?.tracks.length) return
        if (songs.autoPlay) {
            playAll()
            return
        }
        openSaveDialog()
    }
}

watch(input, (value) => {
    refreshSuggestions()
    if (queryTimer) {
        clearTimeout(queryTimer)
        queryTimer = null
    }
    if (parseInput(value).type === 'empty') {
        liveRequestId += 1
        result.value = null
        return
    }
    queryTimer = setTimeout(() => {
        queryTimer = null
        void runLive(value)
    }, 110)
})

const playAll = () => {
    const tracks = result.value?.tracks || []
    if (!tracks.length) return
    emit('song-play', { song: tracks[0], songs: tracks })
}

const onSongPlay = (song) => {
    const tracks = result.value?.tracks || []
    emit('song-play', { song, songs: tracks })
}

const saveDialogMessage = computed(() => (
    t('home.command.saveMessage', { count: result.value?.tracks?.length || 0 })
))

const openSaveDialog = () => {
    const tracks = result.value?.tracks || []
    if (!tracks.length) return
    playlistName.value = result.value.title || ''
    isSaveDialogOpen.value = true
}

const submitSavePlaylist = async () => {
    const name = playlistName.value.trim()
    const tracks = result.value?.kind === 'songs' ? result.value.tracks : []
    if (!name || isSavingPlaylist.value || !tracks.length) return
    isSavingPlaylist.value = true
    try {
        const created = await library.createPlaylist(name)
        const playlistId = created?.id
        if (!playlistId) throw new Error('create playlist failed')
        for (const track of tracks) {
            await library.addToPlaylist(playlistId, track.id)
        }
        isSaveDialogOpen.value = false
        notification?.value?.addNotification?.(
            t('home.command.savedTitle'),
            name,
            Tip,
            null,
            { Tip: t('home.command.savedTip', { count: tracks.length, name }) },
            5000
        )
    } catch (error) {
        console.error('保存播放列表失败:', error)
        notification?.value?.addNotification?.(
            t('home.command.saveFailed'),
            name,
            Tip,
            null,
            { Tip: String(error?.message || error) },
            6000
        )
    } finally {
        isSavingPlaylist.value = false
    }
}

const focusInput = () => {
    inputRef.value?.focus?.()
}

onMounted(() => {
    focusInput()
})

onActivated(() => {
    focusInput()
})

onBeforeUnmount(() => {
    if (queryTimer) {
        clearTimeout(queryTimer)
        queryTimer = null
    }
})
</script>

<style scoped>
.command-console {
    --console-mono: ui-monospace, 'Cascadia Code', Consolas, 'Courier New', monospace;
    --dock-clearance: 148px;
    position: relative;
    width: 100%;
    height: calc(100vh - 64px);
}

/* 结果区（内部滚动；底部留白避开悬浮输入框） */
.console-results {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    overflow-x: hidden;
}

.console-results::-webkit-scrollbar {
    width: 4px;
}

.console-results::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
}

.console-results::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.console-results::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
}

.results-inner {
    width: 100%;
    padding: 20px clamp(48px, 8vw, 112px) var(--dock-clearance);
}

/* 空闲状态 */
.console-idle {
    padding-top: 8vh;
}

.idle-greeting {
    font-size: 34px;
    font-weight: 700;
    color: rgb(var(--text-color));
}

.idle-subtitle {
    margin-top: 10px;
    font-size: 14px;
    color: rgba(var(--text-color), 0.55);
}

.idle-empty {
    margin-top: 6px;
    font-size: 13px;
    color: rgba(var(--text-color), 0.4);
}

.command-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 28px;
}

.command-chip {
    display: inline-flex;
    align-items: baseline;
    gap: 9px;
    padding: 9px 14px;
    border: 1px solid rgba(var(--outline-color), 0.16);
    border-radius: 12px;
    background: color-mix(in srgb, rgba(var(--primary-color), 0.5) 12%, rgba(var(--global-color), 0.55) 88%);
    backdrop-filter: blur(10px);
    color: rgb(var(--text-color));
    font: inherit;
    cursor: pointer;
    transition: border-color 0.16s ease, background 0.16s ease;
}

.command-chip:hover {
    border-color: rgba(var(--primary-color), 0.45);
    background: color-mix(in srgb, rgba(var(--primary-color), 0.5) 18%, rgba(var(--global-color), 0.6) 82%);
}

.chip-usage {
    font-family: var(--console-mono);
    font-size: 13px;
    font-weight: 600;
}

.chip-desc {
    font-size: 12px;
    color: rgba(var(--text-color), 0.55);
}

/* 错误行（Minecraft 式红字） */
.console-error {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    padding: 12px 16px;
    border: 1px solid rgba(244, 67, 54, 0.18);
    border-radius: 12px;
    background: rgba(244, 67, 54, 0.08);
    color: #ef5350;
    font-size: 13.5px;
    font-weight: 500;
}

.error-mark {
    font-weight: 700;
}

/* 用法提示（参数不全时的中性提示条） */
.console-hint {
    display: inline-flex;
    align-items: center;
    padding: 10px 16px;
    border: 1px solid rgba(var(--outline-color), 0.14);
    border-radius: 12px;
    background: rgba(var(--global-color), 0.4);
    color: rgba(var(--text-color), 0.6);
    font-size: 13px;
}

/* /help 结果 */
.console-help {
    max-width: 680px;
}

.help-title {
    font-size: 16px;
    font-weight: 700;
    color: rgb(var(--text-color));
    margin-bottom: 12px;
}

.help-row {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 14px;
    align-items: center;
    padding: 7px 12px;
    border-radius: 10px;
}

.help-row:hover {
    background: rgba(var(--surface-color), 0.5);
}

.help-usage {
    justify-self: start;
    padding: 4px 10px;
    border: 1px solid rgba(var(--primary-color), 0.2);
    border-radius: 8px;
    background: rgba(var(--primary-color), 0.12);
    font-family: var(--console-mono);
    font-size: 12.5px;
    font-weight: 600;
    color: rgb(var(--text-color));
}

.help-desc {
    font-size: 13px;
    color: rgba(var(--text-color), 0.6);
}

/* 歌曲结果 */
.result-summary {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: 14px 18px;
    margin-bottom: 14px;
}

.result-title {
    font-size: 20px;
    font-weight: 700;
    color: rgb(var(--text-color));
}

.result-count {
    margin-left: 10px;
    font-size: 13px;
    font-weight: 500;
    color: rgba(var(--text-color), 0.5);
}

.result-actions {
    display: flex;
    align-items: center;
    gap: 10px;
}

.result-hint {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    margin-right: 4px;
    font-size: 12px;
    color: rgba(var(--text-color), 0.45);
}

.result-action {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    min-height: 36px;
    padding: 0 14px;
    border: 1px solid rgba(var(--outline-color), 0.16);
    border-radius: 11px;
    background: color-mix(in srgb, rgba(var(--primary-color), 0.5) 12%, rgba(var(--global-color), 0.6) 88%);
    backdrop-filter: blur(8px);
    color: rgb(var(--text-color));
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.16s ease, border-color 0.16s ease;
}

.result-action:hover:not(:disabled) {
    transform: translateY(-1px);
    border-color: rgba(var(--primary-color), 0.5);
}

.result-action.primary {
    background: rgba(var(--primary-color), 0.22);
    border-color: rgba(var(--primary-color), 0.3);
}

.result-action:disabled {
    opacity: 0.45;
    cursor: not-allowed;
}

.result-empty {
    padding: 48px 0;
    text-align: center;
    font-size: 14px;
    color: rgba(var(--text-color), 0.5);
}

/* 底部输入区（悬浮 + 毛玻璃） */
.console-dock {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 6;
    padding: 0 clamp(48px, 8vw, 112px) 22px;
    pointer-events: none;
}


.dock-inner {
    position: relative;
    width: 100%;
    pointer-events: auto;
}

.suggestion-list {
    position: absolute;
    left: 0;
    right: 0;
    bottom: calc(100% + 10px);
    z-index: 5;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid rgba(var(--outline-color), 0.16);
    border-radius: 13px;
    background: color-mix(in srgb, rgba(var(--primary-color), 0.5) 18%, rgba(var(--global-color), 0.78) 82%);
    backdrop-filter: blur(22px);
    box-shadow: 0 18px 44px rgba(0, 0, 0, 0.18);
}

.suggestion-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 9px 14px;
    border: 0;
    background: transparent;
    color: rgb(var(--text-color));
    font: inherit;
    text-align: left;
    cursor: pointer;
}

.suggestion-item.active {
    background: rgba(var(--primary-color), 0.16);
}

.suggestion-usage {
    flex: 0 0 auto;
    font-family: var(--console-mono);
    font-size: 13px;
    font-weight: 600;
}

.suggestion-desc {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: rgba(var(--text-color), 0.55);
}

.input-row {
    display: flex;
    align-items: center;
    gap: 11px;
    min-height: 52px;
    padding: 0 16px;
    border: 1px solid rgba(var(--outline-color), 0.18);
    border-radius: 15px;
    background: color-mix(in srgb, rgba(var(--primary-color), 0.5) 20%, rgba(var(--global-color), 0.72) 80%);
    backdrop-filter: blur(22px);
    transition: border-color 0.18s ease, box-shadow 0.18s ease;
}

.input-row:focus-within {
    border-color: rgba(var(--primary-color), 0.6);
    box-shadow: 0 0 0 4px rgba(var(--primary-color), 0.12);
}

.input-icon {
    flex: 0 0 auto;
    color: rgba(var(--text-color), 0.45);
    transition: color 0.18s ease;
}

.input-row.command-mode .input-icon {
    color: rgba(var(--primary-color), 0.95);
}

.console-input {
    flex: 1 1 auto;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font-size: 15px;
    color: rgb(var(--text-color));
}

.console-input::placeholder {
    color: rgba(var(--text-color), 0.35);
}

/* 响应式 */
@media (max-width: 768px) {
    .results-inner {
        padding: 16px 24px 124px;
    }

    .console-dock {
        padding: 0 24px 18px;
    }

    .console-idle {
        padding-top: 4vh;
    }

    .idle-greeting {
        font-size: 26px;
    }

    .help-row {
        grid-template-columns: 1fr;
        gap: 6px;
    }
}

@media (prefers-reduced-motion: reduce) {

    .result-action {
        transition: none;
    }

    .result-action:hover:not(:disabled) {
        transform: none;
    }
}
</style>
