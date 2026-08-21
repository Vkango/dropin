<template>
    <Teleport to="body">
        <AnimatePresence>
            <MotionDiv v-if="open" ref="cardRef" class="popover-card"
                :class="[placement, { 'is-positioned': isPositioned }]" :style="positionStyle"
                :initial="{ opacity: 0, y: placement === 'below' ? -10 : 10, scale: 0.94, filter: 'blur(6px)' }"
                :animate="{ opacity: 1, y: 0, scale: 1, filter: 'blur(0px)' }"
                :exit="{ opacity: 0, y: placement === 'below' ? -8 : 8, scale: 0.94, filter: 'blur(6px)' }"
                :transition="cardTransition" @click.stop>
                <slot />
            </MotionDiv>
        </AnimatePresence>
    </Teleport>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { APPLE_SPRING, INSTANT_MOTION } from '../utils/motion.js'

const props = defineProps({
    open: Boolean,
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
    },
    width: {
        type: Number,
        default: 238
    }
})

const emit = defineEmits(['close'])
const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const cardRef = ref(null)
const isPositioned = ref(false)
const position = ref({ left: 8, top: 8, bottom: 8 })
let positionFrame = 0
const cardTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const positionStyle = computed(() => ({
    '--popover-width': `${props.width}px`,
    left: `${position.value.left}px`,
    ...(props.placement === 'below'
        ? { top: `${position.value.top}px` }
        : { bottom: `${position.value.bottom}px` })
}))

const getAnchorElement = () => {
    if (props.anchorId) return document.getElementById(props.anchorId)
    return props.anchor?.value ?? props.anchor
}

const updatePosition = (attempt = 0) => {
    const anchor = getAnchorElement()
    if (!anchor) {
        if (attempt < 10) positionFrame = requestAnimationFrame(() => updatePosition(attempt + 1))
        return
    }

    const rect = anchor.getBoundingClientRect()
    const maxLeft = Math.max(8, window.innerWidth - props.width - 8)
    const left = Math.max(8, Math.min(maxLeft, rect.left + rect.width / 2 - props.width / 2))
    const nextPosition = props.placement === 'below'
        ? { left, top: rect.bottom + props.gap }
        : { left, bottom: window.innerHeight - rect.top + props.gap }

    position.value = { ...position.value, ...nextPosition }
    isPositioned.value = true
}

const handleDocumentPointerdown = (event) => {
    const anchor = getAnchorElement()
    if (cardRef.value && !cardRef.value.contains(event.target) && !anchor?.contains(event.target)) emit('close')
}

const handleViewportChange = () => {
    if (props.open) updatePosition()
}

watch(() => props.open, async (open) => {
    isPositioned.value = false
    if (!open) return
    await nextTick()
    updatePosition()
})

onMounted(() => {
    document.addEventListener('pointerdown', handleDocumentPointerdown)
    window.addEventListener('resize', handleViewportChange)
})

onBeforeUnmount(() => {
    cancelAnimationFrame(positionFrame)
    document.removeEventListener('pointerdown', handleDocumentPointerdown)
    window.removeEventListener('resize', handleViewportChange)
})
</script>

<style scoped>
.popover-card {
    position: fixed;
    z-index: 1200;
    width: var(--popover-width, 238px);
    padding: 12px;
    color: rgb(var(--text-color));
    background: color-mix(in srgb, rgb(var(--surface-color)) 62%, transparent);
    border: 1px solid rgba(var(--outline-color), 0.16);
    border-radius: 16px;
    box-shadow: 0 18px 46px rgba(0, 0, 0, 0.24), 0 2px 8px rgba(0, 0, 0, 0.12);
    backdrop-filter: blur(28px) saturate(1.3);
    will-change: transform, opacity, filter;
}

.popover-card:not(.is-positioned) {
    visibility: hidden;
    pointer-events: none;
}
</style>
