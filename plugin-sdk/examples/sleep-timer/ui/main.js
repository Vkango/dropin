import { createDropinPluginClient } from '/sdk/dropin-sdk.js'

const dropin = createDropinPluginClient()
const remaining = document.querySelector('#remaining')
const stateText = document.querySelector('#state')
const progress = document.querySelector('#progress')
const minutes = document.querySelector('#minutes')
const message = document.querySelector('#message')
const start = document.querySelector('#start')
const cancel = document.querySelector('#cancel')

const backend = (method, args = {}) => dropin.backend.call(method, {
  ...args,
  nowMs: Date.now()
})

const formatRemaining = (milliseconds) => {
  const totalSeconds = Math.max(0, Math.ceil(milliseconds / 1000))
  const mins = Math.floor(totalSeconds / 60)
  const secs = totalSeconds % 60
  return String(mins).padStart(2, '0') + ':' + String(secs).padStart(2, '0')
}

const render = (state) => {
  remaining.textContent = formatRemaining(state?.remainingMs ?? 0)
  stateText.textContent = state?.active ? '正在计时' : '未开始'
  start.disabled = Boolean(state?.active)
  cancel.disabled = !state?.active
  document.body.dataset.active = state?.active ? 'true' : 'false'
  const durationMs = Math.max(1, Number(state?.durationMs ?? 1))
  const remainingMs = Math.max(0, Number(state?.remainingMs ?? 0))
  const ratio = state?.active ? 1 - Math.min(1, remainingMs / durationMs) : 0
  progress.style.transform = 'scaleX(' + Math.max(0, Math.min(1, ratio)) + ')'
  if (state?.paused) {
    message.textContent = '已自动暂停。晚安。'
  }
}

const refresh = async (method = 'state', args = {}) => {
  try {
    const state = await backend(method, args)
    render(state)
    return state
  } catch (error) {
    message.textContent = error?.message || String(error)
    throw error
  }
}

start.addEventListener('click', async () => {
  const value = Number(minutes.value)
  const durationMs = Math.round(Math.max(0.1, value || 30) * 60_000)
  message.textContent = '定时器已启动。'
  await refresh('start', { durationMs })
})

cancel.addEventListener('click', async () => {
  message.textContent = '定时器已取消。'
  await refresh('cancel')
})

document.querySelectorAll('[data-minutes]').forEach((button) => {
  button.addEventListener('click', () => {
    minutes.value = button.dataset.minutes
  })
})

setInterval(() => {
  void refresh('tick')
}, 1000)

void refresh()
