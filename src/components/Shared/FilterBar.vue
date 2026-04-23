<template>
  <div class="flex items-center gap-3 flex-wrap">
    <div class="relative">
      <input
        :value="modelValue"
        type="text"
        :placeholder="placeholder"
        class="input text-sm w-48 pr-7"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)" />
      <button
        v-if="modelValue"
        type="button"
        class="absolute right-1.5 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary text-sm leading-none px-1"
        @click="$emit('update:modelValue', '')">
        &times;
      </button>
    </div>

    <slot />

    <span v-if="resultCount !== undefined" class="text-xs text-text-muted">
      {{ resultCount.toLocaleString() }} {{ resultLabel }}
    </span>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  modelValue: string
  placeholder?: string
  resultCount?: number
  resultLabel?: string
}>(), {
  placeholder: 'Search...',
  resultLabel: 'results',
})

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>
