import { installGlobalFrameRateScheduler } from './utils/frameRateScheduler.js'
import { frameRateLimits, loadAppSettings } from './stores/appSettingsStore.js'

installGlobalFrameRateScheduler(frameRateLimits.default)

async function bootstrap() {
  await loadAppSettings()
  const [{ createApp }, { default: App }] = await Promise.all([
    import('vue'),
    import('./App.vue')
  ])

  createApp(App).mount('#app')
}

bootstrap()
