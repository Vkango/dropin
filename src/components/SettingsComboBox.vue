<template>
    <div ref="rootRef" class="settings-combobox" :class="{ 'is-open': open }">
        <button ref="triggerRef" type="button" class="combobox-trigger" :aria-haspopup="listbox"
            :aria-expanded="open" :aria-label="triggerLabel ?? label" @click.stop="handleTriggerClick"
            @keydown.stop="handleTriggerKeydown">
            <span class="combobox-trigger-label">{{ triggerLabel ?? label }}</span>
            <ChevronDown class="combobox-chevron" :size="14" :stroke-width="1.8" aria-hidden="true" />
        </button>
        <Transition name="combobox-menu">
            <div v-if="open" class="combobox-menu" :style="menuStyle" role="listbox" :aria-label="label">
                <div v-if="heading" class="combobox-heading">{{ heading }}</div>
                <button v-for="option in options" :key="option.value" type="button" role="option"
                    class="combobox-option" :class="{ 'is-selected': option.value === modelValue }"
                    :aria-selected="option.value === modelValue" @click.stop="select(option.value)">
                    <component :is="option.icon" v-if="option.icon" class="combobox-option-icon" :size="16"
                        :stroke-width="1.7" aria-hidden="true" />
                    <span class="combobox-option-text">{{ option.label }}</span>
                    <span class="combobox-option-value" v-if="option.valueText">{{ option.valueText }}</span>
                    <span class="combobox-option-check" aria-hidden="true">✓</span>
                </button>
            </div>
        </Transition>
    </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ChevronDown } from '@lucide/vue'

const props = defineProps({
    modelValue: {
        type: [String, Number],
        default: null
    },
    options: {
        type: Array,
        required: true
    },
    label: {
        type: String,
        default: ''
    },
    triggerLabel: {
        type: String,
        default: null
    },
    heading: {
        type: String,
        default: ''
    },
    placement: {
        type: String,
        default: 'below'
    },
    width: {
        type: Number,
        default: 220
    },
    open: Boolean
})

const emit = defineEmits(['update:modelValue', 'update:open', 'select'])
const rootRef = ref(null)
const triggerRef = ref(null)
const menuStyle = ref({})

const TRIGGER_BORDER = 1
const TRIGGER_PADDING = 10

const updatePosition = () => {
    const trigger = triggerRef.value
    if (!trigger) return
    const rect = trigger.getBoundingClientRect()
    const margin = 8
    // 列表左缘与选项框内文字左缘对齐（跳过边框与内边距），并限制在视口内
    const contentLeft = rect.left + TRIGGER_BORDER + TRIGGER_PADDING
    const left = Math.max(margin, Math.min(window.innerWidth - props.width - margin, contentLeft))
    const style = {
        width: `${props.width}px`,
        left: `${left}px`,
        right: 'auto',
        top: 'auto',
        bottom: 'auto'
    }
    if (props.placement === 'above') {
        style.bottom = `${window.innerHeight - rect.top + 8}px`
    } else {
        style.top = `${rect.bottom + 8}px`
    }
    menuStyle.value = style
}

const handleViewportChange = () => {
    if (props.open) updatePosition()
}

const handleOutsidePointerDown = (event) => {
    if (!props.open) return
    const root = rootRef.value
    if (root && !root.contains(event.target)) emit('update:open', false)
}

const handleKeydown = (event) => {
    if (event.key === 'Escape' && props.open) {
        emit('update:open', false)
        event.stopPropagation()
    }
}

const handleTriggerClick = () => {
    const nextOpen = !props.open
    if (nextOpen) updatePosition()
    emit('update:open', nextOpen)
}

const handleTriggerKeydown = (event) => {
    if (event.key === 'ArrowDown' || event.key === 'Enter' || event.key === ' ') {
        event.preventDefault()
        updatePosition()
        emit('update:open', true)
    }
}

const select = (value) => {
    emit('select', value)
    emit('update:modelValue', value)
    emit('update:open', false)
}

watch(
    () => props.open,
    (open) => {
        if (open) updatePosition()
    }
)

onMounted(() => {
    document.addEventListener('pointerdown', handleOutsidePointerDown, true)
    document.addEventListener('keydown', handleKeydown, true)
    window.addEventListener('resize', handleViewportChange)
    if (props.open) updatePosition()
})

onBeforeUnmount(() => {
    document.removeEventListener('pointerdown', handleOutsidePointerDown, true)
    document.removeEventListener('keydown', handleKeydown, true)
    window.removeEventListener('resize', handleViewportChange)
})
</script>

<style scoped>
.settings-combobox {
    position: relative;
    z-index: 10;
    display: inline-flex;
}

.combobox-trigger {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border: 1px solid rgba(var(--outline-color, 255, 255, 255), 0.16);
    border-radius: 9px;
    color: rgba(var(--text-color, 255, 255, 255), 0.78);
    background: rgba(var(--surface-color, 255, 255, 255), 0.06);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
}

.combobox-trigger:hover,
.settings-combobox.is-open .combobox-trigger {
    color: rgb(var(--text-color, 255, 255, 255));
    border-color: rgba(var(--primary-color), 0.72);
    background: rgba(var(--primary-color), 0.2);
}

.combobox-trigger-label {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.combobox-chevron {
    flex: 0 0 auto;
    transition: transform 160ms ease;
}

.settings-combobox.is-open .combobox-chevron {
    transform: rotate(180deg);
}

.combobox-menu {
    position: fixed;
    z-index: 2147483647;
    max-height: 70dvh;
    overflow-y: auto;
    padding: 10px;
    color: rgb(var(--text-color, 255, 255, 255));
    background: rgba(18, 16, 14, 0.86);
    border: 1px solid rgba(var(--outline-color, 255, 255, 255), 0.16);
    border-radius: 14px;
    box-shadow: 0 18px 46px rgba(0, 0, 0, 0.4), 0 2px 8px rgba(0, 0, 0, 0.24);
    backdrop-filter: blur(24px) saturate(1.3);
    -webkit-backdrop-filter: blur(24px) saturate(1.3);
}

.combobox-heading {
    padding: 2px 8px 8px;
    color: rgba(var(--text-color, 255, 255, 255), 0.52);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
}

.combobox-option {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    min-height: 38px;
    padding: 7px 8px;
    border: 0;
    border-radius: 8px;
    color: rgba(var(--text-color, 255, 255, 255), 0.78);
    background: transparent;
    font: inherit;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
}

.combobox-option:hover {
    background: rgba(var(--primary-color), 0.14);
    color: rgb(var(--text-color, 255, 255, 255));
}

.combobox-option.is-selected {
    color: rgb(var(--text-color, 255, 255, 255));
    background: rgba(var(--primary-color), 0.18);
}

.combobox-option-icon {
    flex: 0 0 auto;
}

.combobox-option-text {
    flex: 1 1 auto;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.combobox-option-value {
    flex: 0 0 auto;
    color: rgba(var(--text-color, 255, 255, 255), 0.48);
    font-variant-numeric: tabular-nums;
}

.combobox-option-check {
    flex: 0 0 auto;
    margin-left: auto;
    color: rgb(var(--primary-color));
    font-size: 14px;
    font-weight: 800;
    visibility: hidden;
}

.combobox-option.is-selected .combobox-option-check {
    visibility: visible;
}

.combobox-menu-enter-active,
.combobox-menu-leave-active {
    transition: opacity 160ms ease, transform 160ms ease;
}

.combobox-menu-enter-from,
.combobox-menu-leave-to {
    opacity: 0;
    transform: translateY(6px) scale(0.98);
}
</style>
