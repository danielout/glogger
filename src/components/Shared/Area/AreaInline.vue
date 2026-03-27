<template>
  <span class="inline-flex items-center text-entity-area text-xs font-medium border-b border-dotted border-entity-area/30">
    {{ reference }}
  </span>
</template>

<script setup lang="ts">
defineProps<{
  reference: string;
}>();
</script>
