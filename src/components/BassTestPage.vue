<template>
  <div class="bass-page">
    <div class="page-header">
      <div>
        <div class="eyebrow">BASS / BASS_FX</div>
        <h1>音频引擎测试</h1>
        <p>通过 Rust bridge 实测 DLL 加载、输出设备、文件/URL 播放和音频效果。</p>
      </div>
      <div class="header-actions">
        <button class="primary" @click="loadEngine">加载 BASS</button>
        <button @click="refresh">刷新状态</button>
      </div>
    </div>

    <div class="status-grid">
      <div class="status-card"><span>引擎</span><strong>{{ status.loaded ? '已加载' : '未加载' }}</strong></div>
      <div class="status-card"><span>BASS</span><strong>{{ version(status.bassVersion) }}</strong></div>
      <div class="status-card"><span>BASS_FX</span><strong>{{ status.fxLoaded ? version(status.fxVersion) : '未加载' }}</strong></div>
      <div class="status-card"><span>当前 Channel</span><strong>{{ channelId || '—' }}</strong></div>
    </div>

    <section class="panel">
      <div class="section-title"><h2>输出设备</h2><button @click="refreshDevices">重新枚举</button></div>
      <div class="form-grid">
        <label>设备<select v-model.number="init.device"><option :value="-1">默认设备</option><option v-for="device in devices" :key="device.index" :value="device.index">{{ device.index }} · {{ device.name }}</option></select></label>
        <label>后端<select v-model="init.backend"><option value="wasapi">WASAPI</option><option value="directSound">DirectSound</option></select></label>
        <label>采样率<input v-model.number="init.sampleRate" type="number" min="8000" step="100" /></label>
        <label>全局音量<input v-model.number="globalVolume" type="number" min="0" max="1" step="0.01" @change="setGlobalVolume" /></label>
      </div>
      <div class="checks"><label><input v-model="init.exclusive" type="checkbox" /> 独占</label><label><input v-model="init.mono" type="checkbox" /> Mono</label><label><input v-model="init.forceFrequency" type="checkbox" /> 强制采样率</label><label><input v-model="init.floatProcessing" type="checkbox" /> 浮点处理</label></div>
      <div class="button-row"><button class="primary" @click="initialize">初始化设备</button><button @click="engineCommand('bass_start')">启动</button><button @click="engineCommand('bass_pause')">暂停</button><button @click="engineCommand('bass_stop')">停止</button><button @click="engineCommand('bass_free')">释放输出</button></div>
      <pre v-if="outputInfo" class="small-json">{{ pretty(outputInfo) }}</pre>
    </section>

    <section class="panel">
      <div class="section-title"><h2>打开音频</h2></div>
      <div class="source-row"><input v-model="filePath" placeholder="音频文件完整路径，例如 E:\\Music\\demo.mp3" /><button @click="pickFile">选择文件</button><button class="primary" @click="loadFile">打开文件</button></div>
      <div class="source-row"><input v-model="url" placeholder="https://example.com/audio.mp3" /><button class="primary" @click="loadUrl">打开 URL</button></div>
      <div class="form-grid compact"><label>URL offset<input v-model.number="urlOptions.offset" type="number" min="0" /></label><label>URL flags<input v-model.number="urlOptions.flags" type="number" min="0" /></label><label><input v-model="urlOptions.float" type="checkbox" /> 浮点 URL</label></div>
      <div v-if="channelId" class="channel-controls">
        <select v-model.number="selectedChannel"><option v-for="id in channelIds" :key="id" :value="id">Channel {{ id }}</option></select>
        <button class="primary" @click="channelCommand('bass_channel_play', { restart: false })">播放</button><button @click="channelCommand('bass_channel_pause')">暂停</button><button @click="channelCommand('bass_channel_stop')">停止</button><button @click="channelCommand('bass_channel_close')">关闭</button>
        <label>音量<input v-model.number="channelVolume" type="range" min="0" max="1" step="0.01" @input="setChannelAttribute('volume', channelVolume)" /></label>
        <label>定位<input v-model.number="seekSeconds" type="number" min="0" step="0.1" /><button @click="channelCommand('bass_channel_seek', { seconds: seekSeconds })">跳转</button></label>
      </div>
      <pre v-if="snapshot" class="small-json">{{ pretty(snapshot) }}</pre>
    </section>

    <section class="panel">
      <div class="section-title"><h2>音频效果</h2></div>
      <div class="form-grid">
        <label>效果<select v-model="effectKind"><option v-for="item in effectKinds" :key="item.value" :value="item.value">{{ item.label }}</option></select></label>
        <label>优先级<input v-model.number="effectPriority" type="number" /></label>
      </div>
      <textarea v-model="effectJson" rows="5" spellcheck="false"></textarea>
      <div class="button-row"><button class="primary" @click="addEffect">添加效果</button><button @click="setEffect">写入参数</button><button @click="getEffect">读取参数</button><button @click="effectCommand('bass_effect_set_bypass', { bypass: true })">旁路</button><button @click="effectCommand('bass_effect_reset')">重置</button><button @click="effectCommand('bass_effect_close')">移除效果</button></div>
      <div class="button-row"><button @click="fxCommand('bass_channel_to_tempo')">转换为 Tempo</button><button @click="fxCommand('bass_channel_to_reverse')">转换为 Reverse</button><button @click="tempoSet('tempo')">应用速度</button><button @click="reverseSet">应用倒放方向</button></div>
      <div class="form-grid compact"><label>Tempo<input v-model.number="tempoValue" type="number" step="1" /></label><label>Pitch<input v-model.number="pitchValue" type="number" step="1" /></label><label>Reverse direction<input v-model.number="reverseDirection" type="number" step="1" /></label></div>
    </section>

    <section class="panel">
      <div class="section-title"><h2>插件、Raw API 和事件</h2></div>
      <div class="source-row"><input v-model="pluginPath" placeholder="插件 DLL 完整路径" /><button @click="loadPlugin">加载插件</button></div>
      <div class="button-row"><button @click="readRawCatalog">读取 raw 常量</button><button @click="addSync">监听结束事件</button><button @click="addDsp">注册 DSP（静音/透传）</button><button @click="removeCallback">移除回调</button></div>
      <pre class="event-log">{{ logs.join('\n') || '暂无事件' }}</pre>
      <pre v-if="rawCatalog" class="small-json">{{ pretty(rawCatalog) }}</pre>
    </section>

    <div v-if="error" class="error-box">{{ error }}</div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { bassCall } from '../services/bassApi.js'

