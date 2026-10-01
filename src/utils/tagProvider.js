export const defaultTagKey = (provider) => provider?.sourceProviderKey
  || (provider?.pluginId ? provider.key : provider?.tagId)
  || provider?.key
  || ''

export const isManualTag = (provider) => Boolean(provider)
  && !provider.sourceProviderKey
  && !provider.pluginId
  && provider.kind !== 'plugin'

// Tag 值统一转成可读文本（Rust 侧 value_json 已解析为原生 JSON 值）
export const formatTagValueText = (value) => {
  if (value === null || value === undefined) return ''
  if (Array.isArray(value)) return value.map((item) => String(item)).join(', ')
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}

// 展示形如 “Energy 30” 的标签徽标；手动标签的值就是标签名，避免 “Energy Energy”
export const formatTagBadge = (name, value) => {
  const text = formatTagValueText(value)
  if (!text || text === name) return name || ''
  return name ? `${name} ${text}` : text
}
