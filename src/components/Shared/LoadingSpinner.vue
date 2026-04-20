<template>
  <span
    class="inline-block rounded-full animate-spin border-accent-gold/40 border-t-accent-gold"
    :class="sizeClasses" />
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{
  /** xs=8px, sm=12px, md=24px, lg=40px */
  size?: "xs" | "sm" | "md" | "lg"
}>(), {
  size: "sm",
});

const sizeClasses = computed(() => {
  switch (props.size) {
    case "xs": return "w-2 h-2 border";
    case "sm": return "w-3 h-3 border-2";
    case "md": return "w-6 h-6 border-2";
    case "lg": return "w-10 h-10 border-3";
  }
});
</script>