const status = reactive({ loaded: false, fxLoaded: false })
const devices = ref([])
const outputInfo = ref(null)
const filePath = ref('')
const url = ref('')
const channelId = ref(null)
const selectedChannel = ref(null)
const channelIds = ref([])
const snapshot = ref(null)
const channelVolume = ref(1)
const seekSeconds = ref(0)
const globalVolume = ref(1)
const pluginPath = ref('')
const effectId = ref(null)
const effectPriority = ref(0)
const effectKind = ref('dx8.parameq')
const effectJson = ref('{\n  "fCenter": 1000,\n  "fBandwidth": 1,\n  "fGain": 3\n}')
const tempoValue = ref(0)
const pitchValue = ref(0)
const reverseDirection = ref(-1)
const syncId = ref(null)
const dspId = ref(null)
const rawCatalog = ref(null)
const logs = ref([])
const error = ref('')
let stopEvents = () => {}
let timer = null

const init = reactive({ device: -1, sampleRate: 44100, backend: 'wasapi', mono: false, exclusive: false, forceFrequency: false, floatProcessing: false })
const urlOptions = reactive({ offset: 0, flags: 0, float: false })
const effectKinds = [
  { value: 'dx8.parameq', label: 'DX8 ParamEQ' }, { value: 'dx8.compressor', label: 'DX8 Compressor' }, { value: 'dx8.reverb', label: 'DX8 Reverb' },
  { value: 'bassFx.freeverb', label: 'BASS_FX Freeverb' }, { value: 'bassFx.phaser', label: 'BASS_FX Phaser' }, { value: 'bassFx.echo2', label: 'BASS_FX Echo2' }, { value: 'bassFx.compressor2', label: 'BASS_FX Compressor2' }, { value: 'volume', label: 'Volume' }
]

