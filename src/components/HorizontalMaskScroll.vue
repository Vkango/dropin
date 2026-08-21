<template>
  <div ref="containerRef" class="horizontal-mask-scroll" :class="{ 'align-end': align === 'end' }" :style="scrollStyle"
    @scroll="updateScrollState" @wheel="handleWheel">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

const props = withDefaults(defineProps<{
  fadeWidth?: number;
  gap?: number | string;
  align?: "start" | "end";
  padding?: string;
}>(), {
  fadeWidth: 20,
  gap: 0,
  align: "start",
  padding: "0",
});

const containerRef = ref<HTMLElement | null>(null);
const canScrollLeft = ref(false);
const canScrollRight = ref(false);
const hasOverflow = ref(false);
let resizeObserver: ResizeObserver | null = null;
let mutationObserver: MutationObserver | null = null;

const scrollStyle = computed(() => {
  const gapValue = typeof props.gap === "number" ? `${props.gap}px` : props.gap;
  const fade = `${props.fadeWidth}px`;

  const maskImage = hasOverflow.value
    ? canScrollLeft.value && canScrollRight.value
      ? `linear-gradient(90deg, transparent 0, rgba(0, 0, 0, 1) ${fade}, rgba(0, 0, 0, 1) calc(100% - ${fade}), transparent 100%)`
      : canScrollLeft.value
        ? `linear-gradient(90deg, transparent 0, rgba(0, 0, 0, 1) ${fade}, rgba(0, 0, 0, 1) 100%)`
        : canScrollRight.value
          ? `linear-gradient(90deg, rgba(0, 0, 0, 1) 0, rgba(0, 0, 0, 1) calc(100% - ${fade}), transparent 100%)`
          : "none"
    : "none";

  return {
    "--horizontal-mask-scroll-gap": gapValue,
    "--horizontal-mask-scroll-padding": props.padding,
    maskImage,
    WebkitMaskImage: maskImage,
  };
});

function updateScrollState() {
  const container = containerRef.value;
  if (!container) {
    canScrollLeft.value = false;
    canScrollRight.value = false;
    hasOverflow.value = false;
    return;
  }

  const maxScrollLeft = Math.max(0, container.scrollWidth - container.clientWidth);
  hasOverflow.value = maxScrollLeft > 1;
  canScrollLeft.value = container.scrollLeft > 1;
  canScrollRight.value = container.scrollLeft < maxScrollLeft - 1;
}

function handleWheel(event: WheelEvent) {
  const container = containerRef.value;
  if (!container || !hasOverflow.value) return;

  const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
  if (!delta) return;

  event.preventDefault();
  container.scrollLeft += delta;
  updateScrollState();
}

onMounted(() => {
  const container = containerRef.value;
  if (!container) return;

  updateScrollState();

  if (typeof ResizeObserver !== "undefined") {
    resizeObserver = new ResizeObserver(() => updateScrollState());
    resizeObserver.observe(container);
  }

  if (typeof MutationObserver !== "undefined") {
    mutationObserver = new MutationObserver(() => updateScrollState());
    mutationObserver.observe(container, {
      childList: true,
      subtree: true,
      characterData: true,
    });
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  mutationObserver?.disconnect();
});
</script>

<style scoped>
.horizontal-mask-scroll {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  width: 100%;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
  gap: var(--horizontal-mask-scroll-gap, 0px);
  padding: var(--horizontal-mask-scroll-padding, 0);
  scrollbar-width: none;
  -ms-overflow-style: none;
  height: 24px;
}

.horizontal-mask-scroll.align-end {
  justify-content: flex-end;
}

.horizontal-mask-scroll::-webkit-scrollbar {
  display: none;
}
</style>
