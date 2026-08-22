import { animate, useReducedMotion } from 'motion-v'
import { computed, unref } from 'vue'

export const APPLE_SPRING = {
  type: 'spring',
  stiffness: 260,
  damping: 28,
  mass: 0.8
}

export const MICRO_SPRING = {
  type: 'spring',
  stiffness: 500,
  damping: 35,
  mass: 0.5
}

export const SOFT_SPRING = {
  type: 'spring',
  stiffness: 220,
  damping: 30,
  mass: 0.9
}

export const LINEAR_LOOP = {
  type: 'keyframes',
  ease: 'linear',
  duration: 24,
  repeat: Infinity
}

export const INSTANT_MOTION = {
  type: 'keyframes',
  duration: 0
}

export const motionVariants = {
  page: {
    enter: { opacity: [0, 1], y: [20, 0] },
    leave: { opacity: [1, 0], y: [0, -20] },
    spring: SOFT_SPRING
  },
  banner: {
    enter: { opacity: [0, 1], scale: [1.1, 1], y: [30, 0], filter: ['blur(15px)', 'blur(5px)'] },
    leave: { opacity: [1, 0], scale: [1, 0.9], y: [0, -30], filter: ['blur(5px)', 'blur(15px)'] },
    spring: SOFT_SPRING,
    absolute: true
  },
  cover: {
    enter: { opacity: [0, 1], scale: [0.95, 1] },
    leave: { opacity: [1, 0], scale: [1.05, 1] },
    spring: APPLE_SPRING,
    absolute: true
  },
  albumCover: {
    // 封面替换只做淡入淡出+缩放，不带动 rotate，
    // 避免 motion 在离场时把唱片当前旋转角拉回 0 度。
    enter: { opacity: [0, 1], scale: [0.92, 1] },
    leave: { opacity: [1, 0], scale: [1, 1.08] },
    spring: SOFT_SPRING,
    absolute: true
  },
  miniCover: {
    enter: { opacity: [0, 1], scale: [0.72, 1], rotateY: [-90, 0], filter: ['blur(3px)', 'blur(0px)'] },
    leave: { opacity: [1, 0], scale: [1, 0.72], rotateY: [0, 90], filter: ['blur(0px)', 'blur(3px)'] },
    spring: MICRO_SPRING,
    absolute: true
  },
  miniLyric: {
    enter: { opacity: [0, 1], y: [5, 0] },
    leave: { opacity: [1, 0], y: [0, -5] },
    spring: MICRO_SPRING
  },
  songInfo: {
    enter: { opacity: [0, 1], y: [20, 0] },
    leave: { opacity: [1, 0], y: [0, -20] },
    spring: SOFT_SPRING
  },
  modal: {
    enter: { opacity: [0, 1] },
    leave: { opacity: [1, 0] },
    spring: SOFT_SPRING
  },
  card: {
    enter: { opacity: [0, 1], scale: [0.8, 1], rotateX: [-15, 0], rotateY: [10, 0], z: [-100, 0] },
    leave: { opacity: [1, 0], scale: [1, 0.8], rotateX: [0, 15], rotateY: [0, -10], z: [0, -100] },
    spring: SOFT_SPRING
  }
}

export function useMotionPreferences() {
  const reducedMotion = useReducedMotion()
  const transition = (spring = APPLE_SPRING) => computed(() => (
    unref(reducedMotion) ? INSTANT_MOTION : spring
  ))

  return { reducedMotion, transition }
}

const activeAnimations = new WeakMap()

export function animateElement(element, keyframes, options, reducedMotion = false) {
  activeAnimations.get(element)?.cancel()

  const animation = animate(element, keyframes, reducedMotion ? INSTANT_MOTION : options)
  activeAnimations.set(element, animation)

  const cleanup = () => {
    if (activeAnimations.get(element) === animation) {
      activeAnimations.delete(element)
    }
  }

  // Motion rejects `finished` when an animation is cancelled; handle both paths
  // so rapid enter/leave changes do not produce unhandled promise rejections.
  animation.finished.then(cleanup, cleanup)

  return animation
}
