export function createDropinPluginClient(target = window.parent) {
  let sequence = 0
  const pending = new Map()
  const listeners = new Map()

  const onMessage = (event) => {
    if (event.source !== target) return
    const message = event.data
    if (!message || message.type !== 'dropin:response') return
    const request = pending.get(message.requestId)
    if (!request) return
    pending.delete(message.requestId)
    if (message.ok) request.resolve(message.result)
    else request.reject(new Error(message.error || 'plugin request failed'))
  }
  window.addEventListener('message', onMessage)

  const onTheme = (event) => {
    if (event.source !== target || event.data?.type !== 'dropin:theme') return
    document.documentElement.dataset.dropinTheme = event.data.theme?.mode || ''
    const colors = event.data.theme?.colors || {}
    if (colors.primary) document.documentElement.style.setProperty('--dropin-primary', rgb(colors.primary))
    if (colors.background) document.documentElement.style.setProperty('--dropin-background', rgb(colors.background))
    if (colors.surface) document.documentElement.style.setProperty('--dropin-surface', rgb(colors.surface))
    if (colors.onSurface) document.documentElement.style.setProperty('--dropin-text', rgb(colors.onSurface))
  }
  window.addEventListener('message', onTheme)

  const rgb = (color) => `rgb(${color.r}, ${color.g}, ${color.b})`

  const call = (method, args = {}) => new Promise((resolve, reject) => {
    const requestId = `dropin-${++sequence}`
    pending.set(requestId, { resolve, reject })
    try {
      target.postMessage({ type: 'dropin:request', requestId, method, args }, '*')
    } catch (cause) {
      pending.delete(requestId)
      reject(cause)
    }
  })

  const unwrapBackendResponse = (value) => {
    if (!value || typeof value !== 'object' || typeof value.ok !== 'boolean') return value
    if (!('result' in value) && !('error' in value)) return value
    if (value.ok) return value.result
    throw new Error(value.error || 'plugin backend call failed')
  }
  const backendCall = (method, args = {}) => call(method.startsWith('backend.') ? method : `backend.${method}`, args).then(unwrapBackendResponse)
  const showNotification = (options = {}) => {
    const payload = typeof options === 'string' ? { body: options } : (options || {})
    return call('notification.show', payload)
  }

  const subscribe = (eventName, handler) => {
    const handlers = listeners.get(eventName) || new Set()
    handlers.add(handler)
    listeners.set(eventName, handlers)
    return () => handlers.delete(handler)
  }

  return {
    apiVersion: 1,
    host: { call },
    player: {
      getState: () => call('player.getState'),
      play: (args = {}) => call('player.play', args),
      pause: (args = {}) => call('player.pause', args)
    },
    library: { list: (args = {}) => call('library.list', args) },
    notification: { show: showNotification },
    storage: {
      get: (key) => call('storage.get', { key }),
      set: (key, value) => call('storage.set', { key, value }),
      remove: (key) => call('storage.remove', { key })
    },
    backend: { call: backendCall },
    events: { on: subscribe },
    dispose: () => { window.removeEventListener('message', onMessage); window.removeEventListener('message', onTheme); pending.forEach(({ reject }) => reject(new Error('plugin client disposed'))); pending.clear() }
  }
}
