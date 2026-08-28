export const PLUGIN_PERMISSION_KEYS = [
  'ui.panel',
  'player.read',
  'player.control',
  'library.read',
  'notification.show',
  'storage.plugin'
]

const fallbackLabel = (permission) => String(permission || '')
  .split(/[._-]+/)
  .filter(Boolean)
  .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
  .join(' ')

const resolveText = (t, key, fallback) => {
  if (typeof t !== 'function') return fallback
  const value = t(key)
  return value === key ? fallback : value
}

export const pluginPermissionMeta = (permission, t) => {
  const key = String(permission || '')
  const fallback = fallbackLabel(key)

  return {
    key,
    label: resolveText(t, 'pluginPermissions.' + key + '.label', fallback),
    description: resolveText(t, 'pluginPermissions.' + key + '.description', key)
  }
}

