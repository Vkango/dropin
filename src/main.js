import { installGlobalFrameRateScheduler } from './utils/frameRateScheduler.js'
import { frameRateLimits, loadAppSettings } from './stores/appSettingsStore.js'

installGlobalFrameRateScheduler(frameRateLimits.default)

async function bootstrap() {
  const [settings, i18n] = await Promise.all([
    loadAppSettings(),
    import('./stores/i18nStore.js')
  ])

  await i18n.activateLocale(settings?.language || 'system')

  const [{ createApp }, { default: App }] = await Promise.all([
    import('vue'),
    import('./App.vue')
  ])

  createApp(App).mount('#app')
}

bootstrap()
