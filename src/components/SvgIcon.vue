<template>
    <MotionDiv class="svg-icon" :style="iconStyles" :while-hover="hoverState" :transition="microTransition">
        <svg v-if="svgContent" :width="size" :height="size" :viewBox="viewBox" v-html="svgContent"
            class="svg-content" />
        <div v-else-if="loading" class="loading-placeholder">
            <MotionDiv class="loading-spinner" :animate="{ rotate: 360 }" :transition="spinnerTransition"></MotionDiv>
        </div>
        <div v-else class="error-placeholder">
            <span>!</span>
        </div>
    </MotionDiv>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { motion, useReducedMotion } from 'motion-v'
import { INSTANT_MOTION, MICRO_SPRING } from '../utils/motion.js'

const props = defineProps({
    // SVG图片地址
    src: {
        type: String,
        required: true
    },
    // 图标大小
    size: {
        type: [String, Number],
        default: 24
    },
    // 主色调，如果不提供则从CSS变量获取
    color: {
        type: String,
        default: ''
    },
    // 次色调（用于双色图标）
    secondaryColor: {
        type: String,
        default: ''
    },
    // 是否启用hover效果
    hover: {
        type: Boolean,
        default: false
    },
    // hover时的颜色
    hoverColor: {
        type: String,
        default: ''
    }
})

const svgContent = ref('')
const viewBox = ref('0 0 24 24')
const loading = ref(false)
const MotionDiv = motion.div
const reducedMotion = useReducedMotion()
const microTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : MICRO_SPRING)
const spinnerTransition = computed(() => reducedMotion.value
    ? INSTANT_MOTION
    : { type: 'keyframes', duration: 1, ease: 'linear', repeat: Infinity })
const hoverState = computed(() => props.hover
    ? { color: props.hoverColor || 'var(--primary-hover-color, #cccccc)' }
    : undefined)

// 计算样式
const iconStyles = computed(() => {
    const styles = {
        '--icon-size': typeof props.size === 'number' ? `${props.size}px` : props.size,
        '--icon-color': props.color || 'var(--primary-color, #ffffff)',
        '--icon-secondary-color': props.secondaryColor || 'var(--secondary-color, #888888)',
        '--icon-hover-color': props.hoverColor || props.color || 'var(--primary-hover-color, #cccccc)'
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

        // 解析SVG，提取viewBox
        const parser = new DOMParser()
        const svgDoc = parser.parseFromString(svgText, 'image/svg+xml')
        const svgElement = svgDoc.querySelector('svg')

        if (svgElement) {
            // 获取viewBox
            const vb = svgElement.getAttribute('viewBox')
            if (vb) {
                viewBox.value = vb
            }

            // 处理SVG内容，替换颜色属性
            svgContent.value = processSvgContent(svgElement.innerHTML)
        }
    } catch (error) {
        console.error('Error loading SVG:', error)
        svgContent.value = ''
    } finally {
        loading.value = false
    }
}

// 处理SVG内容，替换颜色
const processSvgContent = (content) => {
    return content
        // 替换fill属性为CSS变量
        .replace(/fill="[^"]*"/g, 'fill="var(--icon-color)"')
        // 替换stroke属性为CSS变量
        .replace(/stroke="[^"]*"/g, 'stroke="var(--icon-color)"')
        // 处理特殊情况：如果有多个颜色，可以使用类名
        .replace(/class="primary"/g, 'style="fill: var(--icon-color)"')
        .replace(/class="secondary"/g, 'style="fill: var(--icon-secondary-color)"')
}

// 监听src变化
watch(() => props.src, loadSvg, { immediate: true })

onMounted(() => {
    loadSvg()
})
</script>

<style scoped>
.svg-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-size);
    height: var(--icon-size);
}

.svg-content {
    width: 100%;
    height: 100%;
    fill: var(--icon-color);
    stroke: var(--icon-color);
}

/* 加载状态 */
.loading-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

.loading-spinner {
    width: 60%;
    height: 60%;
    border: 2px solid var(--icon-color, #ffffff);
    border-top: 2px solid transparent;
    border-radius: 50%;
}

/* 错误状态 */
.error-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--icon-color, #ffffff);
    font-weight: bold;
    font-size: 0.8em;
    opacity: 0.5;
}

/* 响应式大小 */
.svg-icon.small {
    --icon-size: 16px;
}

.svg-icon.medium {
    --icon-size: 24px;
}

.svg-icon.large {
    --icon-size: 32px;
}

.svg-icon.extra-large {
    --icon-size: 48px;
}
</style>
