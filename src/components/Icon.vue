<template>
    <MotionSpan class="icon-wrapper" :class="[
        `icon-${size}`,
        { 'icon-hover': hover, 'icon-clickable': clickable }
    ]" :style="iconStyles" :while-hover="hoverState" :while-press="pressState"
        :transition="microTransition" @click="handleClick">
        <svg v-if="svgContent" class="icon-svg" :width="actualSize" :height="actualSize" :viewBox="viewBox"
            v-html="svgContent"></svg>
        <MotionDiv v-else-if="loading" class="loading" :animate="{ opacity: [0.5, 1, 0.5] }"
            :transition="pulseTransition">⟳</MotionDiv>
        <div v-else class="error">!</div>
    </MotionSpan>
</template>

<script setup>
import { computed, ref, watch, onMounted } from 'vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
    // SVG文件路径
    src: {
        type: String,
        required: true
    },
    // 图标大小：xs, sm, md, lg, xl 或具体数值
    size: {
        type: [String, Number],
        default: 'xs'
    },
    // 主色调
    color: {
        type: String,
        default: ''
    },
    // hover颜色
    hoverColor: {
        type: String,
        default: ''
    },
    // 是否启用hover效果
    hover: {
        type: Boolean,
        default: false
    },
    // 是否可点击
    clickable: {
        type: Boolean,
        default: false
    }
})

const emit = defineEmits(['click', 'error'])

const svgContent = ref('')
const viewBox = ref('0 0 24 24')
const loading = ref(false)
const MotionSpan = motion.span
const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const pulseTransition = computed(() => reducedMotion.value
    ? INSTANT_MOTION
    : { type: 'keyframes', duration: 1.5, ease: 'easeInOut', repeat: Infinity })
const hoverState = computed(() => props.hover
    ? { color: props.hoverColor || 'rgb(var(--primary-hover-color, 0, 86, 179))', scale: props.clickable ? 1.05 : 1 }
    : props.clickable ? { scale: 1.05 } : undefined)
const pressState = computed(() => props.clickable ? { scale: 0.95 } : undefined)

// 预设尺寸映射
const sizeMap = {
    xs: 12,
    sm: 16,
    md: 24,
    lg: 32,
    xl: 48
}

// 实际尺寸
const actualSize = computed(() => {
    if (typeof props.size === 'number') return props.size
    return sizeMap[props.size] || sizeMap.md
})

// 计算样式
const iconStyles = computed(() => {
    const styles = {
        '--icon-size': `${actualSize.value}px`
    }

    if (props.color) {
        styles['--icon-color'] = props.color
    }

    if (props.hoverColor) {
        styles['--icon-hover-color'] = props.hoverColor
    }

    return styles
})

// 加载和处理SVG
const loadSvg = async () => {
    if (!props.src) return

    loading.value = true
    try {
        const response = await fetch(props.src)
        if (!response.ok) throw new Error('Failed to load SVG')

        let svgText = await response.text()

        // 解析SVG，提取viewBox和内容
        const parser = new DOMParser()
        const svgDoc = parser.parseFromString(svgText, 'image/svg+xml')
        const svgElement = svgDoc.querySelector('svg')

        if (svgElement) {
            // 获取viewBox
            const vb = svgElement.getAttribute('viewBox')
            if (vb) {
                viewBox.value = vb
            }

            // 处理SVG内容，移除固定颜色属性，让CSS控制颜色
            let content = svgElement.innerHTML
            // console.log("svg", content)
            content = content
                .replace(/fill="[^"]*"/g, '')
                .replace(/stroke="[^"]*"/g, '')
                .replace(/fill:#[a-fA-F0-9]{3,6}/g, '')
                .replace(/stroke:#[a-fA-F0-9]{3,6}/g, '')

            svgContent.value = content
        }
    } catch (error) {
        console.error('Error loading SVG:', error)
        svgContent.value = ''
        emit('error', error)
    } finally {
        loading.value = false
    }
}

// 事件处理
const handleClick = (event) => {
    if (props.clickable) {
        emit('click', event)
    }
}

// 监听src变化
watch(() => props.src, loadSvg, { immediate: true })

onMounted(() => {
    loadSvg()
})
</script>

<style scoped>
.icon-wrapper {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-size);
    height: var(--icon-size);
    color: var(--icon-color, rgb(var(--primary-color, 51, 51, 51)));
    user-select: none;
}

.icon-wrapper.icon-clickable {
    cursor: pointer;
}

.icon-svg {
    width: 100%;
    height: 100%;
    fill: currentColor;
    stroke: currentColor;
}

.loading {
    opacity: 0.5;
}

.error {
    color: #f56565;
}

</style>
