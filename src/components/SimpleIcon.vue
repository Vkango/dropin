<template>
    <div class="svg-icon" :class="{ 'hover-enabled': hover }" :style="iconStyles">
        <svg :width="size" :height="size" :viewBox="viewBox" class="svg-content">
            <use :href="`${src}#icon`" v-if="isFragment" />
            <image :href="src" v-else width="100%" height="100%" />
        </svg>
    </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
    // SVG图片地址或者SVG fragment ID
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
    // viewBox
    viewBox: {
        type: String,
        default: '0 0 24 24'
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

// 判断是否是SVG fragment
const isFragment = computed(() => {
    return props.src.includes('#')
})

// 计算样式
const iconStyles = computed(() => {
    const size = typeof props.size === 'number' ? `${props.size}px` : props.size

    return {
        '--icon-size': size,
        '--icon-color': props.color || 'var(--primary-color, currentColor)',
        '--icon-secondary-color': props.secondaryColor || 'var(--secondary-color, #888888)',
        '--icon-hover-color': props.hoverColor || props.color || 'var(--primary-hover-color, #cccccc)',
        'width': size,
        'height': size
    }
})
</script>

<style scoped>
.svg-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    color: var(--icon-color);
}

.svg-content {
    width: 100%;
    height: 100%;
    fill: currentColor;
    stroke: currentColor;
    transition: all 0.2s ease;
    filter: drop-shadow(0 0 0 var(--icon-color));
}

/* Hover效果 */
.svg-icon.hover-enabled:hover {
    color: var(--icon-hover-color);
}

/* 预设尺寸类 */
.icon-xs {
    --icon-size: 12px;
}

.icon-sm {
    --icon-size: 16px;
}

.icon-md {
    --icon-size: 24px;
}

.icon-lg {
    --icon-size: 32px;
}

.icon-xl {
    --icon-size: 48px;
}
</style>
