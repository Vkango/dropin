import { reactive } from 'vue'
import { bassCall } from './bassApi.js'

const clone = (value) => {
  if (value === undefined) return undefined
  return JSON.parse(JSON.stringify(value))
}

const mergeParameters = (base, saved) => {
  if (!saved || typeof saved !== 'object' || Array.isArray(saved)) return clone(base || {})
  return { ...(base || {}), ...clone(saved) }
}

const emptySettings = () => ({})
const EQUALIZER_KIND = 'bassFx.peakeq'
const DEFAULT_EQUALIZER_BANDS = [31, 63, 125, 250, 500, 1000, 2000, 4000, 8000, 16000]
  .map((frequency) => ({ frequency, gain: 0 }))

const normalizeEqualizerBands = (value) => {
  const source = Array.isArray(value) ? value : DEFAULT_EQUALIZER_BANDS
  return DEFAULT_EQUALIZER_BANDS.map((defaultBand, index) => {
    const entry = source[index] || {}
    const gain = Number(entry.gain)
    return {
      frequency: defaultBand.frequency,
      gain: Number.isFinite(gain) ? Math.max(-15, Math.min(15, Math.round(gain * 10) / 10)) : 0
    }
  })
}

export function createBassEffectsRuntime(settingsStore) {
  const state = reactive({
    catalog: [],
    loading: false,
    error: '',
    fxAvailable: null,
    activeChannelId: null,
    handles: {},
    parameters: {},
    equalizerBands: clone(DEFAULT_EQUALIZER_BANDS),
    errors: {}
  })
  let loadCatalogPromise = null

  const loadCatalog = async () => {
    if (loadCatalogPromise) return loadCatalogPromise
    state.loading = true
    loadCatalogPromise = bassCall('bass_effect_catalog')
      .then((result) => {
        state.catalog = result?.effects || []
        return state.catalog
      })
      .catch((error) => {
        state.error = error?.message || String(error)
        return []
      })
      .finally(() => {
        state.loading = false
      })
    return loadCatalogPromise
  }

  const persist = (nextEffects) => settingsStore.updateEffects(nextEffects)

  const currentSettings = () => settingsStore.state.effects || emptySettings()

  const closeHandles = async () => {
    const ids = Object.values(state.handles).flatMap((value) => Array.isArray(value) ? value : [value]).filter(Boolean)
    state.handles = {}
    state.parameters = {}
    state.errors = {}
    await Promise.all(ids.map((effectId) => bassCall('bass_effect_close', { effectId }).catch(() => undefined)))
  }

  const ensureFx = async () => {
    const status = await bassCall('bass_status')
    if (status?.fxLoaded) {
      state.fxAvailable = true
      state.error = ''
      return true
    }
    try {
      const loaded = await bassCall('bass_load_fx_default')
      state.fxAvailable = Boolean(loaded?.fxLoaded)
      state.error = ''
      return state.fxAvailable
    } catch (error) {
      state.fxAvailable = false
      state.error = error?.message || String(error)
      return false
    }
  }

  const updateSavedEffect = (kind, patch) => {
    const next = clone(currentSettings()) || {}
    next[kind] = { ...(next[kind] || {}), ...clone(patch) }
    persist(next)
    return next[kind]
  }

  const addOne = async (channelId, descriptor, saved, priority) => {
    const result = await bassCall('bass_add_effect', {
      channelId,
      kind: descriptor.kind,
      priority
    })
    const effectId = result.effectId
    if (descriptor.kind === EQUALIZER_KIND) {
      const bands = normalizeEqualizerBands(saved?.bands)
      for (const [index, band] of bands.entries()) {
        await bassCall('bass_effect_set_parameters', {
          effectId,
          parameters: {
            lBand: index,
            fBandwidth: 1,
            fQ: 0,
            fCenter: band.frequency,
            fGain: band.gain,
            lChannel: -1
          }
        })
      }
      state.handles[descriptor.kind] = effectId
      state.equalizerBands = bands
      updateSavedEffect(descriptor.kind, { bands })
      return
    }
    const native = await bassCall('bass_effect_get_parameters', { effectId })
    const parameters = mergeParameters(native?.parameters || {}, saved?.parameters)
    if (saved?.parameters) {
      await bassCall('bass_effect_set_parameters', { effectId, parameters })
    }
    state.handles[descriptor.kind] = effectId
    state.parameters[descriptor.kind] = parameters
    updateSavedEffect(descriptor.kind, { parameters })
  }

  const applyToChannel = async (channelId) => {
    await loadCatalog()
    await closeHandles()
    state.error = ''
    state.activeChannelId = channelId || null
    if (!channelId) return

    const enabled = state.catalog.filter((descriptor) => {
      if (descriptor.kind === 'dx8.parameq') return false
      if (descriptor.kind === EQUALIZER_KIND) {
        return currentSettings()[EQUALIZER_KIND]?.enabled || currentSettings()['dx8.parameq']?.enabled
      }
      return currentSettings()[descriptor.kind]?.enabled
    })
    if (!enabled.length) return
    if (!(await ensureFx())) return

    for (const [priority, descriptor] of enabled.entries()) {
      try {
        await addOne(channelId, descriptor, currentSettings()[descriptor.kind], priority)
      } catch (error) {
        state.errors[descriptor.kind] = error?.message || String(error)
      }
    }
  }

  const setEffect = async (kind, patch) => {
    const saved = updateSavedEffect(kind, patch)
    const effectId = state.handles[kind]
    if (!effectId) {
      if (saved.enabled && state.activeChannelId) await applyToChannel(state.activeChannelId)
      return
    }
    try {
      if (patch.parameters) {
        const parameters = mergeParameters(state.parameters[kind], patch.parameters)
        await bassCall('bass_effect_set_parameters', { effectId, parameters })
        state.parameters[kind] = parameters
        updateSavedEffect(kind, { parameters })
      }
    } catch (error) {
      state.errors[kind] = error?.message || String(error)
    }
  }

  const setEqualizerBand = async (index, gain) => {
    const current = normalizeEqualizerBands(state.equalizerBands)
    if (!current[index]) return
    const nextGain = Math.max(-15, Math.min(15, Number(gain) || 0))
    current[index] = { ...current[index], gain: Math.round(nextGain * 10) / 10 }
    state.equalizerBands = current
    updateSavedEffect(EQUALIZER_KIND, { bands: current, enabled: true })
    const effectId = state.handles[EQUALIZER_KIND]
    if (!effectId) {
      if (state.activeChannelId) await applyToChannel(state.activeChannelId)
      return
    }
    try {
      await bassCall('bass_effect_set_parameters', {
        effectId,
        parameters: {
          lBand: index,
          fBandwidth: 1,
          fQ: 0,
          fCenter: current[index].frequency,
          fGain: current[index].gain,
          lChannel: -1
        }
      })
    } catch (error) {
      state.errors[EQUALIZER_KIND] = error?.message || String(error)
    }
  }

  const setEqualizerEnabled = async (enabled) => {
    updateSavedEffect(EQUALIZER_KIND, { enabled: Boolean(enabled) })
    updateSavedEffect('dx8.parameq', { enabled: false })
    if (state.activeChannelId) await applyToChannel(state.activeChannelId)
  }

  const resetEqualizer = async () => {
    const defaults = normalizeEqualizerBands(DEFAULT_EQUALIZER_BANDS)
    state.equalizerBands = defaults
    updateSavedEffect(EQUALIZER_KIND, { bands: defaults })
    const effectId = state.handles[EQUALIZER_KIND]
    if (!effectId) return
    for (const [index, band] of defaults.entries()) {
      await bassCall('bass_effect_set_parameters', {
        effectId,
        parameters: {
          lBand: index,
          fBandwidth: 1,
          fQ: 0,
          fCenter: band.frequency,
          fGain: 0,
          lChannel: -1
        }
      })
    }
  }

  const setEnabled = async (kind, enabled) => {
    const next = updateSavedEffect(kind, { enabled: Boolean(enabled) })
    if (state.activeChannelId) await applyToChannel(state.activeChannelId)
    return next
  }

  const resetEffect = async (kind) => {
    const next = clone(currentSettings()) || {}
    const effectId = state.handles[kind]
    if (!effectId) {
      if (next[kind]) delete next[kind].parameters
      persist(next)
      return
    }
    try {
      await bassCall('bass_effect_reset', { effectId })
      const result = await bassCall('bass_effect_get_parameters', { effectId })
      next[kind] = { ...(next[kind] || {}), parameters: result?.parameters || {} }
      state.parameters[kind] = result?.parameters || {}
      persist(next)
    } catch (error) {
      state.error = `${kind}: ${error?.message || String(error)}`
      state.errors[kind] = error?.message || String(error)
    }
  }

  const setVolume = async (channelId, volume, muted = false) => {
    if (!channelId) return
    await bassCall('bass_channel_set_volume', {
      channelId,
      volume: muted ? 0 : Math.max(0, Math.min(100, Number(volume) || 0)) / 100
    })
  }

  return {
    state,
    loadCatalog,
    applyToChannel,
    closeHandles,
    setEffect,
    setEqualizerBand,
    setEqualizerEnabled,
    resetEqualizer,
    setEnabled,
    resetEffect,
    setVolume
  }
}