const version = (value) => value ? `0x${Number(value).toString(16)}` : '—'
const pretty = (value) => JSON.stringify(value, null, 2)
const call = async (operation, args = {}) => {
  error.value = ''
  try { return await bassCall(operation, args) } catch (reason) { error.value = reason?.message || String(reason); throw reason }
}

const refresh = async () => {
  const next = await call('bass_status')
  Object.assign(status, next)
  channelIds.value = next.channels || []
  if (!selectedChannel.value && channelIds.value.length) selectedChannel.value = channelIds.value[0]
  await refreshDevices()
}
const loadEngine = async () => { await call('bass_load', { requireFx: true }); await refresh() }
const refreshDevices = async () => { if (status.loaded) devices.value = await call('bass_devices') }
const initialize = async () => { await call('bass_initialize', init); outputInfo.value = await call('bass_output_info'); await refresh() }
const engineCommand = async (operation) => { await call(operation); await refresh() }
const setGlobalVolume = async () => { await call('bass_set_global_volume', { volume: globalVolume.value }) }
const pickFile = async () => { const result = await call('bass_pick_file'); if (result.path) filePath.value = result.path }
const loadFile = async () => { const result = await call('bass_load_file', { path: filePath.value }); channelId.value = result.channelId; selectedChannel.value = result.channelId; await refresh() }
const loadUrl = async () => { const result = await call('bass_load_url', { url: url.value, options: { ...urlOptions } }); channelId.value = result.channelId; selectedChannel.value = result.channelId; await refresh() }
const channelCommand = async (operation, args = {}) => { const result = await call(operation, { channelId: selectedChannel.value, ...args }); await refreshSnapshot(); return result }
const setChannelAttribute = async (field, value) => { if (selectedChannel.value) await call('bass_channel_set_attribute', { channelId: selectedChannel.value, attribute: field === 'volume' ? 2 : 3, value }) }
const refreshSnapshot = async () => { if (selectedChannel.value) snapshot.value = await call('bass_channel_snapshot', { channelId: selectedChannel.value }) }
const addEffect = async () => { const result = await call('bass_add_effect', { channelId: selectedChannel.value, kind: effectKind.value, priority: effectPriority.value }); effectId.value = result.effectId }
const effectCommand = async (operation, args = {}) => { if (effectId.value) await call(operation, { effectId: effectId.value, ...args }) }
const setEffect = async () => { await effectCommand('bass_effect_set_parameters', { parameters: JSON.parse(effectJson.value) }) }
const getEffect = async () => { const result = await call('bass_effect_get_parameters', { effectId: effectId.value }); effectJson.value = pretty(result.parameters) }
const fxCommand = async (operation) => { await channelCommand(operation, {}) }
const tempoSet = async (field) => { await call('bass_tempo_set', { channelId: selectedChannel.value, field, value: field === 'tempo' ? tempoValue.value : pitchValue.value }) }
const reverseSet = async () => { await call('bass_reverse_set', { channelId: selectedChannel.value, direction: reverseDirection.value }) }
const loadPlugin = async () => { await call('bass_load_plugin', { path: pluginPath.value }) }
const readRawCatalog = async () => { rawCatalog.value = await call('bass_raw_catalog') }
const addSync = async () => { const result = await call('bass_channel_add_sync', { channelId: selectedChannel.value, kind: 'end' }); syncId.value = result.registrationId }
const addDsp = async () => { const result = await call('bass_channel_add_dsp', { channelId: selectedChannel.value, mode: 'mute', priority: 0 }); dspId.value = result.registrationId }
const removeCallback = async () => { if (syncId.value) await call('bass_channel_remove_sync', { registrationId: syncId.value }); if (dspId.value) await call('bass_channel_remove_dsp', { registrationId: dspId.value }); syncId.value = null; dspId.value = null }

