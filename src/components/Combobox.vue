<template>
  <div ref="rootRef" class="combobox" :class="{ 'is-open': isOpen, 'is-disabled': disabled }">
    <div class="combobox-control">
      <div v-if="!isOpen && selectedOption" class="combobox-selected-value" aria-hidden="true">
        <slot name="selected" :option="selectedOption.option" :value="selectedOption.value"
          :label="selectedOption.label" :selected="true" :active="false" :disabled="selectedOption.disabled">
          <span>{{ selectedOption.label }}</span>
        </slot>
      </div>
      <input ref="inputRef" class="combobox-input" :id="inputId" :value="isOpen ? searchQuery : selectedLabel"
        :placeholder="placeholder" :disabled="disabled" role="combobox" aria-autocomplete="list" :aria-expanded="isOpen"
        :aria-controls="listId" :aria-activedescendant="activeDescendantId" :aria-valuetext="selectedLabel || undefined"
        autocomplete="off" @focus="openDropdown" @input="handleInput" @keydown="handleKeydown" />
      <button class="combobox-trigger" type="button" tabindex="-1" :disabled="disabled"
        :aria-label="isOpen ? '关闭选项' : '打开选项'" @mousedown.prevent @click="toggleDropdown">
        <span class="combobox-chevron" aria-hidden="true"></span>
      </button>
    </div>

    <AnimatePresence :initial="false">
      <MotionDiv v-if="isOpen" :id="listId" key="combobox-menu" class="combobox-menu" role="listbox"
        :aria-labelledby="inputId" :initial="{ opacity: 0, y: -5, scale: 0.98 }"
        :animate="{ opacity: 1, y: 0, scale: 1 }" :exit="{ opacity: 0, y: -4, scale: 0.98 }"
        :transition="menuTransition">
        <div v-if="filteredOptions.length" class="combobox-options">
          <div v-for="(item, index) in filteredOptions" :id="optionId(item)" :key="item.key" class="combobox-option"
            :class="{
              'is-active': index === activeIndex,
              'is-selected': isSelected(item),
              'is-disabled': item.disabled
            }" role="option" :aria-selected="isSelected(item)" :aria-disabled="item.disabled || undefined"
            @mousedown.prevent @mouseenter="setActiveIndex(index)" @click="selectOption(item)">
            <slot name="option" :option="item.option" :value="item.value" :label="item.label"
              :selected="isSelected(item)" :active="index === activeIndex" :disabled="item.disabled">
              <span>{{ item.label }}</span>
            </slot>
          </div>
        </div>
        <div v-else class="combobox-empty">
          <slot name="empty">无匹配项</slot>
        </div>
      </MotionDiv>
    </AnimatePresence>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { AnimatePresence, motion, useReducedMotion } from 'motion-v'
import { APPLE_SPRING, INSTANT_MOTION } from '../utils/motion.js'

let nextComboboxId = 0

const props = defineProps({
  modelValue: {
    default: undefined
  },
  options: {
    type: Array,
    default: () => []
  },
  valueKey: {
    type: [String, Function],
    default: undefined
  },
  labelKey: {
    type: [String, Function],
    default: undefined
  },
  disabledKey: {
    type: [String, Function],
    default: undefined
  },
  placeholder: {
    type: String,
    default: '请选择'
  },
  disabled: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'change'])
const rootRef = ref(null)
const inputRef = ref(null)
const isOpen = ref(false)
const searchQuery = ref('')
const activeIndex = ref(-1)
const componentId = ++nextComboboxId
const inputId = `combobox-input-${componentId}`
const listId = `combobox-list-${componentId}`
const reducedMotion = useReducedMotion()
const MotionDiv = motion.div

const menuTransition = computed(() => reducedMotion.value ? INSTANT_MOTION : APPLE_SPRING)

const resolveField = (option, key, fallback) => {
  if (typeof key === 'function') return key(option)
  if (typeof key === 'string') return option?.[key]
  return fallback()
}

