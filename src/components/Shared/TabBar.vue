<template>
  <div class="flex border-b border-border-default">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ 'tab-active': modelValue === tab.id }"
      @click="$emit('update:modelValue', tab.id)">
      {{ tab.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
export interface Tab {
  id: string;
  label: string;
}

defineProps<{
  tabs: Tab[];
  modelValue: string;
}>();

defineEmits<{
  "update:modelValue": [value: string];
}>();
</script>
