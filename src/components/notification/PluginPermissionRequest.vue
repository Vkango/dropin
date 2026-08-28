<template>
  <div class="permission-request">
    <p>{{ description }}</p>
    <label v-for="permission in permissions" :key="permission" class="permission-option"><input v-model="granted" type="checkbox" :value="permission" /><span>{{ permission }}</span></label>
    <div class="permission-actions"><button type="button" @click="dismiss">{{ cancelLabel }}</button><button type="button" class="primary" @click="save">{{ saveLabel }}</button></div>
  </div>
</template>
<script setup>
import { ref } from 'vue'
const props = defineProps({ permissions: { type: Array, default: () => [] }, selected: { type: Array, default: () => [] }, description: { type: String, default: '' }, cancelLabel: { type: String, default: 'Cancel' }, saveLabel: { type: String, default: 'Save' }, onSave: { type: Function, required: true }, onDismiss: { type: Function, required: true } })
const granted = ref([...props.selected])
const save = () => props.onSave([...granted.value])
const dismiss = () => props.onDismiss()
</script>
<style scoped>
.permission-request p { margin: 0 0 8px; }.permission-option { display: flex; align-items: center; gap: 8px; padding: 4px 0; }.permission-option input { accent-color: rgb(var(--primary-color)); }.permission-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 10px; }.permission-actions button { border: 0; border-radius: 5px; padding: 5px 9px; color: rgb(var(--text-color)); background: rgba(var(--outline-color), .12); cursor: pointer; }.permission-actions button.primary { background: rgba(var(--primary-color), .22); }
</style>
