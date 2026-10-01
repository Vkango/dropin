<template>
  <PageLayout>
    <template #header>
      <div class="music-banner">
        <div class="image-container">
          <MotionTransition variant="banner">
            <img :key="bannerImage" class="background-image" :src="bannerImage" referrerpolicy="no-referrer" />
          </MotionTransition>
        </div>
        <div class="banner-content">
          <div class="title">{{ t('app.name') }}</div>
          <h2 class="library-title">{{ t('plugins.title') }}</h2>
        </div>
      </div>
    </template>

    <div class="plugins-page">
      <div class="toolbar">
        <div class="filter-tabs">
          <button
            v-for="category in categories"
            :key="category.id"
            class="filter-tab"
            :class="{ active: activeCategory === category.id }"
            @click="activeCategory = category.id"
          >
            {{ categoryLabel(category.id) }}
            <span class="count">{{ category.count }}</span>
          </button>
        </div>
        <button class="import-btn" :disabled="runtime.state.loading" @click="installPlugin">
          {{ t('plugins.import') }}
        </button>
      </div>

      <p v-if="runtime.state.error" class="error">{{ runtime.state.error }}</p>
      <div v-if="runtime.state.loading && !plugins.length" class="empty-state">
        {{ t('plugins.loading') }}
      </div>

      <div v-else class="plugins-list">
        <article
          v-for="plugin in filteredPlugins"
          :key="plugin.id"
          class="plugin-card"
          :class="{ disabled: !plugin.enabled || plugin.faulted }"
          @click="openDetails(plugin)"
        >
          <div class="plugin-icon">
            <img :src="iconFor(plugin)" :alt="plugin.name" />
          </div>

          <div class="plugin-info" role="button" tabindex="0"
            @keydown.enter="openDetails(plugin)" @keydown.space.prevent="openDetails(plugin)">
            <div class="plugin-heading">
              <h3>{{ plugin.name }}</h3>
              <span>v{{ plugin.version }} · {{ statusLabel(plugin) }}</span>
            </div>
            <p>{{ plugin.description || plugin.id }}</p>
            <p v-if="plugin.faulted && plugin.lastError" class="fault-error">{{ plugin.lastError }}</p>
            <div class="permission-row">
              <span
                v-for="permission in declaredPermissions(plugin)"
                :key="permission"
                :title="permission"
                :class="{ granted: isPermissionGranted(plugin, permission) }"
              >
                {{ permissionLabel(permission) }}
              </span>
            </div>
          </div>

          <div class="plugin-actions" @click.stop>
            <button
              v-if="!plugin.enabled"
              class="action-btn enable"
              :disabled="plugin.faulted"
              @click="enablePlugin(plugin)"
            >
              {{ t('plugins.enable') }}
            </button>
            <button v-else class="action-btn disable" @click="disablePlugin(plugin)">
              {{ t('plugins.disable') }}
            </button>
            <button class="action-btn permissions" @click="openDetails(plugin, 'permissions')">
              {{ t('plugins.permissions') }}
            </button>
            <button class="action-btn uninstall" @click="uninstallPlugin(plugin)">
              {{ t('plugins.uninstall') }}
            </button>
          </div>
        </article>
      </div>

      <div v-if="!runtime.state.loading && filteredPlugins.length === 0" class="empty-state">
        <h3>{{ t('plugins.empty') }}</h3>
      </div>
    </div>
  </PageLayout>

  <Dialog v-model="isPermissionDialogOpen" :width="520" :aria-labelledby="'plugin-permission-dialog-title'">
    <div class="dialog-content">
      <header class="dialog-header">
        <h2 id="plugin-permission-dialog-title">{{ t('plugins.permissions') }}</h2>
      </header>
      <PluginPermissionRequest v-if="permissionDialogPlugin" v-model="permissionDialogValue"
        :permissions="declaredPermissions(permissionDialogPlugin)" :description="t('plugins.permissionHint')" />
      <footer class="dialog-actions">
        <button type="button" class="dialog-button secondary" :disabled="isSavingPermissions"
          @click="isPermissionDialogOpen = false">
          {{ t('plugins.cancel') }}
        </button>
        <button type="button" class="dialog-button primary" :disabled="isSavingPermissions"
          @click="savePermissions">
          {{ t('plugins.savePermissions') }}
        </button>
      </footer>
    </div>
  </Dialog>

  <Dialog v-model="isDetailsOpen" :width="620" height="min(580px, calc(100dvh - 36px))"
    :aria-labelledby="'plugin-details-title'">
    <div class="dialog-content plugin-details">
      <header class="plugin-details-heading" v-if="detailPlugin">
        <img :src="iconFor(detailPlugin)" :alt="detailPlugin.name" />
        <div>
          <h2 id="plugin-details-title">{{ detailPlugin.name }}</h2>
          <span class="plugin-details-meta">v{{ detailPlugin.version }} · {{ statusLabel(detailPlugin) }}</span>
        </div>
      </header>
      <div class="plugin-tabs" role="tablist" :aria-label="detailPlugin?.name">
        <button v-for="tab in detailTabs" :key="tab" :id="`plugin-tab-${tab}`" type="button" role="tab"
          class="plugin-tab" :class="{ active: detailTab === tab }"
          :aria-controls="`plugin-panel-${tab}`" :aria-selected="detailTab === tab"
          :tabindex="detailTab === tab ? 0 : -1" @click="detailTab = tab"
          @keydown.right.prevent="moveDetailTab($event, 1)" @keydown.left.prevent="moveDetailTab($event, -1)">
          {{ t(`plugins.tab${tab.charAt(0).toUpperCase()}${tab.slice(1)}`) }}
        </button>
      </div>
      <div v-if="detailPlugin && detailTab === 'overview'" id="plugin-panel-overview" role="tabpanel"
        aria-labelledby="plugin-tab-overview" class="plugin-tab-panel">
        <p class="plugin-details-description">{{ detailPlugin.description || detailPlugin.id }}</p>
        <dl class="plugin-details-facts">
          <div><dt>{{ t('plugins.author') }}</dt><dd>{{ detailPlugin.author || '—' }}</dd></div>
          <div><dt>ID</dt><dd><code>{{ detailPlugin.id }}</code></dd></div>
          <div><dt>{{ t('plugins.permissions') }}</dt>
            <dd>
              <span v-for="permission in declaredPermissions(detailPlugin)" :key="permission" class="permission-chip">
                {{ permissionLabel(permission) }}
              </span>
              <span v-if="!declaredPermissions(detailPlugin).length">{{ t('plugins.noPermissions') }}</span>
            </dd>
          </div>
        </dl>
        <p v-if="detailPlugin.faulted && detailPlugin.lastError" class="fault-error">{{ detailPlugin.lastError }}</p>
      </div>
      <div v-else-if="detailPlugin && detailTab === 'wiki'" id="plugin-panel-wiki" role="tabpanel"
        aria-labelledby="plugin-tab-wiki" class="plugin-tab-panel">
        <div v-if="detailWikiLoading" class="plugin-wiki-state">{{ t('plugins.loading') }}</div>
        <MarkdownContent v-else-if="detailWiki" :source="detailWiki" />
        <div v-else class="plugin-wiki-state" :role="detailWikiError ? 'alert' : undefined">
          {{ detailWikiError || t('plugins.noWiki') }}
        </div>
      </div>
      <div v-else-if="detailPlugin && detailTab === 'permissions'" id="plugin-panel-permissions" role="tabpanel"
        aria-labelledby="plugin-tab-permissions" class="plugin-tab-panel">
        <PluginPermissionRequest v-model="detailPermissions" :permissions="declaredPermissions(detailPlugin)"
          :description="t('plugins.permissionHint')" />
        <footer class="dialog-actions">
          <button type="button" class="dialog-button primary" :disabled="isSavingDetailPermissions"
            @click="saveDetailPermissions">
            {{ t('plugins.savePermissions') }}
          </button>
        </footer>
      </div>
    </div>
  </Dialog>