const normalizedOptions = computed(() => props.options.map((option, index) => {
  const isObject = option !== null && typeof option === 'object'
  const value = resolveField(option, props.valueKey, () => (
    isObject && Object.prototype.hasOwnProperty.call(option, 'value') ? option.value : option
  ))
  const label = resolveField(option, props.labelKey, () => (
    isObject && Object.prototype.hasOwnProperty.call(option, 'label')
      ? option.label
      : option == null ? '' : String(option)
  ))
  const disabled = Boolean(resolveField(option, props.disabledKey, () => (
    isObject && Object.prototype.hasOwnProperty.call(option, 'disabled') ? option.disabled : false
  )))

  return {
    option,
    value,
    label: label == null ? '' : String(label),
    disabled,
    key: `${componentId}-${index}`
  }
}))

const selectedOption = computed(() => normalizedOptions.value.find((item) => isEqual(item.value, props.modelValue)))
const selectedLabel = computed(() => selectedOption.value?.label || '')
const filteredOptions = computed(() => {
  const query = searchQuery.value.trim().toLocaleLowerCase()
  if (!query) return normalizedOptions.value
  return normalizedOptions.value.filter((item) => item.label.toLocaleLowerCase().includes(query))
})
const activeDescendantId = computed(() => {
  const item = filteredOptions.value[activeIndex.value]
  return item ? optionId(item) : undefined
})

const isEqual = (left, right) => Object.is(left, right)
const isSelected = (item) => isEqual(item.value, props.modelValue)
const optionId = (item) => `combobox-option-${item.key}`

const firstEnabledIndex = (items = filteredOptions.value) => items.findIndex((item) => !item.disabled)

const setActiveIndex = (index) => {
  if (!filteredOptions.value[index]?.disabled) activeIndex.value = index
}

const openDropdown = async () => {
  if (props.disabled) return
  if (!isOpen.value) {
    isOpen.value = true
    searchQuery.value = ''
    const selectedIndex = filteredOptions.value.findIndex((item) => isSelected(item))
    activeIndex.value = selectedIndex >= 0 && !filteredOptions.value[selectedIndex].disabled
      ? selectedIndex
      : firstEnabledIndex()
    await nextTick()
    inputRef.value?.select()
  }
}

const closeDropdown = () => {
  if (!isOpen.value) return
  isOpen.value = false
  searchQuery.value = ''
  activeIndex.value = -1
}

const toggleDropdown = async () => {
  if (props.disabled) return
  if (isOpen.value) {
    closeDropdown()
    inputRef.value?.focus()
  } else {
    await openDropdown()
    inputRef.value?.focus()
  }
}

const handleInput = (event) => {
  if (!isOpen.value) isOpen.value = true
  searchQuery.value = event.target.value
  activeIndex.value = firstEnabledIndex()
}

const selectOption = (item) => {
  if (item.disabled) return
  emit('update:modelValue', item.value)
  emit('change', item.value, item.option)
  closeDropdown()
  nextTick(() => inputRef.value?.focus())
}

const moveActive = (direction) => {
  const items = filteredOptions.value
  if (!items.length) return
  let nextIndex = activeIndex.value
  for (let offset = 0; offset < items.length; offset += 1) {
    nextIndex = (nextIndex + direction + items.length) % items.length
    if (!items[nextIndex].disabled) {
      activeIndex.value = nextIndex
      return
    }
  }
}

const handleKeydown = async (event) => {
  if (event.key === 'Tab') {
    closeDropdown()
    return
  }

  if (event.key === 'Escape') {
    if (isOpen.value) {
      event.preventDefault()
      closeDropdown()
    }
    return
  }

  if (!isOpen.value && ['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
    event.preventDefault()
    await openDropdown()
    return
  }

  if (!isOpen.value) return

  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    moveActive(event.key === 'ArrowDown' ? 1 : -1)
  } else if (event.key === 'Home' || event.key === 'End') {
    event.preventDefault()
    const items = filteredOptions.value
    const index = event.key === 'Home' ? firstEnabledIndex(items) : items.findLastIndex((item) => !item.disabled)
    if (index >= 0) activeIndex.value = index
  } else if (event.key === 'Enter') {
    event.preventDefault()
    const item = filteredOptions.value[activeIndex.value]
    if (item) selectOption(item)
  }
}

