<template>
    <PopoverCard :open="open" :anchor-id="anchorId" :anchor="anchor" :placement="placement" :gap="gap" :width="238"
        @close="$emit('close')">
        <div v-if="includeVolume" class="mini-volume-row">
            <button class="mute-button" type="button" :aria-label="t('player.mute')" :aria-pressed="muted"
                @click="$emit('mute-change', !muted)">
                <VolumeX v-if="muted" :size="17" :stroke-width="1.8" />
                <Volume1 v-else-if="volume < 50" :size="17" :stroke-width="1.8" />
                <Volume2 v-else :size="17" :stroke-width="1.8" />
            </button>
            <RangeSlider :model-value="muted ? 0 : volume" :min="0" :max="100" :aria-label="t('player.volume')"
                :aria-value-text="`${Math.round(volume)}%`" @update:model-value="$emit('update:volume', $event)" />
            <output>{{ Math.round(volume) }}%</output>
        </div>
        <div v-if="includeVolume" class="popover-divider"></div>
        <div class="popover-heading">{{ t('player.playbackOrder') }}</div>
        <div class="mode-options" role="radiogroup" :aria-label="t('player.playbackMode')">
            <MotionButton v-for="option in modeOptions" :key="option.value" class="mode-option" role="radio"
                :aria-checked="mode === option.value" :while-hover="buttonHover" :while-press="buttonPress"
                :transition="microTransition" @click="selectMode(option.value)">
                <component :is="option.icon" :size="17" :stroke-width="1.8" />
                <span class="mode-copy">
                    <strong>{{ option.label }}</strong>
                </span>
                <span v-if="mode === option.value" class="mode-check" aria-hidden="true">✓</span>
            </MotionButton>
        </div>
        <label class="list-loop-toggle">
            <span class="checkbox-wrap">
                <input type="checkbox" :checked="listLoop" @change="$emit('update:list-loop', $event.target.checked)" />
                <span class="checkbox-mark" aria-hidden="true"></span>
            </span>
            <span>
                <strong>{{ t('player.listLoop') }}</strong>
                <small>{{ t('player.listLoopHint') }}</small>
            </span>
        </label>
    </PopoverCard>
</template>

<script setup>
import { computed } from 'vue'
import { ListOrdered, Repeat1, Shuffle, Volume1, Volume2, VolumeX } from '@lucide/vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'
import RangeSlider from './RangeSlider.vue'
import PopoverCard from './PopoverCard.vue'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const props = defineProps({
    open: Boolean,
    mode: {
        type: String,
        default: 'sequential'
    },
    listLoop: Boolean,
    includeVolume: Boolean,
    volume: {
        type: Number,
        default: 75
    },
    muted: Boolean,
    anchorId: {
        type: String,
        default: ''
    },
    anchor: {
        type: Object,
        default: null
    },
    placement: {
        type: String,
        default: 'above'
    },
    gap: {
        type: Number,
        default: 13
    }
})

const emit = defineEmits(['update:mode', 'update:list-loop', 'update:volume', 'mute-change', 'close'])
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const buttonHover = { backgroundColor: 'rgba(var(--primary-color), 0.12)' }
const buttonPress = { scale: 0.97 }

const modeOptions = computed(() => [
    { value: 'sequential', label: t('player.sequential'), icon: ListOrdered },
    { value: 'shuffle', label: t('player.shuffle'), icon: Shuffle },
    { value: 'repeat-one', label: t('player.repeatOne'), icon: Repeat1 }
])

const selectMode = (value) => {
    emit('update:mode', value)
}
</script>

<style scoped>
.mini-volume-row {
    display: flex;
    align-items: center;
    gap: 9px;
    color: rgba(var(--text-color), 0.52);
}

.mute-button {
    display: inline-flex;
    flex: 0 0 20px;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    color: inherit;
    background: transparent;
    cursor: pointer;
}

.mini-volume-row output {
    min-width: 38px;
    color: rgba(var(--text-color), 0.7);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-align: right;
}

.popover-divider {
    height: 1px;
    margin: 10px 0 4px;
    background: rgba(var(--outline-color), 0.12);
}

.popover-heading {
    padding: 2px 8px 9px;
    color: rgba(var(--text-color), 0.52);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
}

.mode-options {
    display: grid;
    gap: 3px;
}

.mode-option,
.list-loop-toggle {
    display: flex;
    align-items: center;
    width: 100%;
    min-height: 42px;
    gap: 10px;
    padding: 8px;
    color: rgba(var(--text-color), 0.78);
    background: transparent;
    border: 0;
    text-align: left;
}

.mode-option {
    cursor: pointer;
}

.mode-copy,
.list-loop-toggle>span:last-child {
    display: grid;
    min-width: 0;
    gap: 2px;
}

.mode-copy strong,
.list-loop-toggle strong {
    font-size: 12px;
    font-weight: 650;
}

.mode-copy small,
.list-loop-toggle small {
    color: rgba(var(--text-color), 0.48);
    font-size: 10px;
}

.mode-check {
    margin-left: auto;
    color: rgb(var(--primary-color));
    font-size: 14px;
    font-weight: 800;
}

.list-loop-toggle {
    margin-top: 7px;
    padding-top: 11px;
    border-top: 1px solid rgba(var(--outline-color), 0.12);
    cursor: pointer;
}

.checkbox-wrap {
    position: relative;
    flex: 0 0 17px;
    width: 17px;
    height: 17px;
}

.checkbox-wrap input {
    position: absolute;
    inset: 0;
    z-index: 1;
    width: 100%;
    height: 100%;
    margin: 0;
    opacity: 0;
    cursor: pointer;
}

.checkbox-mark {
    display: block;
    width: 17px;
    height: 17px;
    border: 1px solid rgba(var(--text-color), 0.34);
    border-radius: 5px;
    background: rgba(var(--text-color), 0.04);
}

.checkbox-wrap input:checked+.checkbox-mark {
    border-color: rgb(var(--primary-color));
    background: rgb(var(--primary-color));
    box-shadow: inset 0 0 0 3px rgba(var(--surface-color), 0.92);
}
</style>