</template>

<script setup>
import { computed, inject, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import Dialog from '@/components/ui/Dialog.vue'
import MotionTransition from '@/components/ui/MotionTransition.vue'
import PageLayout from '@/components/layout/PageLayout.vue'
import PluginPermissionRequest from '@/components/plugin/PluginPermissionRequest.vue'
import { useI18n } from '@/i18n/index.js'
import { pluginPermissionMeta } from '@/utils/pluginPermissions.js'
import MarkdownContent from '@/components/ui/MarkdownContent.vue'

const props = defineProps({
  pluginRuntime: {
    type: Object,
    required: true
  }
})

const { t, locale } = useI18n()
const currentSong = inject('currentSong')
const runtime = props.pluginRuntime
const activeCategory = ref('all')
const isPermissionDialogOpen = ref(false)
const permissionDialogPlugin = ref(null)
const permissionDialogValue = ref([])
const isSavingPermissions = ref(false)
const bannerImage = computed(() => currentSong.value?.cover || '/assets/cover.jpg')
const plugins = computed(() => runtime.state.plugins)
const categories = computed(() => runtime.categories.value)
const filteredPlugins = computed(() => activeCategory.value === 'all'
  ? plugins.value
  : plugins.value.filter((plugin) => plugin.categories?.includes(activeCategory.value))
)

const categoryLabel = (id) => ({
  all: t('plugins.categoryAll'),
  effects: t('plugins.categoryEffects'),
  visualizer: t('plugins.categoryVisualizer'),
  utility: t('plugins.categoryUtility'),
  theme: t('plugins.categoryTheme')
}[id] || id)

const iconFor = (plugin) => plugin.iconUrl || '/assets/plugin.svg'
const statusLabel = (plugin) => plugin.faulted
  ? t('plugins.statusFaulted')
  : plugin.enabled ? t('plugins.statusEnabled') : t('plugins.statusDisabled')

const declaredPermissions = (plugin) => plugin.permissions?.declared || []
const grantedPermissions = (plugin) => plugin.permissions?.granted || []
const isPermissionGranted = (plugin, permission) => grantedPermissions(plugin).includes(permission)
const permissionLabel = (permission) => pluginPermissionMeta(permission, t).label

const requestPermissions = (plugin) => {
  const declared = declaredPermissions(plugin)
  const selected = grantedPermissions(plugin).filter((permission) => declared.includes(permission))
  permissionDialogPlugin.value = plugin
  permissionDialogValue.value = [...selected]
  isPermissionDialogOpen.value = true
}

const isDetailsOpen = ref(false)
const detailPluginId = ref('')
const detailPlugin = computed(() => plugins.value.find((plugin) => plugin.id === detailPluginId.value) || null)
const detailTabs = ['overview', 'wiki', 'permissions']
const detailTab = ref('overview')
const detailWiki = ref('')
const detailWikiLoading = ref(false)
const detailWikiError = ref('')
const detailPermissions = ref([])
const isSavingDetailPermissions = ref(false)
let detailWikiRequest = 0

const openDetails = (plugin, tab = 'overview') => {
  detailWikiRequest += 1
  detailPluginId.value = plugin.id
  detailTab.value = tab
  detailWiki.value = ''
  detailWikiError.value = ''
  detailWikiLoading.value = false
  detailPermissions.value = grantedPermissions(plugin)
    .filter((permission) => declaredPermissions(plugin).includes(permission))
  isDetailsOpen.value = true
}

const moveDetailTab = async (event, offset) => {
  const buttons = [...event.currentTarget.parentElement.querySelectorAll('[role="tab"]')]
  const index = (detailTabs.indexOf(detailTab.value) + offset + detailTabs.length) % detailTabs.length
  detailTab.value = detailTabs[index]
  await nextTick()
  buttons[index]?.focus()
}

const loadDetailWiki = async () => {
  const plugin = detailPlugin.value
  if (!plugin) return
  const request = ++detailWikiRequest
  detailWikiLoading.value = true
  detailWiki.value = ''
  detailWikiError.value = ''
  try {
    const result = await runtime.pluginWiki(plugin.id, locale.value)
    if (request === detailWikiRequest) detailWiki.value = String(result?.wiki || '')
  } catch (error) {
    if (request === detailWikiRequest) detailWikiError.value = error?.message || String(error)
  } finally {
    if (request === detailWikiRequest) detailWikiLoading.value = false
  }
}

watch([isDetailsOpen, detailTab, locale], ([open, tab]) => {
  if (open && tab === 'wiki') void loadDetailWiki()
  if (!open) detailWikiRequest += 1
})

watch(detailPlugin, (plugin) => {
  if (!plugin) isDetailsOpen.value = false
})

const saveDetailPermissions = async () => {
  const plugin = detailPlugin.value
  if (!plugin || isSavingDetailPermissions.value) return
  isSavingDetailPermissions.value = true
  try {
    await runtime.setPermissions(plugin.id, [...detailPermissions.value])
  } catch (error) {
    runtime.setError(error)
  } finally {
    isSavingDetailPermissions.value = false
  }
}

const savePermissions = async () => {
  const plugin = permissionDialogPlugin.value
  if (!plugin || isSavingPermissions.value) return

  isSavingPermissions.value = true
  try {
    await runtime.setPermissions(plugin.id, Array.isArray(permissionDialogValue.value) ? permissionDialogValue.value : [])
    isPermissionDialogOpen.value = false
    return true
  } catch (error) {
    runtime.setError(error)
    return false
  } finally {
    isSavingPermissions.value = false
  }
}

const installPlugin = async () => {
  try {
    const plugin = await runtime.installFromPicker()
    if (plugin) await requestPermissions(plugin)
  } catch (error) {
    runtime.setError(error)
  }
}

const enablePlugin = async (plugin) => {
  const missing = declaredPermissions(plugin)
    .filter((permission) => !grantedPermissions(plugin).includes(permission))

  if (missing.length) {
    await requestPermissions(plugin)
    return
  }

  try {
    await runtime.enable(plugin.id)
  } catch (error) {
    runtime.setError(error)
  }
}

const disablePlugin = async (plugin) => {
  try {
    await runtime.disable(plugin.id)
  } catch (error) {
    runtime.setError(error)
  }
}

const uninstallPlugin = async (plugin) => {
  try {
    await runtime.uninstall(plugin.id)
  } catch (error) {
    runtime.setError(error)
  }
}

onMounted(() => {
  if (!runtime.state.plugins.length) runtime.refresh().catch(() => undefined)
})

onBeforeUnmount(() => { detailWikiRequest += 1 })
</script>

<style scoped>
.plugins-page {
  width: 100%;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.filter-tabs {
  display: flex;
  gap: 6px;
  overflow-x: auto;
}

.filter-tab,
.import-btn,
.action-btn {
  border: 0;
  border-radius: 5px;
  padding: 6px 10px;
  font-size: 12px;
  cursor: pointer;
  color: rgb(var(--text-color));
  background: rgba(var(--outline-color), 0.1);
}

.filter-tab.active,
.import-btn {
  background: rgba(var(--primary-color), 0.18);
}

.count {
  margin-left: 4px;
  color: rgba(var(--text-color), 0.5);
}

.plugins-list {
  display: grid;
  width: 100%;
}

.plugin-card {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  padding: 11px 0;
  border-bottom: 1px solid rgba(var(--outline-color), 0.1);
  cursor: pointer;
}

.plugin-card.disabled {
  opacity: 0.58;
}

.plugin-icon {
  width: 36px;
  height: 36px;
  flex: 0 0 auto;
}

.plugin-icon img {
  width: 100%;
  height: 100%;
  border-radius: 6px;
  object-fit: cover;
}

.plugin-info {
  flex: 1;
  min-width: 0;
}

.plugin-info:focus-visible { outline: 2px solid rgba(var(--primary-color), .6); outline-offset: 4px; border-radius: 6px; }
.plugin-details { display: flex; flex: 1; flex-direction: column; gap: 18px; min-height: 0; }
.plugin-details-heading { display: flex; flex: 0 0 auto; align-items: center; gap: 12px; }
.plugin-details-heading img { width: 44px; height: 44px; border-radius: 10px; object-fit: cover; }
.plugin-details-heading h2 { margin: 0 0 4px; font-size: 19px; }
.plugin-details-meta { font-size: 12px; color: rgba(var(--text-color), .55); }
.plugin-tabs { display: flex; flex: 0 0 auto; gap: 6px; padding-bottom: 10px; border-bottom: 1px solid rgba(var(--outline-color), .12); }
.plugin-tab { flex: 1; border: 0; border-radius: 8px; padding: 9px 12px; color: rgba(var(--text-color), .6); background: transparent; cursor: pointer; font-size: 13px; }
.plugin-tab.active { color: rgb(var(--text-color)); background: rgba(var(--primary-color), .16); }
.plugin-tab:focus-visible { outline: 2px solid rgba(var(--primary-color), .6); }
.plugin-tab-panel { flex: 1; min-height: 0; overflow-y: auto; }
.plugin-details-description { margin: 0 0 18px; font-size: 13px; line-height: 1.7; color: rgba(var(--text-color), .75); }
.plugin-details-facts { margin: 0; display: grid; gap: 12px; font-size: 12px; }
.plugin-details-facts > div { display: grid; grid-template-columns: 80px minmax(0, 1fr); gap: 12px; }
.plugin-details-facts dt { color: rgba(var(--text-color), .5); }
.plugin-details-facts dd { margin: 0; display: flex; flex-wrap: wrap; gap: 5px; overflow-wrap: anywhere; }
.permission-chip { padding: 3px 7px; border-radius: 5px; background: rgba(var(--outline-color), .08); }
.plugin-wiki-state { padding: 35px 0; color: rgba(var(--text-color), .55); text-align: center; font-size: 13px; }

.plugin-heading {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 10px;
}

.plugin-heading h3 {
  margin: 0;
  font-size: 14px;
  color: rgb(var(--text-color));
}

.plugin-heading span,
.plugin-info p {
  color: rgba(var(--text-color), 0.58);
  font-size: 11px;
}

.plugin-info p {
  margin: 3px 0 6px;
  font-size: 12px;
}

.permission-row {
  display: flex;
  gap: 5px;
  flex-wrap: wrap;
}

.permission-row span {
  padding: 2px 6px;
  border-radius: 999px;
  color: rgba(var(--text-color), 0.52);
  background: rgba(var(--outline-color), 0.08);
  font-size: 10.5px;
}

.permission-row span.granted {
  color: rgb(var(--text-color));
  background: rgba(76, 175, 80, 0.16);
}

.plugin-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 5px;
  flex: 0 0 auto;
}

.action-btn.enable {
  background: rgba(76, 175, 80, 0.16);
}

.action-btn.uninstall {
  color: #f44336;
  background: rgba(244, 67, 54, 0.1);
}

.empty-state,
.error {
  padding: 24px 0;
  text-align: center;
  color: rgba(var(--text-color), 0.6);
}

.fault-error {
  margin-top: 6px;
  color: #e05b5b;
  font-size: 11px;
  word-break: break-all;
}

.error {
  padding: 8px;
  color: #f44336;
}

@media (max-width: 768px) {
  .toolbar,
  .plugin-card {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .plugin-info {
    flex-basis: calc(100% - 48px);
  }

  .plugin-actions {
    width: 100%;
    justify-content: flex-start;
  }
}

</style>