const handleDocumentPointerDown = (event) => {
  if (isOpen.value && !rootRef.value?.contains(event.target)) closeDropdown()
}

watch(filteredOptions, (items) => {
  if (!items[activeIndex.value] || items[activeIndex.value].disabled) {
    activeIndex.value = firstEnabledIndex(items)
  }
})

watch(() => props.disabled, (value) => {
  if (value) closeDropdown()
})

onMounted(() => document.addEventListener('pointerdown', handleDocumentPointerDown))
onBeforeUnmount(() => document.removeEventListener('pointerdown', handleDocumentPointerDown))
</script>

<style scoped>
.combobox {
  position: relative;
  min-width: 160px;
  color: rgb(var(--text-color));
  font-size: 14px;
}

.combobox-control {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  min-height: 38px;
  overflow: hidden;
  border: 1px solid rgba(var(--outline-color), 0.2);
  border-radius: 8px;
  background: rgba(var(--surface-color), 0.1);
  transition: border-color 160ms ease, background-color 160ms ease;
}

.combobox.is-open .combobox-control,
.combobox-control:focus-within {
  border-color: rgba(var(--primary-color), 0.6);
  background: rgba(var(--surface-color), 0.18);
}

.combobox-input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: 0;
  padding: 8px 4px 8px 12px;
  background: transparent;
  color: inherit;
  font: inherit;
}

.combobox-selected-value {
  position: absolute;
  right: 34px;
  left: 0;
  overflow: hidden;
  padding: 8px 4px 8px 12px;
  white-space: nowrap;
  text-overflow: ellipsis;
  pointer-events: none;
}

.combobox:not(.is-open) .combobox-input {
  color: transparent;
  caret-color: transparent;
}

.combobox-input::placeholder {
  color: rgba(var(--text-color), 0.5);
}

.combobox-trigger {
  display: grid;
  width: 34px;
  height: 36px;
  flex: 0 0 34px;
  place-items: center;
  border: 0;
  background: transparent;
  color: rgba(var(--text-color), 0.65);
  cursor: pointer;
}

.combobox-chevron {
  width: 7px;
  height: 7px;
  border-right: 1.5px solid currentColor;
  border-bottom: 1.5px solid currentColor;
  transform: translateY(-2px) rotate(45deg);
  transition: transform 180ms ease;
}

.combobox.is-open .combobox-chevron {
  transform: translateY(2px) rotate(225deg);
}

.combobox-menu {
  position: absolute;
  z-index: 30;
  top: calc(100% + 6px);
  right: 0;
  left: 0;
  overflow: hidden;
  border: 1px solid rgba(var(--outline-color), 0.22);
  border-radius: 10px;
  background: rgb(var(--surface-color));
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.22);
  transform-origin: top center;
}

.combobox-options {
  max-height: 280px;
  overflow-y: auto;
  padding: 4px;
}

.combobox-option {
  min-height: 34px;
  display: flex;
  align-items: center;
  border-radius: 6px;
  padding: 7px 10px;
  color: rgb(var(--text-color));
  cursor: pointer;
  user-select: none;
}

.combobox-option.is-active {
  background: rgba(var(--primary-color), 0.12);
}

.combobox-option.is-selected {
  color: rgb(var(--text-color));
  font-weight: 600;
}

.combobox-option.is-disabled {
  opacity: 0.42;
  cursor: not-allowed;
}

.combobox-empty {
  padding: 12px 10px;
  color: rgba(var(--text-color), 0.55);
  text-align: center;
}

.combobox.is-disabled {
  opacity: 0.55;
}

.combobox.is-disabled .combobox-control,
.combobox.is-disabled .combobox-trigger {
  cursor: not-allowed;
}
</style>
