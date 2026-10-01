<template>
  <div ref="root">
    <component :is="tag" ref="content" :style="contentStyle">
      <template v-for="(item, offset) in visible" :key="item.id ?? range.start + offset">
        <slot :item="item" :index="range.start + offset" />
      </template>
    </component>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount, onActivated, onDeactivated, nextTick } from 'vue'
import { virtualRange } from '@/utils/virtualRange.js'
const props = defineProps({ items: { type: Array, required: true }, itemHeight: { type: Number, default: 64 }, tag: { type: String, default: 'div' } })
const root = ref(null)
const content = ref(null)
const range = ref({ start: 0, end: 0 })
const visible = computed(() => props.items.slice(range.value.start, range.value.end))
const contentStyle = computed(() => ({
  margin: 0,
  paddingLeft: 0,
  listStyle: 'none',
  paddingTop: `${range.value.start * props.itemHeight}px`,
  paddingBottom: `${(props.items.length - range.value.end) * props.itemHeight}px`
}))
let scroller, observer, frame = 0
const update = () => {
  frame = 0
  if (!root.value || !content.value) return
  const bounds = content.value.getBoundingClientRect()
  const viewport = scroller === window ? { top: 0, height: window.innerHeight } : scroller.getBoundingClientRect()
  const top = scroller === root.value ? scroller.scrollTop : viewport.top - bounds.top
  range.value = virtualRange(props.items.length, props.itemHeight, top, scroller === root.value ? scroller.clientHeight : viewport.height)
}
const schedule = () => { if (!frame) frame = requestAnimationFrame(update) }
const stop = () => {
  scroller?.removeEventListener('scroll', schedule)
  window.removeEventListener('resize', schedule)
  observer?.disconnect()
  cancelAnimationFrame(frame)
  frame = 0
}
const start = () => {
  stop()
  scroller = root.value
  while (scroller && !/(auto|scroll)/.test(getComputedStyle(scroller).overflowY)) scroller = scroller.parentElement
  scroller ||= window
  scroller.addEventListener('scroll', schedule, { passive: true })
  window.addEventListener('resize', schedule)
  observer = new ResizeObserver(schedule)
  if (root.value) observer.observe(root.value)
  if (scroller !== window) observer.observe(scroller)
  schedule()
}
watch(() => [props.items, props.items.length, props.itemHeight], () => nextTick(schedule))
onMounted(start)
onActivated(start)
onDeactivated(stop)
onBeforeUnmount(stop)
</script>