onMounted(async () => {
  const unlisteners = await Promise.all(['bass/download', 'bass/sync', 'bass/dsp', 'bass/channel-state'].map((name) => listen(name, (event) => { logs.value.unshift(`${name}: ${pretty(event.payload)}`); logs.value = logs.value.slice(0, 40) })))
  stopEvents = () => unlisteners.forEach((stop) => stop())
  await refresh().catch(() => {})
  timer = window.setInterval(() => refreshSnapshot().catch(() => {}), 500)
})
onBeforeUnmount(() => { stopEvents(); if (timer) window.clearInterval(timer) })
</script>

<style scoped>
.bass-page { width: 100%; padding: 28px 32px 160px; color: rgb(var(--text-color)); }
.page-header, .section-title, .header-actions, .button-row, .source-row, .channel-controls, .checks { display: flex; align-items: center; gap: 10px; }
.page-header { justify-content: space-between; margin-bottom: 24px; gap: 20px; }
.eyebrow { color: rgb(var(--primary-color)); font-size: 11px; letter-spacing: .12em; font-weight: 700; }
h1 { margin: 5px 0; font-size: 30px; } h2 { margin: 0; font-size: 17px; } p { opacity: .65; font-size: 13px; }
button, input, select, textarea { border: 1px solid rgba(var(--outline-color), .24); border-radius: 8px; background: rgba(var(--surface-color), .5); color: inherit; padding: 9px 11px; font: inherit; }
button { cursor: pointer; } button:hover { border-color: rgb(var(--primary-color)); } button.primary { background: rgb(var(--primary-color)); color: #10212b; border-color: transparent; }
.status-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-bottom: 14px; }.status-card, .panel { border: 1px solid rgba(var(--outline-color), .16); background: rgba(var(--surface-color), .42); border-radius: 14px; }.status-card { padding: 14px; display: grid; gap: 6px; }.status-card span { font-size: 11px; opacity: .6; }.status-card strong { font-size: 15px; }
.panel { padding: 18px; margin-top: 14px; }.section-title { justify-content: space-between; margin-bottom: 14px; }.form-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }.form-grid.compact { grid-template-columns: repeat(3, minmax(0, 1fr)); margin-top: 12px; }.form-grid label, .channel-controls label { display: grid; gap: 6px; font-size: 12px; opacity: .85; }.form-grid input, .form-grid select { width: 100%; }.checks { margin: 14px 0; flex-wrap: wrap; font-size: 12px; }.checks label { display: flex; align-items: center; gap: 5px; }.button-row { flex-wrap: wrap; margin-top: 14px; }.source-row { margin-top: 10px; }.source-row input:not(.file-input) { flex: 1; min-width: 180px; }.file-input { width: 170px; padding: 6px; }.channel-controls { margin-top: 14px; flex-wrap: wrap; }.channel-controls label { display: flex; align-items: center; }.channel-controls input[type=range] { width: 120px; padding: 0; }.small-json, .event-log { margin-top: 14px; padding: 12px; border-radius: 8px; background: rgba(0, 0, 0, .18); white-space: pre-wrap; word-break: break-word; font: 12px/1.5 ui-monospace, monospace; max-height: 220px; overflow: auto; }.event-log { max-height: 180px; }.error-box { margin-top: 14px; padding: 12px; border-radius: 8px; background: rgba(220, 60, 60, .15); color: #ff9c9c; font-size: 13px; }
textarea { width: 100%; margin-top: 14px; font: 12px/1.5 ui-monospace, monospace; }.header-actions { flex-shrink: 0; }
@media (max-width: 900px) { .status-grid, .form-grid, .form-grid.compact { grid-template-columns: repeat(2, minmax(0, 1fr)); } .page-header { align-items: flex-start; flex-direction: column; } }
</style>
