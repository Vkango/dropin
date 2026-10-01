export const defaultTagKey = (provider) => provider?.sourceProviderKey
  || (provider?.pluginId ? provider.key : provider?.tagId)
  || provider?.key
  || ''

export const isManualTag = (provider) => Boolean(provider)
  && !provider.sourceProviderKey
  && !provider.pluginId
  && provider.kind !== 'plugin'
