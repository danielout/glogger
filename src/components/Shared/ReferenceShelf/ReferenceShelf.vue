<template>
  <div
    v-if="shelf.pins.length > 0"
    class="shrink-0 bg-surface-base border-t border-border-default"
  >
    <!-- Collapsed: just a thin bar with count -->
    <div
      v-if="shelf.collapsed"
      class="flex items-center justify-between px-3 h-7 cursor-pointer hover:bg-surface-raised/50 transition-colors"
      @click="shelf.toggleCollapsed()"
    >
      <div class="flex items-center gap-1.5 text-text-muted text-[10px]">
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2L12 12M12 12L8 8M12 12L16 8" />
          <rect x="3" y="14" width="18" height="7" rx="1" />
        </svg>
        <span>{{ shelf.pins.length }} pinned</span>
      </div>
      <svg class="w-3.5 h-3.5 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="18 15 12 9 6 15" />
      </svg>
    </div>

    <!-- Expanded: chip row -->
    <div v-else class="flex items-start gap-1 px-3 py-1.5 flex-wrap">
      <ShelfChip
        v-for="pin in shelf.pins"
        :key="`${pin.type}:${pin.reference}`"
        :pin="pin"
      />
      <button
        class="ml-auto shrink-0 self-center text-text-muted hover:text-text-default transition-colors p-0.5"
        title="Collapse shelf"
        @click="shelf.toggleCollapsed()"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useReferenceShelfStore } from "../../../stores/referenceShelfStore";
import ShelfChip from "./ShelfChip.vue";

const shelf = useReferenceShelfStore();

onMounted(() => {
  shelf.load();
});
</script>
