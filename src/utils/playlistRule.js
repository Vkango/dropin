export const PLAYLIST_SOURCE_KINDS = ['library', 'playlist', 'tag']
export const PLAYLIST_OPERATORS = ['union', 'inter', 'concatenate', 'subtract', 'randomChoose']

export const sourceKey = (source) => `${source?.kind || ''}:${source?.id || ''}`

export const sourceStep = (source) => ({
  type: 'source',
  kind: source.kind,
  id: source.id ?? null
})

export const operatorStep = (op, count = null) => ({
  type: 'operator',
  op,
  count: op === 'randomChoose' ? Math.max(1, Math.floor(Number(count) || 1)) : null
})

export const emptyPlaylistRule = () => ({ version: 1, steps: [] })

export const clonePlaylistRule = (rule) => {
  if (!rule || typeof rule !== 'object') return emptyPlaylistRule()
  return {
    version: Number(rule.version) || 1,
    steps: Array.isArray(rule.steps)
      ? rule.steps.map((step) => step?.type === 'source'
        ? { type: 'source', kind: step.kind, id: step.id ?? null }
        : {
          type: 'operator',
          op: step?.op,
          count: step?.op === 'randomChoose'
            ? Math.max(1, Math.floor(Number(step.count) || 1))
            : null
        })
      : []
  }
}

export const isPlaylistRuleValid = (rule) => {
  if (!rule || Number(rule.version) !== 1 || !Array.isArray(rule.steps) || !rule.steps.length) return false
  if (rule.steps.length > 64 || rule.steps[0]?.type !== 'source') return false

  for (const step of rule.steps) {
    if (step?.type === 'source') {
      if (!PLAYLIST_SOURCE_KINDS.includes(step.kind)) return false
      if (step.kind === 'library' && step.id != null) return false
      if (step.kind !== 'library' && !String(step.id || '').trim()) return false
    }
  }

  let index = 1
  while (index < rule.steps.length) {
    const step = rule.steps[index]
    if (step?.type !== 'operator' || !PLAYLIST_OPERATORS.includes(step.op)) return false
    if (step.op === 'randomChoose') {
      if (!Number.isInteger(Number(step.count)) || Number(step.count) < 1 || index !== rule.steps.length - 1) return false
      index += 1
      continue
    }
    if (step.count != null || rule.steps[index + 1]?.type !== 'source') return false
    index += 2
  }

  return true
}

export const isPlaylistComposerValueValid = (value) => {
  if (!value || !['static', 'dynamic'].includes(value.mode)) return false
  if (value.mode === 'static') return Array.isArray(value.trackIds) && value.trackIds.length > 0
  return isPlaylistRuleValid(value.rule)
}
