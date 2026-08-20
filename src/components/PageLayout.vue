<template>
    <main class="page-layout">
        <header v-if="$slots.header" class="page-layout-header">
            <slot name="header" />
        </header>
        <section class="page-layout-content">
            <slot />
        </section>
    </main>
</template>

<style scoped>
.page-layout {
    width: 100%;
    height: 100%;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
    background: rgba(var(--surface-color), 0.1);
    color: rgb(var(--text-color));
}

.page-layout-header {
    width: 100%;
}

.page-layout-content {
    width: 100%;
    min-width: 0;
    padding: 28px clamp(48px, 8vw, 112px) 48px;
}

.page-layout::-webkit-scrollbar {
    width: 4px;
}

.page-layout::-webkit-scrollbar-track {
    background: rgba(var(--outline-color), 0.1);
    border-radius: 2px;
}

.page-layout::-webkit-scrollbar-thumb {
    background: rgba(var(--outline-color), 0.3);
    border-radius: 2px;
}

.page-layout::-webkit-scrollbar-thumb:hover {
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
