import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export function bassCall(operation, args = {}) {
  return invoke('bass_call', { operation, args })
}

export async function listenToBassEvents(handler) {
  const eventNames = ['bass/download', 'bass/sync', 'bass/dsp', 'bass/channel-state']
  const unlisten = await Promise.all(eventNames.map((eventName) =>
    listen(eventName, (event) => handler(eventName, event.payload))
  ))
  return () => unlisten.forEach((stop) => stop())
}
