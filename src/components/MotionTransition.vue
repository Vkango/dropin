<script setup>
import { computed } from 'vue'
import { useReducedMotion } from 'motion-v'
import { animateElement, motionVariants } from '../utils/motion.js'

const props = defineProps({
  variant: {
    type: String,
    default: 'page'
  },
  mode: {
    type: String,
    default: undefined
  },
  appear: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['after-enter', 'after-leave'])

const reducedMotion = useReducedMotion()
const currentVariant = computed(() => motionVariants[props.variant] || motionVariants.page)

const setAbsolutePosition = (element) => {
  if (!currentVariant.value.absolute) return
  element.dataset.motionAbsolute = 'true'
  element.style.position = 'absolute'
  element.style.inset = '0'
}

const clearTemporaryStyles = (element) => {
  if (element.dataset.motionAbsolute !== 'true') return
  delete element.dataset.motionAbsolute
  element.style.position = ''
  element.style.inset = ''
}

const run = (element, keyframes, done) => {
  const transition = reducedMotion.value
    ? { type: 'keyframes', duration: 0 }
    : currentVariant.value.spring

  const animation = animateElement(element, keyframes, transition, reducedMotion.value)
  let completed = false
  const complete = () => {
    if (completed) return
    completed = true
    done()
  }

  animation.finished.then(complete, complete)
}

const beforeEnter = (element) => setAbsolutePosition(element)
const beforeLeave = (element) => setAbsolutePosition(element)
const enter = (element, done) => run(element, currentVariant.value.enter, done)
const leave = (element, done) => run(element, currentVariant.value.leave, done)
const afterEnter = (element) => {
  clearTemporaryStyles(element)
  emit('after-enter')
}
const afterLeave = (element) => {
  clearTemporaryStyles(element)
  emit('after-leave')
}
</script>

<template>
  <Transition
    :css="false"
    :mode="mode"
    :appear="appear"
    @before-enter="beforeEnter"
    @before-leave="beforeLeave"
    @enter="enter"
    @leave="leave"
    @after-enter="afterEnter"
    @after-leave="afterLeave"
  >
    <slot />
  </Transition>
</template>
