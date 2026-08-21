import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const isWindows = /Windows/i.test(navigator.userAgent || '')

export function smtcCall(operation, args = {}) {
  if (!isWindows) return Promise.resolve(null)
  return invoke('smtc_call', { operation, args })
}

export const smtcApi = {
  setMediaInfo: ({ title = '', artist = '', album = '', thumbnailPath = null } = {}) =>
    smtcCall('smtc_set_media_info', { title, artist, album, thumbnailPath }),
  setPlaybackStatus: (playing) => smtcCall('smtc_set_playback_status', { playing: Boolean(playing) }),
  setTimeline: (positionMs = 0, durationMs = 0) => smtcCall('smtc_set_timeline', {
    positionMs: Math.max(0, Math.round(Number(positionMs) || 0)),
    durationMs: Math.max(0, Math.round(Number(durationMs) || 0))
  }),
  close: () => smtcCall('smtc_close')
}

export async function listenToSmtcEvents(handler) {
  if (!isWindows) return () => {}
  return listen('smtc/event', (event) => handler(event.payload))
}
