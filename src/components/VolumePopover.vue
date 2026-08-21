<template>
    <PopoverCard :open="open" :anchor-id="anchorId" :anchor="anchor" :placement="placement" :gap="gap"
        :width="218" @close="$emit('close')">
            <div class="volume-slider-row">
                <button class="mute-button" type="button" aria-label="切换静音" :aria-pressed="muted"
                    @click="$emit('mute-change', !muted)">
                    <VolumeX v-if="muted" :size="17" :stroke-width="1.8" />
                    <Volume1 v-else-if="volume < 50" :size="17" :stroke-width="1.8" />
                    <Volume2 v-else :size="17" :stroke-width="1.8" />
                </button>
                <RangeSlider :model-value="muted ? 0 : volume" :min="0" :max="100" aria-label="音量"
                    :aria-value-text="`${Math.round(volume)}%`"
                    @update:model-value="$emit('update:volume', $event)" />
                <output>{{ Math.round(volume) }}%</output>
            </div>
    </PopoverCard>
</template>

<script setup>
import { Volume1, Volume2, VolumeX } from '@lucide/vue'
import RangeSlider from './RangeSlider.vue'
import PopoverCard from './PopoverCard.vue'

const props = defineProps({
    open: Boolean,
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

const emit = defineEmits(['update:volume', 'mute-change', 'close'])
</script>

<style scoped>
.volume-popover {
    position: fixed;
    z-index: 1200;
    width: 218px;
    padding: 14px 15px 13px;
    color: rgb(var(--text-color));
    background: color-mix(in srgb, rgb(var(--surface-color)) 62%, transparent);
    border: 1px solid rgba(var(--outline-color), 0.16);
    border-radius: 16px;
    box-shadow: 0 18px 46px rgba(0, 0, 0, 0.24), 0 2px 8px rgba(0, 0, 0, 0.12);
    backdrop-filter: blur(28px) saturate(1.3);
    transform-origin: bottom center;
    will-change: transform, opacity, filter;
}

.volume-popover.below {
    transform-origin: top center;
}

.volume-slider-row {
    display: flex;
    align-items: center;
}

.volume-slider-row {
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

.volume-slider-row output {
    min-width: 38px;
    color: rgba(var(--text-color), 0.7);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-align: right;
}
</style>
