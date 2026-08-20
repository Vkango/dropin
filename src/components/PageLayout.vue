<template>
    <div class="page-layout-scroll">
        <div class="page-layout-titlebar-spacer" aria-hidden="true"></div>
        <main class="page-layout" :class="{ 'has-header': Boolean($slots.header) }">
            <header v-if="$slots.header" class="page-layout-header">
                <slot name="header" />
            </header>
            <section class="page-layout-content">
                <slot />
            </section>
        </main>
    </div>
</template>

<style scoped>
.page-layout-scroll {
    width: 100%;
    height: 100%;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
    color: rgb(var(--text-color));
}

.page-layout {
    width: 100%;
    min-width: 0;
    min-height: calc(100% - 64px);
    background: rgba(var(--surface-color), 0.1);
    border-radius: 10px 0 0 0;
}

.page-layout-titlebar-spacer {
    width: 100%;
    height: 64px;
    min-height: 64px;
    pointer-events: none;
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
    .page-layout-header {
        padding: 0 24px;
    }

    .page-layout-content {
        padding: 24px;
    }
}
</style>
