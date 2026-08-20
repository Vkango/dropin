<template>
  <Teleport to="#decorum-player-mount">
    <div v-if="!isFullscreen" class="title-bar-player">
      <PlayerControls :current-song="currentSong" :is-playing="isPlaying" :current-time="currentTime"
        :current-time-ms="currentTimeMs" :total-time="totalTime" :progress="progress" :lyrics="lyrics"
        :lyrics-loading="lyricsLoading" @toggle-play="$emit('toggle-play')" @previous="$emit('previous')"
        @next="$emit('next')" @progress-change="$emit('progress-change', $event)"
        @progress-commit="$emit('progress-commit', $event)"
        @repeat="$emit('repeat')" @queue="$emit('queue')" @expand-player="$emit('expand-player')" />
    </div>
  </Teleport>
</template>

<script setup>
import { onBeforeUnmount, onMounted, watch } from 'vue'
import PlayerControls from './PlayerControls.vue'

const props = defineProps({
  currentSong: {
    type: Object,
    required: true
  },
  isPlaying: {
    type: Boolean,
    default: false
  },
  currentTime: {
    type: String,
    default: '00:00'
  },
  currentTimeMs: {
    type: Number,
    default: 0
  },
  totalTime: {
    type: String,
    default: '00:00'
  },
  progress: {
    type: Number,
    default: 0
  },
  lyrics: {
    type: Object,
    default: null
  },
  lyricsLoading: {
    type: Boolean,
    default: false
  },
  isScrolled: {
    type: Boolean,
    default: false
  },
  isFullscreen: {
    type: Boolean,
    default: false
  }
})

defineEmits([
  'toggle-play',
  'previous',
  'next',
  'progress-change',
  'repeat',
  'queue',
  'expand-player',
  'progress-commit'
])

const syncTitlebarState = () => {
  const titlebar = document.getElementById('decorum-titlebar')
  if (!titlebar) return

  titlebar.classList.toggle('is-fullscreen', props.isFullscreen)
  titlebar.classList.toggle('is-scrolled', props.isScrolled)
}

onMounted(syncTitlebarState)
watch(() => [props.isFullscreen, props.isScrolled], syncTitlebarState)

onBeforeUnmount(() => {
  const titlebar = document.getElementById('decorum-titlebar')
  titlebar?.classList.remove('is-fullscreen', 'is-scrolled')
})
</script>

<style scoped>
.title-bar-player {
  width: 100%;
  height: 60px;
  min-width: 0;
  pointer-events: auto;
  font-family: MiSans, 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}
</style>
