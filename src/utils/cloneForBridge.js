import { toRaw } from 'vue'

// Bridge payloads must be JSON-compatible plain data, not Vue reactive proxies.
export function cloneForBridge(value) {
  const serialized = JSON.stringify(toRaw(value))
  return serialized === undefined ? null : JSON.parse(serialized)
}
