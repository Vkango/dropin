<template>
  <Teleport to="body">
    <AnimatePresence>
      <div v-if="modelValue" class="dialog-layer" role="presentation" @click.self="requestClose('backdrop')">
        <MotionDiv class="dialog-backdrop" aria-hidden="true" :initial="{ opacity: 0 }" :animate="{ opacity: 1 }"
          :exit="{ opacity: 0 }" :transition="backdropTransition" @click="requestClose('backdrop')" />

        <MotionDiv ref="panelRef" v-bind="$attrs" class="dialog-container" :style="containerStyle"
          :initial="panelInitial" :animate="panelAnimate" :exit="panelExit" :transition="panelTransition" role="dialog"
          aria-modal="true" :aria-label="ariaLabel || undefined" :aria-labelledby="ariaLabelledby || undefined"
          :aria-describedby="ariaDescribedby || undefined" tabindex="-1" @click.stop>
          <slot />
        </MotionDiv>
      </div>
    </AnimatePresence>
  </Teleport>
</template>

<script setup>
import { computed, inject, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { APPLE_SPRING, INSTANT_MOTION, SOFT_SPRING } from '@/utils/motion.js'

defineOptions({ inheritAttrs: false })

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  width: {
    type: [Number, String],
    default: 460
  },
  height: {
    type: [Number, String],
    default: null
  },
  maxHeight: {
    type: [Number, String],
    default: null
  },
  closeOnBackdrop: {
    type: Boolean,
    default: true
  },
  closeOnEscape: {
    type: Boolean,
    default: true
  },
  ariaLabel: {
    type: String,
    default: ''
  },
  ariaLabelledby: {
    type: String,
    default: ''
  },
  ariaDescribedby: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:modelValue', 'close'])

const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const currentSong = inject('currentSong', null)
const panelRef = ref(null)
const isPageLocked = ref(false)
const previousFocusedElement = ref(null)
const previousBodyOverflow = ref('')

const accentCache = new Map()

const extractAccentColor = (src) => {
  if (!src) return null
  const cached = accentCache.get(src)
  if (cached) return cached
  let color = null
  try {
    const image = new Image()
    image.crossOrigin = 'anonymous'
    image.onload = () => {
      try {
        const canvas = document.createElement('canvas')
        const size = 16
        canvas.width = size
        canvas.height = size
        const context = canvas.getContext('2d')
        context.drawImage(image, 0, 0, size, size)
        const { data } = context.getImageData(0, 0, size, size)
        let r = 0
        let g = 0
        let b = 0
        let count = 0
        for (let i = 0; i < data.length; i += 16) {
          r += data[i]
          g += data[i + 1]
          b += data[i + 2]
          count += 1
        }
        if (count > 0) {
          color = [Math.round(r / count), Math.round(g / count), Math.round(b / count)]
          accentCache.set(src, color)
        }
      } catch (error) {
        console.error('提取主题色失败:', error)
      }
    }
    image.src = src
  } catch (error) {
    console.error('提取主题色失败:', error)
  }
  return color
}

const accentColor = computed(() => {
  const cover = currentSong?.value?.cover
  return cover ? extractAccentColor(cover) : null
})

const toCssLength = (value) => {
  if (value === null || value === undefined || value === '') return null
  return typeof value === 'number' ? `${value}px` : value
}

const containerStyle = computed(() => {
  const style = {
    '--dialog-width': toCssLength(props.width)
  }
  const height = toCssLength(props.height)
  const maxHeight = toCssLength(props.maxHeight)
  if (height) style['--dialog-height'] = height
  if (maxHeight) style['--dialog-max-height'] = maxHeight
  if (accentColor.value) {
    style['--dialog-accent'] = accentColor.value.join(' ')
  }
  return style
})

const panelTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const backdropTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)

const panelInitial = computed(() => (
  reducedMotion.value
    ? { opacity: 0, y: 0, scale: 1 }
    : { opacity: 0, y: 18, scale: 0.94, filter: 'blur(10px)' }
))

const panelAnimate = computed(() => (
  reducedMotion.value
    ? { opacity: 1, y: 0, scale: 1 }
    : { opacity: 1, y: 0, scale: 1, filter: 'blur(0px)' }
))

const panelExit = computed(() => (
  reducedMotion.value
    ? { opacity: 0, y: 0, scale: 1 }
    : { opacity: 0, y: 12, scale: 0.96, filter: 'blur(8px)' }
))

const getPanelElement = () => panelRef.value?.$el ?? panelRef.value

const isFocusable = (element) => {
  if (!(element instanceof HTMLElement)) return false
  if (element.hasAttribute('disabled')) return false
  if (element.getAttribute('aria-hidden') === 'true') return false
  const style = window.getComputedStyle(element)
  return style.display !== 'none' && style.visibility !== 'hidden' && element.tabIndex >= 0
}

const getFocusableElements = () => {
  const root = getPanelElement()
  if (!(root instanceof HTMLElement)) return []
  return Array.from(root.querySelectorAll([
    'button',
    'input',
    'select',
    'textarea',
    'a[href]',
    '[tabindex]'
  ].join(','))).filter(isFocusable)
}

const focusInitialElement = async () => {
  await nextTick()
  const focusable = getFocusableElements()
  if (focusable.length) {
    focusable[0].focus()
    return
  }
  getPanelElement()?.focus?.()
}

