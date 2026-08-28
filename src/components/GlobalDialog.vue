<template>
  <Teleport to="body">
    <AnimatePresence :initial="false">
      <div v-if="dialogState.open" class="dialog-layer" role="presentation" @click.self="handleBackdropClick">
        <MotionDiv class="dialog-backdrop" aria-hidden="true" :initial="{ opacity: 0 }" :animate="{ opacity: 1 }"
          :exit="{ opacity: 0 }" :transition="backdropTransition" @click="handleBackdropClick" />

        <MotionDiv ref="panelRef" class="dialog-panel" :style="panelStyle" :initial="panelInitial"
          :animate="panelAnimate" :exit="panelExit" :transition="panelTransition" role="dialog" aria-modal="true"
          :aria-labelledby="titleId" :aria-describedby="dialogState.message ? messageId : undefined" tabindex="-1"
          @click.stop>
          <div class="dialog-flare" aria-hidden="true"></div>

          <header class="dialog-header">
            <div class="dialog-icon" :data-variant="dialogState.variant" aria-hidden="true">
              <span />
            </div>
            <div class="dialog-title-group">
              <span v-if="showEyebrow" class="dialog-eyebrow">{{ dialogEyebrow }}</span>
              <h2 :id="titleId" class="dialog-title">{{ dialogTitle }}</h2>
            </div>
          </header>

          <div class="dialog-body">
            <p v-if="dialogState.message" :id="messageId" class="dialog-message">
              {{ dialogState.message }}
            </p>

            <MotionDiv v-if="isPromptFieldVisible" class="dialog-input-shell" :animate="inputShellAnimate"
              :transition="microTransition">
              <input ref="inputRef" v-model="dialogInputValue" class="dialog-input" :type="dialogState.inputType"
                :placeholder="dialogState.inputPlaceholder" :autocomplete="dialogState.inputAutocomplete"
                :autocapitalize="dialogState.inputAutocapitalize" :spellcheck="dialogState.inputSpellcheck"
                :maxlength="dialogState.inputMaxlength ?? undefined" @focus="isInputFocused = true"
                @blur="isInputFocused = false" />
            </MotionDiv>

            <component v-else-if="dialogState.bodyComponent" :is="dialogState.bodyComponent" v-model="dialogInputValue"
              v-bind="dialogState.bodyProps" class="dialog-custom-body" />
          </div>

          <component v-if="dialogState.actionsComponent" :is="dialogState.actionsComponent" v-model="dialogInputValue"
            v-bind="customActionsProps" class="dialog-custom-actions" />

          <footer v-else-if="dialogState.showCancel || dialogState.showConfirm" class="dialog-actions">
            <MotionButton v-if="dialogState.showCancel" class="dialog-button dialog-button-secondary" type="button"
              :while-hover="buttonHover" :while-press="buttonPress" :transition="microTransition" @click="handleCancel">
              {{ cancelLabel }}
            </MotionButton>
            <MotionButton v-if="dialogState.showConfirm" class="dialog-button dialog-button-primary" type="button"
              :disabled="isConfirmDisabled" :while-hover="confirmHover" :while-press="buttonPress"
              :transition="microTransition" @click="handleConfirm">
              {{ confirmLabel }}
            </MotionButton>
          </footer>
        </MotionDiv>
      </div>
    </AnimatePresence>
  </Teleport>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { APPLE_SPRING, INSTANT_MOTION, MICRO_SPRING, SOFT_SPRING } from '../utils/motion.js'
import { useI18n } from '../i18n/index.js'
import {
  canSubmitDialog,
  cancelDialog,
  dialogState,
  submitDialog
} from '../services/dialogService.js'

const { t } = useI18n()

const MotionDiv = motion.div
const MotionButton = motion.button
const reducedMotion = useReducedMotion()

const panelRef = ref(null)
const inputRef = ref(null)
const isInputFocused = ref(false)
const isPageLocked = ref(false)
const previousFocusedElement = ref(null)
const previousBodyOverflow = ref('')
const titleId = 'dialog-title-' + Math.random().toString(36).slice(2)
const messageId = 'dialog-message-' + Math.random().toString(36).slice(2)

const panelTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const backdropTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)

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

