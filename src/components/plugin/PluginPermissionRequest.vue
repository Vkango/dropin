<template>
  <div class="permission-request">
    <p v-if="description" class="permission-description">{{ description }}</p>

    <div v-if="permissionItems.length" class="permission-list">
      <label
        v-for="permission in permissionItems"
        :key="permission.key"
        class="permission-option"
        :class="{ granted: selectedPermissions.includes(permission.key) }"
      >
        <span class="permission-check">
          <input v-model="selectedPermissions" type="checkbox" :value="permission.key" />
          <span aria-hidden="true"></span>
        </span>
        <span class="permission-copy">
          <strong>{{ permission.label }}</strong>
          <small>{{ permission.description }}</small>
          <code>{{ permission.key }}</code>
        </span>
      </label>
    </div>

    <p v-else class="permission-empty">{{ t('plugins.noPermissions') }}</p>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from '@/i18n/index.js'
import { pluginPermissionMeta } from '@/utils/pluginPermissions.js'

const props = defineProps({
  modelValue: {
    type: Array,
    default: () => []
  },
  permissions: {
    type: Array,
    default: () => []
  },
  description: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:modelValue'])
const { t } = useI18n()

const selectedPermissions = computed({
  get: () => props.modelValue,
  set: (value) => {
    emit('update:modelValue', Array.isArray(value) ? value : [])
  }
})

const permissionItems = computed(() =>
  props.permissions.map((permission) => pluginPermissionMeta(permission, t))
)
</script>

<style scoped>
.permission-request {
  display: grid;
  gap: 12px;
}

.permission-description,
.permission-empty {
  margin: 0;
  color: rgba(var(--text-color), 0.64);
  font-size: 13px;
  line-height: 1.5;
}

.permission-list {
  display: grid;
  gap: 8px;
}

.permission-option {
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr);
  gap: 10px;
  padding: 11px;
  border: 1px solid rgba(var(--outline-color), 0.12);
  border-radius: 15px;
  background: rgba(var(--outline-color), 0.06);
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease,
    transform 0.2s ease;
}

.permission-option:hover {
  transform: translateY(-1px);
  background: rgba(var(--primary-color), 0.08);
  border-color: rgba(var(--primary-color), 0.18);
}

.permission-option.granted {
  background: rgba(var(--primary-color), 0.12);
  border-color: rgba(var(--primary-color), 0.24);
}

.permission-check {
  position: relative;
  width: 20px;
  height: 20px;
  margin-top: 1px;
}

.permission-check input {
  position: absolute;
  inset: 0;
  z-index: 1;
  width: 100%;
  height: 100%;
  margin: 0;
  opacity: 0;
  cursor: pointer;
}

.permission-check span {
  display: block;
  width: 20px;
  height: 20px;
  border: 1px solid rgba(var(--text-color), 0.32);
  border-radius: 7px;
  background: rgba(var(--surface-color), 0.72);
}

.permission-check input:checked + span {
  border-color: rgb(var(--primary-color));
  background: rgb(var(--primary-color));
  box-shadow: inset 0 0 0 4px rgba(var(--surface-color), 0.92);
}

.permission-copy {
  display: grid;
  min-width: 0;
  gap: 3px;
}

.permission-copy strong {
  color: rgb(var(--text-color));
  font-size: 13px;
  font-weight: 700;
}

.permission-copy small {
  color: rgba(var(--text-color), 0.56);
  font-size: 11.5px;
  line-height: 1.45;
}

.permission-copy code {
  width: fit-content;
  margin-top: 2px;
  padding: 2px 6px;
  border-radius: 7px;
  color: rgba(var(--text-color), 0.52);
  background: rgba(var(--outline-color), 0.09);
  font-size: 10.5px;
  font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
}
</style>
