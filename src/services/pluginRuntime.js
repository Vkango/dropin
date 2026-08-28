import { computed, reactive, readonly } from 'vue'
import { pluginApi } from './pluginApi.js'

export function createPluginRuntime() {
  const state = reactive({ plugins: [], loading: false, error: '' })

  const refresh = async () => {
    state.loading = true
    state.error = ''
    try {
      state.plugins = await pluginApi.list()
      return state.plugins
    } catch (error) {
      state.error = error?.message || String(error)
      throw error
    } finally {
      state.loading = false
    }
  }

  const update = async (operation, id) => {
    const plugin = await operation(id)
    const index = state.plugins.findIndex((item) => item.id === id)
    if (index >= 0) state.plugins[index] = plugin
    else state.plugins.push(plugin)
    return plugin
  }

  const installFromPicker = async () => {
    const path = await pluginApi.pickPackage()
    if (!path) return null
    const plugin = await pluginApi.install(path)
    state.plugins.push(plugin)
    return plugin
  }

  const permissions = (id) => pluginApi.getPermissions(id)
  const setPermissions = async (id, granted) => update((value) => pluginApi.setPermissions(value, granted), id)
  const enable = async (id) => update(pluginApi.enable, id)
  const disable = async (id) => update(pluginApi.disable, id)
  const uninstall = async (id) => {
    await pluginApi.uninstall(id)
    state.plugins = state.plugins.filter((plugin) => plugin.id !== id)
  }

  const categories = computed(() => {
    const result = [{ id: 'all', count: state.plugins.length }]
    const counts = new Map()
    state.plugins.forEach((plugin) => (plugin.categories || []).forEach((category) => counts.set(category, (counts.get(category) || 0) + 1)))
    counts.forEach((count, id) => result.push({ id, count }))
    return result
  })

  const setError = (error) => { state.error = error?.message || String(error) }

  return {
    state: readonly(state),
    categories,
    refresh,
    installFromPicker,
    permissions,
    setPermissions,
    enable,
    disable,
    uninstall,
    setError,
    call: pluginApi.call,
    uiUrl: pluginApi.uiUrl
  }
}
