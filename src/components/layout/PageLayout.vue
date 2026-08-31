<template>
    <div class="page-layout-scroll">
        <div class="page-layout-titlebar-spacer" aria-hidden="true"></div>
        <main class="page-layout" :class="{ 'has-header': Boolean($slots.header), flush }">
            <header v-if="$slots.header" class="page-layout-header">
                <slot name="header" />
            </header>
            <section ref="contentRef" class="page-layout-content">
                <slot />
            </section>
        </main>
    </div>
</template>

<script setup>
import { onActivated, onBeforeUnmount, onMounted, ref } from 'vue'
import { useReducedMotion } from 'motion-v'
import { animateElement, APPLE_SPRING, INSTANT_MOTION } from '@/utils/motion.js'

const reducedMotion = useReducedMotion()
defineProps({ flush: { type: Boolean, default: false } })
const contentRef = ref(null)
const blockAnimations = new WeakMap()
let animationFrame = null

const getContentBlocks = () => {
    const content = contentRef.value
    if (!content || content.getClientRects().length === 0) return []

    const contentRoot = content.firstElementChild
    const contentBlocks = contentRoot?.children?.length
        ? [...contentRoot.children]
        : [...content.children]
    const header = content.parentElement?.querySelector('.page-layout-header')

    return [header, ...contentBlocks]
        .filter((element, index, elements) => element && elements.indexOf(element) === index)
        .filter((element) => element.getClientRects().length > 0)
}

const prepareContentBlocks = () => {
    if (reducedMotion.value) return

    getContentBlocks().forEach((element) => {
        element.style.opacity = '0'
        element.style.transform = 'translateY(10px)'
        element.style.filter = 'blur(2px)'
    })
}

const animateContentBlocks = () => {
    if (animationFrame !== null) cancelAnimationFrame(animationFrame)
    prepareContentBlocks()

    animationFrame = requestAnimationFrame(() => {
        animationFrame = null

        getContentBlocks().forEach((element, index) => {
            const animation = animateElement(
                element,
                {
                    opacity: [0, 1],
                    y: [10, 0],
                    filter: ['blur(2px)', 'blur(0px)']
                },
                reducedMotion.value
                    ? INSTANT_MOTION
                    : { ...APPLE_SPRING, delay: index * 0.035 },
                reducedMotion.value
            )
            blockAnimations.set(element, animation)
            animation.finished.then(() => {
                if (blockAnimations.get(element) !== animation) return
                blockAnimations.delete(element)
                element.style.opacity = ''
                element.style.transform = ''
                element.style.filter = ''
            }, () => { })
        })
    })
}

onMounted(() => {
    prepareContentBlocks()
    animateContentBlocks()
})
onActivated(() => {
    prepareContentBlocks()
    animateContentBlocks()
})
onBeforeUnmount(() => {
    if (animationFrame !== null) cancelAnimationFrame(animationFrame)
})
</script>

<style scoped>
.page-layout-scroll {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
    color: rgb(var(--text-color));
}

.page-layout {
    flex: 0 0 auto;
    width: 100%;
    min-width: 0;
    min-height: calc(100% - 64px);
    background: transparent;
    border-radius: 10px 0 0 0;
}

.page-layout-titlebar-spacer {
    width: 100%;
    height: 64px;
    min-height: 64px;
    pointer-events: none;
    flex: 0 0 64px;
}

.page-layout-header {
    width: 100%;
    position: relative;
    z-index: 0;
    overflow: hidden;
    border-radius: 10px 0 0 0;
}

.page-layout-content {
    width: 100%;
    min-width: 0;
    padding: 28px clamp(48px, 8vw, 112px) 48px;
    /* overflow: hidden; */
}

.page-layout.flush {
    display: flex;
    flex-direction: column;
    min-height: calc(100% - 64px);
}

.page-layout.flush .page-layout-content {
    display: flex;
    flex: 1 1 auto;
    min-height: 0;
    padding: 0;
}

.page-layout.has-header .page-layout-content {
    margin-top: -120px;
}

.page-layout-scroll::-webkit-scrollbar {
    width: 4px;
}

.page-layout-scroll::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
    margin-top: 64px;
    margin-bottom: 8px;
}

.page-layout-scroll::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.page-layout-scroll::-webkit-scrollbar-thumb:hover {
    background: rgba(var(--outline-color), 0.5);
}

@media (max-width: 768px) {
    .page-layout-content {
        padding: 24px;
    }
}
</style>
