import { markRaw, reactive } from 'vue'

const CLEANUP_DELAY = 320

const createInitialState = () => ({
  open: false,
  session: 0,
  variant: 'confirm',
  title: '',
  eyebrow: '',
  message: '',
  bodyComponent: null,
  bodyProps: {},
  actionsComponent: null,
  actionsProps: {},
  inputValue: '',
  inputType: 'text',
  inputPlaceholder: '',
  inputAutocomplete: 'off',
  inputAutocapitalize: 'off',
  inputSpellcheck: false,
  inputMaxlength: null,
  required: false,
  validate: null,
  width: 460,
  showCancel: true,
  showConfirm: true,
  closeOnBackdrop: true,
  closeOnEscape: true,
  submitOnEnter: true,
  confirmLabel: '',
  cancelLabel: '',
  confirmDisabled: false
})

export const dialogState = reactive(createInitialState())

let activeResolve = null
let nextSession = 0
let cleanupTimer = 0

const clearCleanupTimer = () => {
  if (cleanupTimer) {
    clearTimeout(cleanupTimer)
    cleanupTimer = 0
  }
}

const resetDialogState = () => {
  Object.assign(dialogState, createInitialState())
}

const scheduleReset = (session) => {
  clearCleanupTimer()
  cleanupTimer = setTimeout(() => {
    cleanupTimer = 0
    if (!dialogState.open && dialogState.session === session) {
      resetDialogState()
    }
  }, CLEANUP_DELAY)
}

const settleDialog = (action, reason) => {
  const session = dialogState.session
  const resolve = activeResolve
  activeResolve = null
  dialogState.open = false

  if (resolve) {
    resolve({
      action,
      reason,
      value: action === 'confirm' ? dialogState.inputValue : null
    })
  }

  scheduleReset(session)
  return Boolean(resolve)
}

export const canSubmitDialog = () => {
  if (dialogState.confirmDisabled) return false
  if (dialogState.variant === 'prompt' && dialogState.required && typeof dialogState.inputValue === 'string') {
    if (!dialogState.inputValue.trim()) return false
  }
  if (typeof dialogState.validate === 'function') {
    return dialogState.validate(dialogState.inputValue, dialogState) !== false
  }
  return true
}

export const submitDialog = (reason = 'confirm') => {
  if (!canSubmitDialog()) return false
  return settleDialog('confirm', reason)
}

export const cancelDialog = (reason = 'cancel') => settleDialog('cancel', reason)

export const openDialog = (options = {}) => {
  if (dialogState.open) {
    cancelDialog('replaced')
  }

  clearCleanupTimer()
  const variant = options.variant ?? 'confirm'
  const session = nextSession += 1

  Object.assign(dialogState, createInitialState(), {
    open: true,
    session,
    variant,
    title: options.title ?? '',
    eyebrow: options.eyebrow ?? '',
    message: options.message ?? '',
    bodyComponent: options.bodyComponent ? markRaw(options.bodyComponent) : null,
    bodyProps: options.bodyProps ? { ...options.bodyProps } : {},
    actionsComponent: options.actionsComponent ? markRaw(options.actionsComponent) : null,
    actionsProps: options.actionsProps ? { ...options.actionsProps } : {},
    inputValue: options.value ?? options.inputValue ?? '',
    inputType: options.inputType ?? 'text',
    inputPlaceholder: options.inputPlaceholder ?? '',
    inputAutocomplete: options.inputAutocomplete ?? 'off',
    inputAutocapitalize: options.inputAutocapitalize ?? 'off',
    inputSpellcheck: options.inputSpellcheck ?? false,
    inputMaxlength: options.inputMaxlength ?? null,
    required: options.required ?? (variant === 'prompt'),
    validate: typeof options.validate === 'function' ? options.validate : null,
    width: options.width ?? (variant === 'prompt' ? 420 : 460),
    showCancel: options.showCancel ?? (variant !== 'alert'),
    showConfirm: options.showConfirm ?? true,
    closeOnBackdrop: options.closeOnBackdrop ?? (variant === 'prompt' ? false : true),
    closeOnEscape: options.closeOnEscape ?? true,
    submitOnEnter: options.submitOnEnter ?? (variant !== 'custom'),
    confirmLabel: options.confirmLabel ?? '',
    cancelLabel: options.cancelLabel ?? '',
    confirmDisabled: Boolean(options.confirmDisabled)
  })

  return new Promise((resolve) => {
    activeResolve = resolve
  })
}

export const promptDialog = async (options = {}) => {
  const result = await openDialog({ ...options, variant: 'prompt' })
  return result.action === 'confirm' ? String(result.value ?? '') : null
}

export const confirmDialog = async (options = {}) => {
  const result = await openDialog({ ...options, variant: 'confirm' })
  return result.action === 'confirm'
}

export const alertDialog = async (options = {}) => {
  const result = await openDialog({
    ...options,
    variant: 'alert',
    showCancel: false,
    closeOnBackdrop: true,
    submitOnEnter: true
  })
  return result.action === 'confirm'
}
