import { invoke } from '@tauri-apps/api/core'
import { computed, reactive } from 'vue'
import { frameRateLimits, setGlobalAnimationFrameRate } from '../utils/frameRateScheduler.js'

const SETTINGS_VERSION = 1
const SAVE_DELAY_MS = 220

const state = reactive({
  version: SETTINGS_VERSION,
  animationFrameRate: frameRateLimits.default,
  loading: false,
  saving: false,
  error: null
})

let loadPromise = null
let saveTimer = null
let savePromise = Promise.resolve()

const normalizeFrameRate = (value) => {
  if (value === null) return null
  const numericValue = Number(value)
  if (!Number.isFinite(numericValue)) return frameRateLimits.default
  return Math.round(Math.max(frameRateLimits.min, Math.min(frameRateLimits.max, numericValue)))
}

const normalizeSettings = (settings = {}) => ({
  version: SETTINGS_VERSION,
  animationFrameRate: normalizeFrameRate(settings.animationFrameRate)
})

const applySettings = (settings) => {
  const normalized = normalizeSettings(settings)
  state.version = normalized.version
  state.animationFrameRate = normalized.animationFrameRate
  setGlobalAnimationFrameRate(normalized.animationFrameRate)
  return normalized
}

const writeSettings = async () => {
  const settings = normalizeSettings(state)
  state.saving = true
  state.error = null
  try {
    const saved = await invoke('app_settings_write', { settings })
    applySettings(saved)
  } catch (error) {
    state.error = error?.message || String(error)
  } finally {
    state.saving = false
  }
}

const scheduleSave = () => {
  if (saveTimer) window.clearTimeout(saveTimer)
  saveTimer = window.setTimeout(() => {
    saveTimer = null
    savePromise = savePromise.then(writeSettings, writeSettings)
  }, SAVE_DELAY_MS)
}

export async function loadAppSettings() {
  if (loadPromise) return loadPromise

  state.loading = true
  state.error = null
  loadPromise = invoke('app_settings_read')
    .then((settings) => applySettings(settings))
    .catch((error) => {
      state.error = error?.message || String(error)
      return applySettings({ animationFrameRate: frameRateLimits.default })
    })
    .finally(() => {
      state.loading = false
    })

  return loadPromise
}

export function updateAnimationFrameRate(value) {
  state.animationFrameRate = normalizeFrameRate(value)
  setGlobalAnimationFrameRate(state.animationFrameRate)
  scheduleSave()
}

export function useAppSettingsStore() {
  return {
    state,
    isFrameRateUnlimited: computed(() => state.animationFrameRate === null),
    effectiveFrameRate: computed(() => state.animationFrameRate ?? '无限制'),
    load: loadAppSettings,
    updateAnimationFrameRate
  }
}

export { frameRateLimits }
