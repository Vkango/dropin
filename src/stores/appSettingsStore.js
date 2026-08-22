import { invoke } from '@tauri-apps/api/core'
import { computed, reactive } from 'vue'
import { frameRateLimits, setGlobalAnimationFrameRate } from '../utils/frameRateScheduler.js'

const SETTINGS_VERSION = 4
const SAVE_DELAY_MS = 220
const DEFAULT_THEME_MODE = 'system'
const DEFAULT_AUTO_ALBUM_THEME = true
const DEFAULT_MANUAL_THEME_COLOR = '#88d0ec'
const DEFAULT_LANGUAGE = 'system'
const DEFAULT_SIDEBAR_WIDTH = 280
const SIDEBAR_MIN_WIDTH = 200
const SIDEBAR_MAX_WIDTH = 480
const THEME_MODES = ['system', 'light', 'dark']
const LANGUAGE_PATTERN = /^[a-zA-Z0-9_-]{1,32}$/

const state = reactive({
  version: SETTINGS_VERSION,
  animationFrameRate: frameRateLimits.default,
  themeMode: DEFAULT_THEME_MODE,
  autoAlbumTheme: DEFAULT_AUTO_ALBUM_THEME,
  manualThemeColor: DEFAULT_MANUAL_THEME_COLOR,
  language: DEFAULT_LANGUAGE,
  sidebarWidth: DEFAULT_SIDEBAR_WIDTH,
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

const normalizeThemeMode = (value) => THEME_MODES.includes(value) ? value : DEFAULT_THEME_MODE

const normalizeManualThemeColor = (value) => {
  if (typeof value !== 'string' || !/^#[0-9a-f]{6}$/i.test(value)) return DEFAULT_MANUAL_THEME_COLOR
  return value.toLowerCase()
}

const normalizeLanguage = (value) => {
  if (typeof value !== 'string') return DEFAULT_LANGUAGE
  const trimmed = value.trim()
  return LANGUAGE_PATTERN.test(trimmed) ? trimmed : DEFAULT_LANGUAGE
}

const normalizeSidebarWidth = (value) => {
  const numericValue = Number(value)
  if (!Number.isFinite(numericValue)) return DEFAULT_SIDEBAR_WIDTH
  return Math.round(Math.max(SIDEBAR_MIN_WIDTH, Math.min(SIDEBAR_MAX_WIDTH, numericValue)))
}

const normalizeSettings = (settings = {}) => ({
  version: SETTINGS_VERSION,
  animationFrameRate: normalizeFrameRate(settings.animationFrameRate),
  themeMode: normalizeThemeMode(settings.themeMode),
  autoAlbumTheme: settings.autoAlbumTheme !== false,
  manualThemeColor: normalizeManualThemeColor(settings.manualThemeColor),
  language: normalizeLanguage(settings.language),
  sidebarWidth: normalizeSidebarWidth(settings.sidebarWidth)
})

const applySettings = (settings) => {
  const normalized = normalizeSettings(settings)
  state.version = normalized.version
  state.animationFrameRate = normalized.animationFrameRate
  state.themeMode = normalized.themeMode
  state.autoAlbumTheme = normalized.autoAlbumTheme
  state.manualThemeColor = normalized.manualThemeColor
  state.language = normalized.language
  state.sidebarWidth = normalized.sidebarWidth
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

export function updateThemeMode(value) {
  state.themeMode = normalizeThemeMode(value)
  scheduleSave()
}

export function updateAutoAlbumTheme(value) {
  state.autoAlbumTheme = Boolean(value)
  scheduleSave()
}

export function updateManualThemeColor(value) {
  state.manualThemeColor = normalizeManualThemeColor(value)
  scheduleSave()
}

export function updateLanguage(value) {
  state.language = normalizeLanguage(value)
  scheduleSave()
}

export function updateSidebarWidth(value) {
  state.sidebarWidth = normalizeSidebarWidth(value)
  scheduleSave()
}

export function useAppSettingsStore() {
  return {
    state,
    isFrameRateUnlimited: computed(() => state.animationFrameRate === null),
    effectiveFrameRate: computed(() => state.animationFrameRate ?? '无限制'),
    load: loadAppSettings,
    updateAnimationFrameRate,
    updateThemeMode,
    updateAutoAlbumTheme,
    updateManualThemeColor,
    updateLanguage,
    updateSidebarWidth
  }
}

export {
  frameRateLimits,
  DEFAULT_THEME_MODE,
  DEFAULT_AUTO_ALBUM_THEME,
  DEFAULT_MANUAL_THEME_COLOR,
  DEFAULT_LANGUAGE,
  DEFAULT_SIDEBAR_WIDTH,
  SIDEBAR_MIN_WIDTH,
  SIDEBAR_MAX_WIDTH,
  THEME_MODES
}
