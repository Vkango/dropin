<template>
    <span class="icon-wrapper" :class="[
        `icon-${size}`,
        { 'icon-hover': hover, 'icon-clickable': clickable }
    ]" :style="iconStyles" @click="handleClick">
        <svg v-if="svgContent" class="icon-svg" :width="actualSize" :height="actualSize" :viewBox="viewBox"
            v-html="svgContent"></svg>
        <div v-else-if="loading" class="loading">⟳</div>
        <div v-else class="error">!</div>
    </span>
</template>

<script setup>
import { computed, ref, watch, onMounted } from 'vue'

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
    color: rgb(var(--icon-color, var(--primary-color, #333)));
    transition: color 0.2s ease, transform 0.1s ease;
    user-select: none;
}

.icon-wrapper.icon-hover:hover {
    color: rgb(var(--icon-hover-color, var(--primary-hover-color, #007acc)));
}

.icon-wrapper.icon-clickable {
    cursor: pointer;
}

.icon-wrapper.icon-clickable:hover {
    transform: scale(1.05);
}

.icon-wrapper.icon-clickable:active {
    transform: scale(0.95);
}

.icon-svg {
    width: 100%;
    height: 100%;
    fill: currentColor;
    stroke: currentColor;
}

.loading {
    opacity: 0.5;
    animation: pulse 1.5s ease-in-out infinite;
}

.error {
    color: #f56565;
}

@keyframes pulse {

    0%,
    100% {
        opacity: 0.5;
    }

    50% {
        opacity: 1;
    }
}
</style>
