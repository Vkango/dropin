<template>
    <AnimatePresence>
        <MotionDiv v-if="isFullscreen" class="player-backdrop" :initial="{ opacity: 1 }" :animate="{ opacity: 1 }"
            :exit="{ opacity: 1 }" :transition="backdropTransition">
            <MotionDiv class="player-backdrop-shade" :initial="{ opacity: 0, backdropFilter: 'blur(0px)' }"
                :animate="{ opacity: 0.7, backdropFilter: 'blur(14px)' }"
                :exit="{ opacity: 0, backdropFilter: 'blur(0px)' }" :transition="backdropTransition" />
        </MotionDiv>
    </AnimatePresence>

    <div class="player-surface">
        <MotionDiv class="surface-fill" :initial="false" :animate="surfaceMotion" :transition="surfaceTransition">
            <AnimatePresence :initial="false" mode="sync">
                <MotionDiv v-if="isFullscreen" key="full-player" class="player-layer full-layer"
                :initial="fullLayerInitial"
                :animate="fullLayerOpen"
                :exit="fullLayerClosed"
                :transition="contentTransition">
                <FullscreenPlayer :is-visible="isFullscreen" :current-song="currentSong" :is-playing="isPlaying"
                    :current-time="currentTime" :total-time="totalTime" :progress="progress"
                    @close="$emit('close')" @toggle-play="$emit('toggle-play')" @previous="$emit('previous')"
                    @next="$emit('next')" @progress-change="$emit('progress-change', $event)"
                    @volume-change="$emit('volume-change', $event)" @shuffle="$emit('shuffle')"
                    @repeat="$emit('repeat')" @add-to-playlist="$emit('add-to-playlist')"
                    @queue="$emit('queue')" />
                </MotionDiv>
            </AnimatePresence>
        </MotionDiv>

        <AnimatePresence :initial="false" mode="sync">
            <MotionDiv v-if="!isFullscreen" :ref="setMiniLayerRef" key="mini-player" class="mini-layer"
                :initial="{ opacity: 0 }" :animate="{ opacity: 1 }" :exit="{ opacity: 0 }"
                :transition="contentTransition">
                <PlayerControls :current-song="currentSong" :is-playing="isPlaying" :current-time="currentTime"
                    :total-time="totalTime" :progress="progress" @toggle-play="$emit('toggle-play')"
                    @previous="$emit('previous')" @next="$emit('next')" @progress-change="$emit('progress-change', $event)"
                    @repeat="$emit('repeat')" @menu="$emit('menu')" @add="$emit('add')"
                    @expand-player="$emit('expand-player')" />
            </MotionDiv>
        </AnimatePresence>
    </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import PlayerControls from './PlayerControls.vue'
import FullscreenPlayer from './FullscreenPlayer.vue'
import { APPLE_SPRING, INSTANT_MOTION, SOFT_SPRING } from '../utils/motion.js'

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
    totalTime: {
        type: String,
        default: '00:00'
    },
    progress: {
        type: Number,
        default: 0
    },
    isFullscreen: {
        type: Boolean,
        default: false
    }
})

defineEmits([
    'close',
    'toggle-play',
    'previous',
    'next',
    'progress-change',
    'volume-change',
    'shuffle',
    'repeat',
    'add-to-playlist',
    'queue',
    'menu',
    'add',
    'expand-player'
])

const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const miniLayerRef = ref(null)
const miniSize = ref({ width: 560, height: 90 })
const viewportSize = ref({
    width: typeof window === 'undefined' ? 1280 : window.innerWidth,
    height: typeof window === 'undefined' ? 720 : window.innerHeight
})
let miniResizeObserver

const setMiniLayerRef = (value) => {
    miniLayerRef.value = value?.$el ?? value
}

const updateViewportSize = () => {
    viewportSize.value = {
        width: window.innerWidth,
        height: window.innerHeight
    }
}

const updateMiniSize = () => {
    const element = miniLayerRef.value
    if (!element) return

    const rect = element.getBoundingClientRect()
    if (rect.width > 0 && rect.height > 0) {
        miniSize.value = { width: rect.width, height: rect.height }
    }
}

onMounted(() => {
    updateViewportSize()
    window.addEventListener('resize', updateViewportSize)
    miniResizeObserver = new ResizeObserver(updateMiniSize)
    if (miniLayerRef.value) {
        miniResizeObserver.observe(miniLayerRef.value)
    }
    requestAnimationFrame(updateMiniSize)
})

onBeforeUnmount(() => {
    window.removeEventListener('resize', updateViewportSize)
    miniResizeObserver?.disconnect()
})

const surfaceTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)
const backdropTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : SOFT_SPRING)
const contentTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)

const miniScaleX = computed(() => Math.min(1, miniSize.value.width / viewportSize.value.width))
const miniScaleY = computed(() => Math.min(1, miniSize.value.height / viewportSize.value.height))
const fullLayerInitial = {
    opacity: 0,
    scaleX: 1,
    scaleY: 1,
    filter: 'blur(6px)'
}
const fullLayerOpen = {
    opacity: 1,
    scaleX: 1,
    scaleY: 1,
    filter: 'blur(0px)'
}
const fullLayerClosed = {
    opacity: 0,
    scaleX: 1,
    scaleY: 1,
    filter: 'blur(6px)'
}

const surfaceMotion = computed(() => props.isFullscreen ? {
    scaleX: 1,
    scaleY: 1,
    x: 0,
    y: 0,
    borderRadius: 0,
    backgroundColor: 'rgb(var(--primary-color))',
    boxShadow: '0 0 0 rgba(0, 0, 0, 0)'
} : {
    scaleX: miniScaleX.value,
    scaleY: miniScaleY.value,
    x: 20,
    y: -20,
    borderRadius: 8,
    backgroundColor: 'rgba(var(--secondary-color), 0.2)',
    boxShadow: '0 12px 30px rgba(0, 0, 0, 0.28)'
})
</script>

<style scoped>
.player-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
    pointer-events: none;
    background: rgba(0, 0, 0, 0.18);
}

.player-backdrop-shade {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    will-change: opacity, backdrop-filter;
}

.player-surface {
    position: fixed;
    inset: 0;
    z-index: 100;
    pointer-events: none;
}

.surface-fill {
    position: absolute;
    inset: 0;
    overflow: hidden;
    transform-origin: 0% 100%;
    will-change: transform, border-radius, background-color, box-shadow;
    backface-visibility: hidden;
    pointer-events: auto;
}

.player-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    transform-origin: 0% 100%;
    will-change: transform, opacity, filter;
}

.mini-layer {
    position: absolute;
    inset: auto;
    left: 20px;
    bottom: 20px;
    width: fit-content;
    max-width: calc(100vw - 40px);
    height: 90px;
    display: flex;
    align-items: stretch;
    pointer-events: auto;
}

.full-layer {
    overflow: hidden;
}
</style>
