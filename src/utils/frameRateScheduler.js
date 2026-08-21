const DEFAULT_FRAME_RATE = 60
const MIN_FRAME_RATE = 15
const MAX_FRAME_RATE = 120
const FRAME_EPSILON_MS = 0.5

const nativeRequestAnimationFrame = window.requestAnimationFrame.bind(window)
const nativeCancelAnimationFrame = window.cancelAnimationFrame.bind(window)

let installed = false
let frameRate = DEFAULT_FRAME_RATE
let nextRequestId = 1
let limitedFrameId = null
let lastLimitedFrameAt = 0
const records = new Map()
const limitedQueue = new Set()

const normalizeFrameRate = (value) => {
  if (value === null) return null
  const numericValue = Number(value)
  if (!Number.isFinite(numericValue)) return DEFAULT_FRAME_RATE
  return Math.round(Math.max(MIN_FRAME_RATE, Math.min(MAX_FRAME_RATE, numericValue)))
}

const limitedFrameInterval = () => 1000 / (frameRate || DEFAULT_FRAME_RATE)

const flushLimitedCallbacks = (timestamp) => {
  const dueIds = [...limitedQueue]
  limitedQueue.clear()

  dueIds.forEach((id) => {
    const record = records.get(id)
    if (!record || record.mode !== 'limited') return
    records.delete(id)
    record.callback(timestamp)
  })
}

const scheduleLimitedFrame = () => {
  if (limitedFrameId !== null || !limitedQueue.size) return
  limitedFrameId = nativeRequestAnimationFrame(handleLimitedFrame)
}

function handleLimitedFrame(timestamp) {
  limitedFrameId = null
  if (!limitedQueue.size) return

  if (frameRate !== null) {
    const elapsed = timestamp - lastLimitedFrameAt
    if (lastLimitedFrameAt > 0 && elapsed + FRAME_EPSILON_MS < limitedFrameInterval()) {
      scheduleLimitedFrame()
      return
    }
    lastLimitedFrameAt = timestamp
  }

  flushLimitedCallbacks(timestamp)
  if (limitedQueue.size) scheduleLimitedFrame()
}

const cappedRequestAnimationFrame = (callback) => {
  const id = nextRequestId++

  if (frameRate === null) {
    const nativeId = nativeRequestAnimationFrame((timestamp) => {
      const record = records.get(id)
      if (!record) return
      records.delete(id)
      record.callback(timestamp)
    })
    records.set(id, { callback, nativeId, mode: 'native' })
    return id
  }

  records.set(id, { callback, mode: 'limited' })
  limitedQueue.add(id)
  scheduleLimitedFrame()
  return id
}

const cappedCancelAnimationFrame = (id) => {
  const record = records.get(id)
  if (!record) return
  if (record.mode === 'native') nativeCancelAnimationFrame(record.nativeId)
  limitedQueue.delete(id)
  records.delete(id)
}

export function installGlobalFrameRateScheduler(initialFrameRate = DEFAULT_FRAME_RATE) {
  setGlobalAnimationFrameRate(initialFrameRate)
  if (installed) return

  window.requestAnimationFrame = cappedRequestAnimationFrame
  window.cancelAnimationFrame = cappedCancelAnimationFrame
  installed = true
}

export function setGlobalAnimationFrameRate(nextFrameRate) {
  const normalizedFrameRate = normalizeFrameRate(nextFrameRate)
  const previousFrameRate = frameRate
  frameRate = normalizedFrameRate
  lastLimitedFrameAt = 0

  if (previousFrameRate === null && frameRate !== null) {
    records.forEach((record, id) => {
      if (record.mode !== 'native') return
      nativeCancelAnimationFrame(record.nativeId)
      record.mode = 'limited'
      limitedQueue.add(id)
    })
  }

  if (frameRate === null && limitedQueue.size) scheduleLimitedFrame()
  if (frameRate !== null && limitedQueue.size) scheduleLimitedFrame()
}

export function getGlobalAnimationFrameRate() {
  return frameRate
}

export const frameRateLimits = {
  default: DEFAULT_FRAME_RATE,
  min: MIN_FRAME_RATE,
  max: MAX_FRAME_RATE
}
