import { invoke } from '@tauri-apps/api/core'
import { computed, reactive } from 'vue'
import { setLocale, setMessages, isLocaleLoaded } from '../i18n/index.js'

const state = reactive({
  available: [],
  ready: false
})

let loadPromise = null
const loadedCache = new Set()

export function hasLocale(locale) {
  return state.available.some((entry) => entry.code === locale)
}

export function useLocaleList() {
  return {
    available: computed(() => state.available)
  }
}

const labelFor = (code, name) =>
  (typeof name === 'string' && name.trim() ? name.trim() : code)

export async function refreshLocaleList() {
  if (loadPromise) return loadPromise
  loadPromise = invoke('i18n_list_custom')
    .then((entries) => {
      state.available = (Array.isArray(entries) ? entries : [])
        .filter((entry) => entry && typeof entry.code === 'string' && entry.code.length > 0)
        .map((entry) => ({
          code: entry.code,
          name: labelFor(entry.code, entry.name)
        }))
      return state.available
    })
    .catch((error) => {
      console.debug('i18n locale list failed:', error)
      return []
    })
    .finally(() => {
      state.ready = true
    })
  return loadPromise
}

export async function ensureLocaleLoaded(locale) {
  if (!locale || isLocaleLoaded(locale)) return
  if (loadedCache.has(locale)) return
  loadedCache.add(locale)
  try {
    const messages = await invoke('i18n_load_custom', { locale })
    if (messages) setMessages(locale, messages)
  } catch (error) {
    console.debug(`i18n load ${locale} failed:`, error)
  }
}

export function systemLocale() {
  const language = typeof navigator !== 'undefined'
    ? (navigator.language || navigator.userLanguage || '')
    : ''
  return language
}

export function bestAvailableLocale(locale) {
  if (state.available.some((entry) => entry.code === locale)) return locale
  const primary = (locale || '').split('-')[0]
  const match = state.available.find(
    (entry) => entry.code.toLowerCase() === primary.toLowerCase()
      || entry.code.split('-')[0].toLowerCase() === primary.toLowerCase()
  )
  return match ? match.code : ''
}

export async function activateLocale(locale) {
  await refreshLocaleList()
  if (!locale || locale === 'system') {
    locale = bestAvailableLocale(systemLocale())
    if (!locale) {
      const first = state.available[0]
      locale = first ? first.code : ''
    }
  } else {
    locale = bestAvailableLocale(locale) || locale
  }
  if (!locale) {
    setLocale('')
    return ''
  }
  await ensureLocaleLoaded(locale)
  const applied = setLocale(locale)
  return applied
}
