<template>
  <section class="plugin-host" v-if="plugin">
    <div v-if="!plugin.enabled || plugin.faulted" class="plugin-blocked">{{ plugin.faulted ? 'Plugin failed.' : 'Plugin\
      disabled.' }}</div>
    <iframe v-else ref="frame" class="plugin-frame"
      sandbox="allow-scripts allow-same-origin allow-top-navigation-by-user-activation" referrerpolicy="no-referrer"
      :title="plugin.name" :src="uiUrl" @load="handleLoad" />
    <p v-if="error" class="plugin-host-error">{{ error }}</p>
  </section>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { pluginApi } from '../services/pluginApi.js'
import { cloneForBridge } from '../utils/cloneForBridge.js'

const props = defineProps({
  plugin: { type: Object, required: true },
  hostState: { type: Object, default: () => ({}) }
})

const frame = ref(null)
const uiUrl = ref('')
const ready = ref(false)
const error = ref('')
const sendTheme = () => {
  if (!frame.value?.contentWindow || !props.hostState?.theme) return
  try {
    frame.value.contentWindow.postMessage({
      type: 'dropin:theme',
      theme: cloneForBridge(props.hostState.theme)
    }, '*')
  } catch (cause) {
    error.value = `Theme sync failed: ${cause?.message || String(cause)}`
  }
}
const handleLoad = () => {
  ready.value = true
  sendTheme()
}
const request = async (method, args = {}) => {
  const nextArgs = method === 'player.play' || method === 'player.pause'
    ? { ...args, channelId: props.hostState.channelId }
    : args
  return pluginApi.call(props.plugin.id, method, nextArgs)
}

const handleMessage = async (event) => {
  if (!frame.value?.contentWindow || event.source !== frame.value.contentWindow) return
  const message = event.data
  if (!message || message.type !== 'dropin:request' || typeof message.requestId !== 'string' || typeof message.method !== 'string') return
  try {
    const result = await request(message.method, message.args || {})
    event.source.postMessage({ type: 'dropin:response', requestId: message.requestId, ok: true, result }, '*')
  } catch (cause) {
    event.source.postMessage({ type: 'dropin:response', requestId: message.requestId, ok: false, error: cause?.message || String(cause) }, '*')
  }
}

const load = async () => {
  if (!props.plugin?.enabled || props.plugin?.faulted) return
  try {
    uiUrl.value = await pluginApi.uiUrl(props.plugin.id)
  } catch (cause) {
    error.value = cause?.message || String(cause)
  }
}

onMounted(() => {
  window.addEventListener('message', handleMessage)
  load()
})
watch(() => props.plugin?.id, load)
watch(
  () => JSON.stringify(props.hostState?.theme ?? null),
  sendTheme
)
onBeforeUnmount(() => {
  window.removeEventListener('message', handleMessage)
})
</script>

<style scoped>
.plugin-host {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-height: 0;
  width: 100%;
  overflow: hidden;
  background: transparent;
}

.plugin-frame {
  display: block;
  flex: 1 1 auto;
  width: 100%;
  height: 100%;
  min-height: 0;
  border: 0;
  background: transparent;
}

.plugin-blocked {
  display: grid;
  min-height: 220px;
  place-items: center;
  color: rgba(var(--text-color), .55);
  font-size: 13px;
}

.plugin-host-error {
  margin: 0;
  padding: 10px;
  color: #f44336;
  font-size: 12px;
}
</style>
