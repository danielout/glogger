<template>
  <div class="relative" ref="root">
    <div
      class="flex items-center gap-1 px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm cursor-pointer focus-within:border-accent-gold/50"
      :class="{ 'border-accent-gold/50': open }"
      @click="toggle">
      <input
        ref="inputEl"
        v-model="query"
        type="text"
        :placeholder="modelValue || placeholder"
        class="flex-1 bg-transparent text-text-primary placeholder-text-muted outline-none min-w-0"
        :class="{ 'placeholder-text-primary': modelValue && !open }"
        autocomplete="off"
        @focus="open = true"
        @input="open = true"
        @keydown.down.prevent="highlightedIndex = Math.min(highlightedIndex + 1, filtered.length - 1)"
        @keydown.up.prevent="highlightedIndex = Math.max(highlightedIndex - 1, 0)"
        @keydown.enter.prevent="selectHighlighted"
        @keydown.escape="close" />
      <button
        v-if="modelValue"
        class="text-text-dim hover:text-text-primary text-xs shrink-0"
        @click.stop="clear">
        &times;
      </button>
      <span class="text-text-dim text-xs shrink-0">&#9662;</span>
    </div>
    <ul
      v-if="open && filtered.length > 0"
      class="absolute z-10 left-0 right-0 top-full mt-0.5 bg-surface-card border border-border-default rounded shadow-lg max-h-48 overflow-y-auto list-none m-0 p-0">
      <li
        v-for="(option, idx) in filtered"
        :key="option"
        class="px-3 py-1.5 text-sm cursor-pointer hover:bg-surface-elevated"
        :class="{ 'bg-surface-elevated': idx === highlightedIndex }"
        @mousedown.prevent="select(option)">
        <span class="text-text-primary">{{ option }}</span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'

const props = defineProps<{
  modelValue: string
  options: string[]
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const root = ref<HTMLElement | null>(null)
const inputEl = ref<HTMLInputElement | null>(null)
const open = ref(false)
const query = ref('')
const highlightedIndex = ref(0)

const filtered = computed(() => {
  const q = query.value.toLowerCase()
  if (!q) return props.options
  return props.options.filter(o => o.toLowerCase().includes(q))
})

watch(query, () => {
  highlightedIndex.value = 0
})

function toggle() {
  if (!open.value) {
    open.value = true
    query.value = ''
    inputEl.value?.focus()
  }
}

function select(option: string) {
  emit('update:modelValue', option)
  query.value = ''
  open.value = false
}

function selectHighlighted() {
  if (filtered.value.length > 0) {
    select(filtered.value[highlightedIndex.value])
  }
}

function clear() {
  emit('update:modelValue', '')
  query.value = ''
}

function close() {
  open.value = false
  query.value = ''
}

function onClickOutside(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) {
    close()
  }
}

onMounted(() => document.addEventListener('mousedown', onClickOutside))
onBeforeUnmount(() => document.removeEventListener('mousedown', onClickOutside))
</script>
