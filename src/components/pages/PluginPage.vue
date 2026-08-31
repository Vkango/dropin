<template>
  <PageLayout flush>
    <div class="plugin-page">
      <PluginHostFrame v-if="plugin" :plugin="plugin" :host-state="hostState" />
      <div v-else class="plugin-missing">{{ t('plugins.empty') }}</div>
    </div>
  </PageLayout>
</template>
<script setup>
import { computed, inject } from 'vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import PluginHostFrame from '@/components/plugin/PluginHostFrame.vue'
import { useI18n } from '@/i18n/index.js'
const props = defineProps({ plugin: { type: Object, default: null } })
const { t } = useI18n()
const hostState = inject('pluginHostState')
const plugin = computed(() => props.plugin)
</script>
<style scoped>
.plugin-page { display: flex; flex: 1 1 auto; flex-direction: column; min-height: 0; width: 100%; }.plugin-page :deep(.plugin-host) { flex: 1 1 auto; min-height: 0; }.plugin-missing { display: grid; flex: 1 1 auto; min-height: 240px; place-items: center; color: rgba(var(--text-color), .55); }
</style>
