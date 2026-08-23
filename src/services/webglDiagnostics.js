import { reactive } from 'vue'

export const webglDiagnostics = reactive({
  status: 'not-tested',
  renderer: '',
  vendor: '',
  version: '',
  shadingLanguageVersion: '',
  context: 'webgl',
  error: '',
  contextLost: false,
  lastUpdated: 0
})

export function updateWebglDiagnostics(patch = {}) {
  Object.assign(webglDiagnostics, patch, { lastUpdated: Date.now() })
}

export function resetWebglDiagnostics() {
  updateWebglDiagnostics({
    status: 'not-tested',
    renderer: '',
    vendor: '',
    version: '',
    shadingLanguageVersion: '',
    context: 'webgl',
    error: '',
    contextLost: false
  })
}

