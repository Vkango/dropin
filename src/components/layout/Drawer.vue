<template>
    <Teleport to="body">
        <AnimatePresence>
            <div v-if="open" class="drawer-layer" :class="`drawer-layer-${resolvedPlacement}`" role="presentation"
                @click.self="requestClose">
                <MotionDiv class="drawer-backdrop" :initial="{ opacity: 0 }" :animate="{ opacity: 1 }"
                    :exit="{ opacity: 0 }" :transition="backdropTransition" aria-hidden="true" @click="requestClose" />

                <MotionDiv ref="panelRef" class="drawer-panel" :class="`drawer-panel-${resolvedPlacement}`"
                    :initial="panelMotion.initial" :animate="panelMotion.animate" :exit="panelMotion.exit"
                    :transition="panelTransition" role="dialog" aria-modal="true" :aria-labelledby="titleId"
                    tabindex="-1" @click.stop>
                    <header class="drawer-header">
                        <MotionButton ref="closeButtonRef" class="drawer-close-button" type="button"
                            :while-hover="buttonHover" :while-press="buttonPress" :transition="microTransition"
                            :aria-label="closeLabel || t('drawer.close')" @click="requestClose">
                            <ArrowLeft :size="20" :stroke-width="1.8" />
                        </MotionButton>
                        <h2 :id="titleId" class="drawer-title">{{ title || t('drawer.defaultTitle') }}</h2>
                        <div class="drawer-header-actions">
                            <slot name="header-actions" />
                        </div>
                    </header>

                    <div class="drawer-content">
                        <slot />
                    </div>
                </MotionDiv>
            </div>
        </AnimatePresence>
    </Teleport>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { ArrowLeft } from '@lucide/vue'
import { APPLE_SPRING, INSTANT_MOTION, MICRO_SPRING, SOFT_SPRING } from '@/utils/motion.js'
import { useI18n } from '@/i18n/index.js'

const { t } = useI18n()

const props = defineProps({
    open: {
        type: Boolean,
        default: false
    },
    title: {
        type: String,
        default: ''
    },
    placement: {
        type: String,
        default: 'right',
        validator: (value) => ['right', 'bottom'].includes(value)
    },
    closeLabel: {
        type: String,
        default: ''
    }
})

const emit = defineEmits(['close'])

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()
const closeButtonRef = ref(null)
const panelRef = ref(null)
const titleId = `drawer-title-${Math.random().toString(36).slice(2)}`
const isPageLocked = ref(false)
const previousBodyOverflow = ref('')
const previousFocusedElement = ref(null)
const isCompactViewport = ref(typeof window !== 'undefined'
    && window.matchMedia('(max-width: 720px)').matches)

const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const panelTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const backdropTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)
const buttonHover = { scale: 1.08, color: 'rgb(var(--primary-color))' }
const buttonPress = { scale: 0.92 }
const resolvedPlacement = computed(() => props.placement === 'right' && isCompactViewport.value ? 'bottom' : props.placement)

const panelMotion = computed(() => resolvedPlacement.value === 'bottom'
    ? {
        initial: { opacity: 0, y: '100%' },
        animate: { opacity: 1, y: 0 },
        exit: { opacity: 0, y: '100%' }
    }
    : {
        initial: { opacity: 0, x: '100%' },
        animate: { opacity: 1, x: 0 },
        exit: { opacity: 0, x: '100%' }
    })

const requestClose = () => emit('close')

const handleKeydown = (event) => {
    if (event.key === 'Escape') {
        event.preventDefault()
        requestClose()
    }
}

const focusCloseButton = () => {
    const element = closeButtonRef.value?.$el ?? closeButtonRef.value
    element?.focus?.()
}

const lockPage = () => {
    if (isPageLocked.value || typeof document === 'undefined') return
    isPageLocked.value = true
    previousFocusedElement.value = document.activeElement instanceof HTMLElement ? document.activeElement : null
    previousBodyOverflow.value = document.body.style.overflow
    document.body.style.overflow = 'hidden'
    window.addEventListener('keydown', handleKeydown)
    nextTick(focusCloseButton)
}

const unlockPage = () => {
    if (!isPageLocked.value || typeof document === 'undefined') return
    isPageLocked.value = false
    window.removeEventListener('keydown', handleKeydown)
    document.body.style.overflow = previousBodyOverflow.value
    const element = previousFocusedElement.value
    if (element?.isConnected) nextTick(() => element.focus?.())
    previousFocusedElement.value = null
}

const updateViewport = () => {
    isCompactViewport.value = typeof window !== 'undefined'
        && window.matchMedia('(max-width: 720px)').matches
}

watch(() => props.open, (open) => {
    if (open) lockPage()
    else unlockPage()
})

onMounted(() => {
    updateViewport()
    window.addEventListener('resize', updateViewport)
    if (props.open) lockPage()
})

onUnmounted(() => window.removeEventListener('resize', updateViewport))
onBeforeUnmount(unlockPage)
</script>

<style scoped>
.drawer-layer {
    position: fixed;
    inset: 0;
    z-index: 1200;
    display: flex;
    pointer-events: auto;
}

.drawer-layer-right {
    justify-content: flex-end;
}

.drawer-layer-bottom {
    align-items: flex-end;
    justify-content: center;
}

.drawer-backdrop {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.58);
    cursor: pointer;
}

.drawer-panel {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: rgb(var(--text-color));
    background: color-mix(in srgb, rgb(var(--surface-color)) 96%, rgb(var(--global-color)) 4%);
    box-shadow: -18px 0 42px rgba(0, 0, 0, 0.22);
    will-change: transform, opacity;
}

.drawer-panel-right {
    width: min(340px, 86vw);
    height: 100%;
}

.drawer-panel-bottom {
    width: 100%;
    max-height: 88dvh;
    box-shadow: 0 -18px 42px rgba(0, 0, 0, 0.22);
}

.drawer-header {
    display: flex;
    align-items: center;
    flex: 0 0 auto;
    min-height: 64px;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(var(--outline-color), 0.14);
}

.drawer-close-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    width: 36px;
    height: 36px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    color: rgb(var(--text-color));
    background: transparent;
    cursor: pointer;
}

.drawer-title {
    min-width: 0;
    overflow: hidden;
    font-size: 18px;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.drawer-header-actions {
    display: flex;
    align-items: center;
    margin-left: auto;
}

.drawer-content {
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
}

@media (max-width: 720px) {
    .drawer-layer-right {
        align-items: flex-end;
        justify-content: center;
    }

    .drawer-panel-right {
        width: 96vw;
        max-height: 88dvh;
        height: auto;
        min-height: 54dvh;
        border-radius: 18px 18px 0 0;
        box-shadow: 0 -18px 42px rgba(0, 0, 0, 0.22);
    }
}
</style>
