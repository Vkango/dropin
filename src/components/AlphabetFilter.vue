<template>
    <div class="alphabet-filter" data-alphabet-filter :aria-label="t('generic.alphabetNavigation')"
        :style="{ marginTop: `${topOffset}px` }">
        <button v-for="option in alphabetOptions" :key="option.value" type="button" class="alphabet-btn"
            :class="{ active: activeInitial === option.value, disabled: isDisabled(option.value) }"
            :disabled="isDisabled(option.value)" :aria-pressed="activeInitial === option.value"
            :aria-label="t('generic.jumpTo', { letter: option.label })" @click="select(option.value)">
            {{ option.label }}
        </button>
    </div>
</template>

<script setup>
import { computed } from 'vue'
import { alphabetOptions, ALL_INITIAL } from '../utils/alphabet.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const props = defineProps({
    activeInitial: {
        type: String,
        default: ALL_INITIAL
    },
    topOffset: {
        type: Number,
        default: 0
    },
    availableInitials: {
        type: Array,
        default: () => []
    }
})

const emit = defineEmits(['select'])
const availableSet = computed(() => new Set(props.availableInitials))

const isDisabled = (initial) => initial !== ALL_INITIAL && !availableSet.value.has(initial)

const select = (initial) => {
    if (!isDisabled(initial)) emit('select', initial)
}
</script>

<style scoped>
.alphabet-filter {
    display: flex;
    align-items: center;
    flex-direction: column;
    gap: 2px;
    position: sticky;
    top: 72px;
    align-self: flex-start;
    flex: 0 0 26px;
    max-height: calc(100vh - 96px);
    padding: 6px 3px;
    overflow-y: auto;
    opacity: 0.58;
    scrollbar-width: none;
    transition: opacity 160ms ease;
}

.alphabet-filter::-webkit-scrollbar {
    display: none;
}

.alphabet-filter:hover,
.alphabet-filter:focus-within {
    opacity: 0.92;
}

.alphabet-btn {
    width: 26px;
    min-width: 26px;
    height: 24px;
    padding: 0;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: rgba(var(--text-color), 0.72);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 160ms ease, color 160ms ease, opacity 160ms ease;
}

.alphabet-btn:hover:not(:disabled) {
    background: rgba(var(--surface-color), 0.32);
    color: rgb(var(--primary-color));
}

.alphabet-btn.active {
    background: rgba(var(--primary-color), 0.24);
    color: rgb(var(--primary-color));
}

.alphabet-btn:disabled,
.alphabet-btn.disabled {
    opacity: 0.28;
    cursor: not-allowed;
}

.alphabet-btn:focus-visible {
    outline: 2px solid rgba(var(--primary-color), 0.6);
    outline-offset: 2px;
}
</style>
