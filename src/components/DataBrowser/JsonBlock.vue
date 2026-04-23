<template>
  <pre
    class="bg-surface-dark border border-surface-card p-2 text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed rounded"
    :class="compact ? 'text-[10px]' : 'text-xs'"
  >{{ formatted }}</pre>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  value: unknown;
  compact?: boolean;
}>();

const formatted = computed(() => {
  if (props.value === null || props.value === undefined) return "null";
  if (typeof props.value === "string") return props.value;
  return JSON.stringify(props.value, null, 2);
});
</script>
