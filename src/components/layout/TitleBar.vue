<template>
  <Teleport to="#decorum-player-mount">
    <div v-if="!isFullscreen" class="title-bar-player">
      <!-- 窄窗口下的汉堡菜单：展开/收起抽屉式侧边栏 -->
      <button v-if="isDrawer" type="button" class="menu-toggle" :aria-label="t('sidebar.menuToggle')"
        :aria-expanded="isDrawerOpen" @click="$emit('menu')">
        <Menu :size="18" :stroke-width="1.8" />
      </button>
      <PlayerControls class="title-bar-controls" :current-song="currentSong" :is-playing="isPlaying"
        :current-time="currentTime" :current-time-ms="currentTimeMs" :total-time="totalTime"
        :progress="progress" :lyrics="lyrics" :lyrics-loading="lyricsLoading"
        @toggle-play="$emit('toggle-play')" @previous="$emit('previous')"
        @next="$emit('next')" @progress-change="$emit('progress-change', $event)"
        @progress-commit="$emit('progress-commit', $event)"
        :playback-mode="playbackMode" :list-loop="listLoop"
        :volume="volume" :muted="muted"
        @playback-mode-change="$emit('playback-mode-change', $event)"
        @list-loop-change="$emit('list-loop-change', $event)"
        @volume-change="$emit('volume-change', $event)" @mute-change="$emit('mute-change', $event)"
        @queue="$emit('queue')" @expand-player="$emit('expand-player')" />
    </div>
  </Teleport>
</template>

<script setup>
import { onBeforeUnmount, onMounted, watch } from 'vue'
import { Menu } from '@lucide/vue'
import PlayerControls from '@/components/player/PlayerControls.vue'
import { useI18n } from '@/i18n/index.js'

const { t } = useI18n()

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
  },
  playbackMode: {
    type: String,
    default: 'sequential'
  },
  listLoop: {
    type: Boolean,
    default: false
  },
  volume: {
    type: Number,
    default: 75
  },
  muted: {
    type: Boolean,
    default: false
  },
  isDrawer: {
    type: Boolean,
    default: false
  },
  isDrawerOpen: {
    type: Boolean,
    default: false
  }
})

defineEmits([
  'toggle-play',
  'previous',
  'next',
  'progress-change',
  'playback-mode-change',
  'list-loop-change',
  'volume-change',
  'mute-change',
  'queue',
  'expand-player',
  'progress-commit',
  'menu'
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
  display: flex;
  align-items: stretch;
  width: 100%;
  height: 60px;
  min-width: 0;
  pointer-events: auto;
  font-family: MiSans, 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.title-bar-controls {
  flex: 1 1 auto;
  min-width: 0;
}

.menu-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 48px;
  width: 48px;
  height: 48px;
  margin: 6px 4px 6px 10px;
  padding: 0;
  border: none;
  border-radius: 50%;
  color: rgb(var(--text-color));
  background: transparent;
  cursor: pointer;
}

.menu-toggle:hover {
  color: rgb(var(--primary-color));
  background: rgba(var(--primary-color), 0.14);
}
</style>