const dialogInputValue = computed({
  get: () => dialogState.inputValue,
  set: (value) => {
    dialogState.inputValue = value
  }
})

const dialogVariantLabel = computed(() => ({
  prompt: t('dialog.variants.prompt'),
  confirm: t('dialog.variants.confirm'),
  alert: t('dialog.variants.alert'),
  custom: t('dialog.variants.confirm')
}[dialogState.variant] || t('dialog.variants.confirm')))

const dialogEyebrow = computed(() => dialogState.eyebrow || dialogVariantLabel.value)
const dialogTitle = computed(() => dialogState.title || dialogEyebrow.value)
const showEyebrow = computed(() => Boolean(dialogState.title && dialogEyebrow.value))
const confirmLabel = computed(() => dialogState.confirmLabel || (
  dialogState.variant === 'alert' ? t('dialog.actions.ok') : t('dialog.actions.confirm')
))
const cancelLabel = computed(() => dialogState.cancelLabel || t('dialog.actions.cancel'))
const panelStyle = computed(() => ({
  '--dialog-width': dialogState.width + 'px'
}))
const customActionsProps = computed(() => ({
  ...dialogState.actionsProps,
  canSubmit: canSubmitDialog,
  cancel: cancelDialog,
  submit: submitDialog
}))
const isPromptFieldVisible = computed(() => !dialogState.bodyComponent && dialogState.variant === 'prompt')
const isConfirmDisabled = computed(() => !canSubmitDialog())
const buttonHover = { y: -1, scale: 1.01 }
const confirmHover = { y: -1, scale: 1.02 }
const buttonPress = { scale: 0.98 }

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
  if (isPromptFieldVisible.value) {
    inputRef.value?.focus?.()
    inputRef.value?.select?.()
    return
  }

  const focusable = getFocusableElements()
  if (focusable.length) {
    focusable[0].focus()
    return
  }

  getPanelElement()?.focus?.()
}

const lockPage = () => {
  if (typeof document === 'undefined') return
  if (isPageLocked.value) return
  isPageLocked.value = true
  previousFocusedElement.value = document.activeElement instanceof HTMLElement ? document.activeElement : null
  previousBodyOverflow.value = document.body.style.overflow
  document.body.style.overflow = 'hidden'
  window.addEventListener('keydown', handleKeydown)
}

const unlockPage = () => {
  if (typeof document === 'undefined') return
  if (!isPageLocked.value) return
  isPageLocked.value = false
  window.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = previousBodyOverflow.value
  const element = previousFocusedElement.value
  if (element?.isConnected) {
    element.focus?.()
  }
  previousFocusedElement.value = null
}

const handleBackdropClick = () => {
  if (!dialogState.closeOnBackdrop) return
  cancelDialog('backdrop')
}

const handleConfirm = () => {
  submitDialog('button')
}

const handleCancel = () => {
  cancelDialog('button')
}

const handleKeydown = (event) => {
  if (!dialogState.open) return

  if (event.key === 'Escape' && dialogState.closeOnEscape) {
    event.preventDefault()
    cancelDialog('escape')
    return
  }

  if (event.key === 'Enter' && dialogState.submitOnEnter && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
    const target = event.target
    if (!(target instanceof HTMLTextAreaElement) && !target?.isContentEditable) {
      event.preventDefault()
      submitDialog('enter')
      return
    }
  }

  if (event.key === 'Tab') {
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
}

const inputShellAnimate = computed(() => (
  isInputFocused.value
    ? { scale: 1.01, y: -1 }
    : { scale: 1, y: 0 }
))

watch(() => dialogState.open, async (open) => {
  isInputFocused.value = false
  if (open) {
    lockPage()
    await focusInitialElement()
    return
  }

  unlockPage()
})

onMounted(() => {
  if (dialogState.open) {
    lockPage()
    void focusInitialElement()
  }
})

onBeforeUnmount(() => {
  unlockPage()
})
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
  background:
    radial-gradient(circle at top, rgba(var(--primary-color), 0.14), transparent 45%),
    rgba(4, 7, 12, 0.58);
}

