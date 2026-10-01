import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const call = (command, args = {}) => invoke(command, args)

export const pluginApi = {
  list: () => call('plugin_list'),
  pickPackage: () => call('plugin_pick_package'),
  install: (path) => call('plugin_install', { path }),
  uninstall: (id) => call('plugin_uninstall', { id }),
  enable: (id) => call('plugin_enable', { id }),
  disable: (id) => call('plugin_disable', { id }),
  getPermissions: (id) => call('plugin_get_permissions', { id }),
  setPermissions: (id, granted) => call('plugin_set_permissions', { id, granted }),
  call: (id, method, args = {}) => call('plugin_call', { id, method, args }),
  updateHostState: (state) => call('plugin_update_host_state', { state }),
  uiUrl: (id) => call('plugin_get_ui_url', { id }),
  tagProviders: () => call('tag_provider_list'),
  tagProviderRefresh: (providerKey) => call('tag_provider_refresh', { providerKey }),
  tagProviderCancel: (jobId) => call('tag_provider_cancel', { jobId }),
  tagProviderResults: (providerKey, options = {}) => call('tag_provider_results', {
    providerKey,
    key: options.key || null,
    search: options.search || null,
    limit: options.limit,
    offset: options.offset
  }),
  tagProviderWiki: (providerKey, locale = '', provided = false) => call('tag_provider_wiki', { providerKey, locale, provided }),
  pluginWiki: (pluginId, locale = '') => call('plugin_wiki', { pluginId, locale })
}

export const listenToPluginEvents = async (handler) => {
  const eventNames = [
    'plugin/notification',
    'tag/analysis-progress',
    'tag/analysis-finished',
    'tag/analysis-error'
  ]
  const unlisteners = await Promise.all(eventNames.map((eventName) =>
    listen(eventName, (event) => handler(eventName, event.payload))
  ))
  return () => unlisteners.forEach((unlisten) => unlisten())
}
