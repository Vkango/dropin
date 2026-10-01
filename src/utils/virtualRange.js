export function virtualRange(count, height, top, viewportHeight, overscan = 6) {
  if (!count || top >= count * height || top + viewportHeight <= 0) return { start: 0, end: 0 }
  return {
    start: Math.max(0, Math.floor(top / height) - overscan),
    end: Math.min(count, Math.ceil((top + viewportHeight) / height) + overscan)
  }
}