const requestClose = (reason) => {
  if (reason === 'backdrop' && !props.closeOnBackdrop) return
  if (reason === 'escape' && !props.closeOnEscape) return
  emit('update:modelValue', false)
  emit('close', reason)
}

const handleKeydown = (event) => {
  if (!props.modelValue) return

  if (event.key === 'Escape' && props.closeOnEscape) {
    event.preventDefault()
    requestClose('escape')
    return
  }

  if (event.key !== 'Tab') return
  const focusable = getFocusableElements()
  if (!focusable.length) {
    event.preventDefault()
    getPanelElement()?.focus?.()
    return
  }

  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  const active = document.activeElement
  const root = getPanelElement()
  const inside = root instanceof HTMLElement ? root.contains(active) : false

  if (event.shiftKey) {
    if (active === first || !inside) {
      event.preventDefault()
      last.focus()
    }
    return
  }

  if (active === last || !inside) {
    event.preventDefault()
    first.focus()
  }
}

const lockPage = () => {
  if (typeof document === 'undefined' || isPageLocked.value) return
  isPageLocked.value = true
  previousFocusedElement.value = document.activeElement instanceof HTMLElement ? document.activeElement : null
  previousBodyOverflow.value = document.body.style.overflow
  document.body.style.overflow = 'hidden'
  window.addEventListener('keydown', handleKeydown)
}

const unlockPage = () => {
  if (typeof document === 'undefined' || !isPageLocked.value) return
  isPageLocked.value = false
  window.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = previousBodyOverflow.value
  if (previousFocusedElement.value?.isConnected) previousFocusedElement.value.focus?.()
  previousFocusedElement.value = null
}

watch(() => props.modelValue, async (open) => {
  if (open) {
    lockPage()
    await focusInitialElement()
  } else {
    unlockPage()
  }
})

onMounted(() => {
  if (props.modelValue) {
    lockPage()
    void focusInitialElement()
  }
})

onBeforeUnmount(unlockPage)
</script>

<style scoped>
.dialog-layer {
  position: fixed;
  inset: 0;
  z-index: 1400;
  display: grid;
  place-items: center;
  padding: 18px;
  pointer-events: auto;
}

.dialog-layer * {
  -webkit-user-select: none;
  user-select: none;
}

.dialog-layer input,
.dialog-layer textarea,
.dialog-layer [contenteditable='true'] {
  -webkit-user-select: text;
  user-select: text;
}

.dialog-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(4, 7, 12, 0.58);
}

.dialog-container {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  width: min(var(--dialog-width, 460px), calc(100vw - 36px));
  height: var(--dialog-height, auto);
  max-height: var(--dialog-max-height, calc(100dvh - 36px));
  overflow: hidden;
  color: rgb(var(--text-color));
  background-color: color-mix(in srgb, rgba(var(--primary-color), 0.5) 25%, rgba(var(--global-color), 0.5) 75%);
  border: 1px solid rgba(var(--outline-color), 0.16);
  border-radius: 15px;
  backdrop-filter: blur(20px);
  box-shadow:
    0 28px 72px rgba(0, 0, 0, 0.28),
    0 2px 10px rgba(0, 0, 0, 0.12);
  will-change: transform, opacity, filter;
}

@media (max-width: 640px) {
  .dialog-layer {
    padding: 14px;
  }

  .dialog-container {
    width: min(100%, calc(100vw - 28px));
    border-radius: 22px;
  }
}
</style>

<style>
.dialog-content {
  display: grid;
  gap: 14px;
  padding: 22px;
}

.dialog-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.dialog-header h2 {
  color: rgb(var(--text-color));
  font-size: 18px;
  font-weight: 700;
  line-height: 1.25;
}

.dialog-message {
  color: rgba(var(--text-color), 0.66);
  font-size: 13px;
  line-height: 1.55;
}

.dialog-input {
  width: 100%;
  min-height: 48px;
  padding: 0 14px;
  color: rgb(var(--text-color));
  background: rgba(var(--global-inverse-color), 0.06);
  border: 1px solid rgba(var(--outline-color), 0.18);
  border-radius: 16px;
  outline: none;
}

.dialog-input::placeholder {
  color: rgba(var(--text-color), 0.34);
}

.dialog-input:focus {
  border-color: rgba(var(--primary-color), 0.62);
  box-shadow: 0 0 0 4px rgba(var(--primary-color), 0.12);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 4px;
}

.dialog-button {
  min-width: 90px;
  min-height: 38px;
  padding: 0 14px;
  border: 1px solid transparent;
  border-radius: 12px;
  color: rgb(var(--text-color));
  font-size: 12.5px;
  font-weight: 650;
  cursor: pointer;
}

.dialog-button.secondary {
  background: rgba(var(--outline-color), 0.12);
  border-color: rgba(var(--outline-color), 0.08);
}

.dialog-button.primary {
  background: rgba(var(--primary-color), 0.22);
  border-color: rgba(var(--primary-color), 0.24);
}

.dialog-button.danger {
  color: #f44336;
  background: rgba(244, 67, 54, 0.12);
  border-color: rgba(244, 67, 54, 0.16);
}

.dialog-button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

@media (max-width: 640px) {
  .dialog-content {
    padding: 18px;
  }
}
</style>
