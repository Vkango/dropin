<template>
    <button type="button" class="group-label" :data-group-initial="label" @click="$emit('click', label)">
        {{ label }}
    </button>
</template>

<script setup>
defineProps({
    label: {
        type: String,
        required: true
    }
})

defineEmits(['click'])
</script>

<style scoped>
.group-label {
    display: block;
    width: fit-content;
    margin: 20px 0 10px 16px;
    scroll-margin-top: 72px;
    padding: 6px 10px;
    border: 0;
    border-radius: 10px;
    background: rgba(var(--surface-color), 0.5);
    color: rgb(var(--text-color));
    font-family: inherit;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
}

.group-label:hover,
.group-label:focus-visible {
    background: rgba(var(--primary-color), 0.2);
}

.group-label:focus-visible {
    outline: 2px solid rgba(var(--primary-color), 0.6);
    outline-offset: 2px;
}
</style>
