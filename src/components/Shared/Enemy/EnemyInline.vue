<template>
  <span class="inline-flex items-center text-entity-enemy text-xs font-medium border-b border-dotted border-entity-enemy/30">
    {{ reference }}
  </span>
</template>

<script setup lang="ts">
defineProps<{
  reference: string;
}>();
</script>
