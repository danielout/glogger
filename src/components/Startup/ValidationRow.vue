<template>
  <div class="flex items-center gap-2 text-sm">
    <span v-if="ok" class="text-accent-green">&#10003;</span>
    <span v-else class="text-text-muted">&#10007;</span>
    <span :class="ok ? 'text-text-primary' : 'text-text-muted'">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  ok: boolean;
  label: string;
}>();
</script>
