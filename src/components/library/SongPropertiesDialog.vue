<template>
    <Dialog :model-value="modelValue" :width="380" :close-on-backdrop="true" :aria-label="t('songProperties.title')"
        @update:model-value="$emit('update:modelValue', $event)" @close="$emit('update:modelValue', false)">
        <div v-if="song" class="song-properties">
            <header class="properties-hero">
                <img class="properties-cover" :src="song.cover" :alt="song.title" referrerpolicy="no-referrer">
                <div class="properties-heading">
                    <h2 class="properties-title">{{ song.title }}</h2>
                    <p class="properties-artist">{{ song.artist }}</p>
                </div>
            </header>
            <dl class="properties-grid">
                <div class="properties-row">
                    <dt>{{ t('songProperties.album') }}</dt>
                    <dd>{{ song.album || '–' }}</dd>
                </div>
                <div class="properties-row">
                    <dt>{{ t('songProperties.duration') }}</dt>
                    <dd>{{ durationLabel(song) }}</dd>
                </div>
                <div class="properties-row">
                    <dt>{{ t('songProperties.tags') }}</dt>
                    <dd class="properties-tags">
                        <template v-if="tags.length">
                            <span v-for="tag in tags" :key="tag.tagId" class="tag-chip"
                                :title="formatTagBadge(tag.label, tag.value)">
                                {{ formatTagBadge(tag.label, tag.value) }}
                            </span>
                        </template>
                        <span v-else class="tag-empty">–</span>
                    </dd>
                </div>
                <div class="properties-row">
                    <dt>{{ t('songProperties.path') }}</dt>
                    <dd class="properties-path" :title="song.sourcePath || ''">
                        <span class="path-text">{{ song.sourcePath || '–' }}</span>
                        <button v-if="song.sourcePath" type="button" class="path-copy" @click="copyPath">
                            <Check v-if="copied" :size="12" :stroke-width="2" />
                            <Copy v-else :size="12" :stroke-width="2" />
                            {{ copied ? t('songProperties.copied') : t('songProperties.copyPath') }}
                        </button>
                    </dd>
                </div>
            </dl>
            <footer class="properties-actions">
                <button type="button" class="dialog-button secondary" @click="$emit('update:modelValue', false)">
                    {{ t('songProperties.close') }}
                </button>
            </footer>
        </div>
    </Dialog>
</template>

<script setup>
import { ref, watch } from 'vue'
import { Check, Copy } from '@lucide/vue'
import Dialog from '@/components/ui/Dialog.vue'
import { useLibraryStore } from '@/stores/libraryStore.js'
import { formatTagBadge } from '@/utils/tagProvider.js'
import { useI18n } from '@/i18n/index.js'

const props = defineProps({
    modelValue: {
        type: Boolean,
        default: false
    },
    song: {
        type: Object,
        default: null
    }
})

defineEmits(['update:modelValue'])

const { t } = useI18n()
const libraryStore = useLibraryStore()
const copied = ref(false)
const tags = ref([])
let tagsRequestId = 0
let copiedTimer = null

// 优先用现成的时长文本，缺失时从 durationMs 现算
const durationLabel = (song) => {
    if (song?.duration) return song.duration
    const ms = Number(song?.durationMs)
    if (!Number.isFinite(ms) || ms <= 0) return '–'
    const totalSeconds = Math.max(0, Math.floor(ms / 1000))
    const minutes = Math.floor(totalSeconds / 60)
    const seconds = totalSeconds % 60
    return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
}

const loadTags = async () => {
    const trackId = props.song?.id
    if (!props.modelValue || !trackId) {
        tags.value = []
        return
    }
    const request = ++tagsRequestId
    try {
        const result = await libraryStore.trackTags(trackId)
        if (request === tagsRequestId) tags.value = result
    } catch {
        if (request === tagsRequestId) tags.value = []
    }
}

watch(
    () => [props.modelValue, props.song?.id],
    (value, previous) => {
        if (!props.modelValue) {
            tags.value = []
            return
        }
        const [open, trackId] = value
        const [previousOpen, previousTrackId] = previous || []
        if (open && (previousOpen !== open || trackId !== previousTrackId)) void loadTags()
    },
    { immediate: true }
)

const copyPath = async () => {
    if (!props.song?.sourcePath) return
    try {
        await navigator.clipboard.writeText(props.song.sourcePath)
        copied.value = true
        if (copiedTimer) clearTimeout(copiedTimer)
        copiedTimer = setTimeout(() => {
            copied.value = false
        }, 2000)
    } catch {
        copied.value = false
    }
}

watch(() => props.modelValue, (open) => {
    if (!open) copied.value = false
})
</script>

<style scoped>
.song-properties {
    display: grid;
    gap: 18px;
    padding: 22px;
}

.properties-hero {
    display: flex;
    align-items: center;
    gap: 16px;
    /* 网格项目的 min-width 默认为 auto，长标题（nowrap）会把对话框撑溢出 */
    min-width: 0;
}

.properties-cover {
    flex: 0 0 72px;
    width: 72px;
    height: 72px;
    border-radius: 10px;
    object-fit: cover;
    box-shadow: 0 8px 22px rgba(0, 0, 0, 0.18);
}

.properties-heading {
    flex: 1 1 auto;
    min-width: 0;
}

.properties-title {
    margin: 0 0 4px 0;
    font-size: 16px;
    font-weight: 700;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.properties-artist {
    margin: 0;
    color: rgba(var(--text-color), 0.6);
    font-size: 12.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.properties-grid {
    margin: 0;
    display: grid;
    gap: 10px;
}

.properties-row {
    display: grid;
    grid-template-columns: 64px 1fr;
    gap: 12px;
    align-items: start;
}

.properties-row dt {
    color: rgba(var(--text-color), 0.45);
    font-size: 12px;
    line-height: 1.7;
}

.properties-row dd {
    margin: 0;
    color: rgb(var(--text-color));
    font-size: 12.5px;
    line-height: 1.7;
    min-width: 0;
}

.properties-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}

.tag-chip {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 2px 8px;
    border-radius: 6px;
    background: rgba(var(--primary-color), 0.16);
    color: rgb(var(--text-color));
    font-size: 11.5px;
    line-height: 1.6;
}

.tag-empty {
    color: rgba(var(--text-color), 0.4);
}

.properties-path {
    display: flex;
    align-items: center;
    gap: 8px;
}

.path-text {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
}

.path-copy {
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border: 1px solid rgba(var(--outline-color), 0.18);
    border-radius: 8px;
    background: rgba(var(--global-inverse-color), 0.05);
    color: rgba(var(--text-color), 0.7);
    font-size: 11px;
    cursor: pointer;
}

.path-copy:hover {
    background: rgba(var(--global-inverse-color), 0.09);
}

.properties-actions {
    display: flex;
    justify-content: flex-end;
}
</style>
