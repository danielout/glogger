<template>
  <img
    v-if="iconSrc"
    :src="iconSrc"
    :alt="alt"
    :class="[sizeClass, isInline ? 'shrink-0 rounded-sm object-contain' : 'shrink-0 rounded-sm object-contain bg-black/30 border border-border-light']"
    loading="lazy" />
  <div
    v-else
    :class="[sizeClass, isInline
      ? 'shrink-0 rounded-sm inline-flex items-center justify-center text-text-muted'
      : 'shrink-0 rounded-sm flex items-center justify-center bg-black/50 border border-border-light text-text-muted']"
  >
    <span v-if="iconLoading" class="animate-spin text-[0.6em]">&#x27F3;</span>
    <span v-else class="text-[0.6em]">?</span>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { useGameIcon } from "../../composables/useGameIcon";

const props = defineProps<{
  iconId: number | null | undefined;
  alt?: string;
  size?: "xs" | "sm" | "md" | "lg" | "inline";
}>();

const { iconSrc, iconLoading, loadIcon } = useGameIcon();

const isInline = computed(() => props.size === "inline");

const sizeClasses: Record<string, string> = {
  xs: "w-4 h-4",
  sm: "w-5 h-5",
  md: "w-8 h-8",
  lg: "w-12 h-12",
  inline: "w-[1.1em] h-[1.1em]",
};

const sizeClass = computed(() => sizeClasses[props.size ?? "sm"]);

watch(() => props.iconId, (id) => loadIcon(id), { immediate: true });
</script>
