<template>
  <MotionButton class="ripple-button" :while-hover="hoverStyle"
    :while-press="pressStyle"
    :transition="microTransition" @mousedown="startRipple">
    <span class="ripple-content">
      <slot></slot>
    </span>
    <AnimatePresence>
      <MotionSpan v-for="ripple in ripples" :key="ripple.id" class="ripple"
        :initial="{ scale: 0, opacity: 0.3 }" :animate="{ scale: 1, opacity: 0 }"
        :exit="{ opacity: 0 }" :transition="rippleTransition"
        :style="{ left: `${ripple.x}px`, top: `${ripple.y}px`, width: `${ripple.size}px`, height: `${ripple.size}px` }"
        @animation-complete="removeRipple(ripple.id)" />
    </AnimatePresence>
  </MotionButton>
</template>

<script setup>
import { computed, ref } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
  hoverStyle: {
    type: Object,
    default: () => ({ backgroundColor: 'rgba(var(--background-color), 0.5)' })
  },
  pressStyle: {
    type: Object,
    default: () => ({ backgroundColor: 'rgba(var(--background-color), 0.3)', scale: 0.98 })
  }
})

const MotionButton = motion.button
const MotionSpan = motion.span
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const rippleTransition = computed(() => reducedMotion.value
  ? INSTANT_MOTION
  : { type: 'keyframes', duration: 0.5, ease: 'easeOut' })
const ripples = ref([])
const hoverStyle = computed(() => props.hoverStyle)
const pressStyle = computed(() => props.pressStyle)
let nextRippleId = 0

const startRipple = (event) => {
  const button = event.currentTarget
  const rect = button.getBoundingClientRect()
  const size = Math.max(rect.width, rect.height) * 2

  ripples.value.push({
    id: ++nextRippleId,
    x: event.clientX - rect.left - size / 2,
    y: event.clientY - rect.top - size / 2,
    size
  })
}

const removeRipple = (id) => {
  ripples.value = ripples.value.filter((ripple) => ripple.id !== id)
}
</script>

<style scoped>
.ripple-button {
  position: relative;
  overflow: hidden;
  color: rgba(var(--text-color));
  border: none;
  border-radius: 4px;
  outline: none;
}

.ripple {
  position: absolute;
  background-color: rgba(var(--text-color), 0.3);
  border-radius: 50%;
  pointer-events: none;
}
</style>
