import { reactive } from 'vue'
import { bassCall } from './bassApi.js'

const clamp = (value, min, max) => Math.max(min, Math.min(max, Number(value)))

export function createBassPlaybackRuntime(settingsStore) {
  const state = reactive({
    activeChannelId: null,
    baseFrequency: 44100,
    tempoReady: false,
    reverseReady: false,
    error: ''
  })

  const playbackSettings = () => settingsStore.state.playback || {
    speed: 0,
    frequencyRatio: 1,
    pan: 0,
    reverse: false
  }

  const persist = (patch) => {
    settingsStore.updatePlayback({ ...playbackSettings(), ...patch })
  }

  const ensureFx = async () => {
    const status = await bassCall('bass_status')
    if (status?.fxLoaded) return true
    const loaded = await bassCall('bass_load_fx_default')
    return Boolean(loaded?.fxLoaded)
  }

  const prepareChannel = async (channelId) => {
    state.activeChannelId = channelId || null
    state.tempoReady = false
    state.reverseReady = false
    state.error = ''
    if (!channelId) return false

    try {
      const snapshot = await bassCall('bass_channel_snapshot', { channelId })
      state.baseFrequency = Number(snapshot?.frequency) > 0 ? Number(snapshot.frequency) : 44100
      const fxReady = await ensureFx()
      if (fxReady) {
        state.tempoReady = Boolean(await bassCall('bass_tempo_get', { channelId }).catch(() => null))
        state.reverseReady = state.tempoReady
          && Boolean(await bassCall('bass_reverse_get', { channelId }).catch(() => null))
      }
      await applySettings(channelId)
      return true
    } catch (error) {
      state.error = error?.message || String(error)
      // Frequency and pan remain available on a plain channel if TempoChannel cannot be created.
      await setPlainChannelSettings(channelId).catch(() => undefined)
      return false
    }
  }

  const setPlainChannelSettings = async (channelId) => {
    const playback = playbackSettings()
    await bassCall('bass_channel_set_frequency', {
      channelId,
      frequency: state.baseFrequency * clamp(playback.frequencyRatio, 0.5, 2)
    })
    await bassCall('bass_channel_set_pan', {
      channelId,
      pan: clamp(playback.pan, -1, 1)
    })
  }

  const applySettings = async (channelId = state.activeChannelId) => {
    if (!channelId) return
    const playback = playbackSettings()
    if (state.tempoReady) {
      await bassCall('bass_tempo_set', { channelId, field: 'tempo', value: clamp(playback.speed, -95, 1000) })
      await bassCall('bass_tempo_set', {
        channelId,
        field: 'frequency',
        value: state.baseFrequency * clamp(playback.frequencyRatio, 0.5, 2)
      })
      if (state.reverseReady) {
        await bassCall('bass_reverse_set', {
          channelId,
          direction: playback.reverse ? -1 : 1
        })
      }
    } else {
      await bassCall('bass_channel_set_frequency', {
        channelId,
        frequency: state.baseFrequency * clamp(playback.frequencyRatio, 0.5, 2)
      })
    }
    await bassCall('bass_channel_set_pan', { channelId, pan: clamp(playback.pan, -1, 1) })
  }

  const update = async (patch) => {
    persist(patch)
    if (state.activeChannelId) {
      try {
        await applySettings()
      } catch (error) {
        state.error = error?.message || String(error)
      }
    }
  }

  const close = () => {
    state.activeChannelId = null
    state.tempoReady = false
    state.reverseReady = false
    state.error = ''
  }

  return { state, prepareChannel, applySettings, update, close }
}
