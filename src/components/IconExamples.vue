<!-- Icon组件使用示例 -->
<template>
    <div class="icon-examples">
        <h2>Icon 组件使用示例</h2>

        <!-- 基础用法 -->
        <section class="example-section">
            <h3>基础用法</h3>
            <div class="example-row">
                <!-- 使用SVG文件 -->
                <Icon src="/assets/home.svg" size="md" />
                <Icon src="/assets/search.svg" size="lg" color="#007acc" />
                <Icon src="/assets/user.svg" size="xl" color="#28a745" />
            </div>
        </section>

        <!-- 不同尺寸 -->
        <section class="example-section">
            <h3>不同尺寸</h3>
            <div class="example-row">
                <Icon src="/assets/apps.svg" size="xs" />
                <Icon src="/assets/apps.svg" size="sm" />
                <Icon src="/assets/apps.svg" size="md" />
                <Icon src="/assets/apps.svg" size="lg" />
                <Icon src="/assets/apps.svg" size="xl" />
                <Icon src="/assets/apps.svg" :size="64" />
            </div>
        </section>

        <!-- 颜色主题 -->
        <section class="example-section">
            <h3>颜色主题</h3>
            <div class="example-row">
                <Icon src="/assets/home.svg" color="#007acc" />
                <Icon src="/assets/home.svg" color="#28a745" />
                <Icon src="/assets/home.svg" color="#dc3545" />
                <Icon src="/assets/home.svg" color="#ffc107" />
                <Icon src="/assets/home.svg" color="#6f42c1" />
            </div>
        </section>

        <!-- 交互效果 -->
        <section class="example-section">
            <h3>交互效果</h3>
            <div class="example-row">
                <Icon src="/assets/user.svg" size="lg" hover hover-color="#007acc" @click="handleIconClick" clickable />
                <Icon src="/assets/settings.svg" size="lg" hover hover-color="#28a745" clickable
                    @click="handleIconClick" />
            </div>
        </section>

        <!-- 在按钮中使用 -->
        <section class="example-section">
            <h3>在按钮中使用</h3>
            <div class="example-row">
                <button class="icon-button">
                    <Icon src="/assets/close.svg" size="sm" color="white" />
                    关闭
                </button>
                <button class="icon-button primary">
                    <Icon src="/assets/user.svg" size="sm" color="white" />
                    用户
                </button>
                <button class="icon-button secondary">
                    <Icon src="/assets/settings.svg" size="sm" />
                    设置
                </button>
            </div>
        </section>
    </div>
</template>

<script setup>
const handleIconClick = () => {
    console.log('图标被点击了!')
}
</script>

<style scoped>
/* CSS变量定义主题色 */
:root {
    --primary-color: #007acc;
    --primary-hover-color: #0056b3;
    --secondary-color: #6c757d;
}

.icon-examples {
    padding: 20px;
    max-width: 800px;
    margin: 0 auto;
}

.example-section {
    margin-bottom: 30px;
}

.example-section h3 {
    margin-bottom: 15px;
    color: #333;
}

.example-row {
    display: flex;
    align-items: center;
    gap: 15px;
    flex-wrap: wrap;
}

.icon-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border: 1px solid #ddd;
    border-radius: 4px;
    background: white;
    cursor: pointer;
    transition: all 0.2s;
}

.icon-button:hover {
    background: #f8f9fa;
    border-color: #007acc;
}

.icon-button.primary {
    background: #007acc;
    color: white;
    border-color: #007acc;
}

.icon-button.primary:hover {
    background: #0056b3;
}

.icon-button.secondary {
    background: #6c757d;
    color: white;
    border-color: #6c757d;
}

.icon-button.secondary:hover {
    background: #545b62;
}
</style>
