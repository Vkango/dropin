import { computed, reactive } from 'vue'

const state = reactive({
  locale: '',
  messages: {}
})

const isPlainObject = (value) =>
  typeof value === 'object' && value !== null && !Array.isArray(value)

const lookupKey = (messages, key) => {
  if (!isPlainObject(messages) || typeof key !== 'string') return undefined
  if (Object.prototype.hasOwnProperty.call(messages, key)) return messages[key]
  const segments = key.split('.')
  let node = messages
  for (const segment of segments) {
    if (!isPlainObject(node) || !Object.prototype.hasOwnProperty.call(node, segment)) return undefined
    node = node[segment]
  }
  return node
}

const interpolate = (template, params) => {
  if (typeof template !== 'string') return String(template)
  if (!params) return template
  return template.replace(/\{(\w+)\}/g, (match, name) =>
    Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : match
  )
}

export const t = (key, params) => {
  const resolved = lookupKey(state.messages[state.locale], key)
  if (typeof resolved === 'string') return interpolate(resolved, params)
  return key
}

export const setLocale = (locale) => {
  state.locale = isPlainObject(state.messages[locale]) ? locale : ''
  return state.locale
}

export const getLocale = () => state.locale

export const supportedLocales = () => Object.keys(state.messages)

export const isLocaleLoaded = (locale) => isPlainObject(state.messages[locale])

export const setMessages = (locale, messages) => {
  if (!locale || !isPlainObject(messages)) return false
  state.messages[locale] = messages
  return true
}

export const useI18n = () => ({
  t,
  locale: computed(() => state.locale),
  setLocale
})

export default { t, setLocale, getLocale, supportedLocales, isLocaleLoaded, setMessages, useI18n }