.dialog-panel {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  width: min(var(--dialog-width, 460px), calc(100vw - 36px));
  max-height: calc(100dvh - 36px);
  overflow: hidden;
  color: rgb(var(--text-color));
  background:
    linear-gradient(180deg, rgba(var(--surface-color), 0.98), rgba(var(--surface-color), 0.92)),
    color-mix(in srgb, rgb(var(--surface-color)) 80%, transparent);
  border: 1px solid rgba(var(--outline-color), 0.16);
  border-radius: 24px;
  box-shadow:
    0 28px 72px rgba(0, 0, 0, 0.28),
    0 2px 10px rgba(0, 0, 0, 0.12);
  backdrop-filter: blur(30px) saturate(1.25);
  -webkit-backdrop-filter: blur(30px) saturate(1.25);
  will-change: transform, opacity, filter;
}

.dialog-flare {
  position: absolute;
  top: -48px;
  right: -56px;
  width: 160px;
  height: 160px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(var(--primary-color), 0.18), transparent 70%);
  filter: blur(14px);
  pointer-events: none;
}

.dialog-header {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 18px 18px 0;
}

.dialog-icon {
  position: relative;
  flex: 0 0 42px;
  width: 42px;
  height: 42px;
  border-radius: 14px;
  background: rgba(var(--primary-color), 0.16);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.18),
    0 8px 18px rgba(var(--primary-color), 0.12);
}

.dialog-icon span {
  position: absolute;
  inset: 12px;
  border-radius: 999px;
  background: rgba(var(--primary-color), 0.72);
}

.dialog-title-group {
  min-width: 0;
  flex: 1 1 auto;
  padding-top: 1px;
}

.dialog-eyebrow {
  display: block;
  margin-bottom: 4px;
  color: rgba(var(--text-color), 0.5);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.11em;
  text-transform: uppercase;
}

.dialog-title {
  color: rgb(var(--text-color));
  font-size: 18px;
  font-weight: 700;
  line-height: 1.25;
}

.dialog-body {
  display: grid;
  gap: 14px;
  min-height: 0;
  padding: 12px 18px 18px;
}

.dialog-message {
  color: rgba(var(--text-color), 0.66);
  font-size: 13px;
  line-height: 1.55;
  white-space: pre-wrap;
}

.dialog-input-shell {
  transform-origin: center;
}

.dialog-input {
  width: 100%;
  min-height: 48px;
  padding: 0 14px;
  color: rgb(var(--text-color));
  background: rgba(var(--surface-color), 0.86);
  border: 1px solid rgba(var(--outline-color), 0.18);
  border-radius: 16px;
  outline: none;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.16),
    0 1px 0 rgba(0, 0, 0, 0.02);
  transition:
    border-color 0.22s ease,
    box-shadow 0.22s ease,
    transform 0.22s ease,
    background-color 0.22s ease;
}

.dialog-input::placeholder {
  color: rgba(var(--text-color), 0.34);
}

.dialog-input:focus {
  transform: translateY(-1px);
  border-color: rgba(var(--primary-color), 0.62);
  box-shadow:
    0 0 0 4px rgba(var(--primary-color), 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.18);
}

.dialog-custom-body {
  min-width: 0;
}

.dialog-custom-actions {
  min-width: 0;
  padding: 0 18px 18px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 0 18px 18px;
}

.dialog-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 90px;
  height: 38px;
  padding: 0 14px;
  border: 1px solid transparent;
  border-radius: 12px;
  color: rgb(var(--text-color));
  font-size: 12.5px;
  font-weight: 650;
  letter-spacing: 0.01em;
  cursor: pointer;
}

.dialog-button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.dialog-button-secondary {
  background: rgba(var(--outline-color), 0.12);
  border-color: rgba(var(--outline-color), 0.08);
}

.dialog-button-primary {
  background: rgba(var(--primary-color), 0.22);
  border-color: rgba(var(--primary-color), 0.24);
}

.dialog-button-secondary:hover,
.dialog-button-primary:hover {
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.08);
}

@media (max-width: 640px) {
  .dialog-layer {
    padding: 14px;
  }

  .dialog-panel {
    width: min(100%, calc(100vw - 28px));
    border-radius: 22px;
  }

  .dialog-header {
    padding-inline: 16px;
  }

  .dialog-body {
    padding-inline: 16px;
  }

  .dialog-actions {
    padding-inline: 16px;
  }

  .dialog-custom-actions {
    padding-inline: 16px;
  }
}
</style>
