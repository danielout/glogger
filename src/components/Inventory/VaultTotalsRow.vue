<template>
  <div class="bg-bg-tertiary border border-border-secondary rounded">
    <!-- Summary row (clickable to expand) -->
    <button
      class="w-full flex items-center justify-between px-3 py-1.5 bg-transparent border-none cursor-pointer text-left hover:bg-bg-secondary transition-colors"
      @click="expanded = !expanded"
    >
      <div class="flex items-center gap-2 min-w-0">
        <span class="text-xs text-text-muted shrink-0 transition-transform w-3" :class="{ 'rotate-90': expanded }">&#9654;</span>
        <ItemInline :reference="itemName" />
      </div>
      <div class="flex items-center gap-3 shrink-0">
        <span class="text-xs text-text-muted">
          {{ locations.length }} {{ locations.length === 1 ? 'location' : 'locations' }}
        </span>
        <span class="text-sm text-text-primary font-medium tabular-nums">
          x{{ totalQuantity.toLocaleString() }}
        </span>
      </div>
    </button>

    <!-- Expanded per-location breakdown -->
    <div v-if="expanded" class="px-3 pb-2 pt-0.5 flex flex-col gap-0.5">
      <div
        v-for="loc in locations"
        :key="loc.vaultKey"
        class="flex items-center justify-between py-0.5 px-3 text-xs hover:bg-bg-secondary rounded ml-3"
      >
        <span class="text-text-secondary">{{ loc.displayName }}</span>
        <span class="text-text-secondary tabular-nums">x{{ loc.quantity.toLocaleString() }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import ItemInline from "../Shared/Item/ItemInline.vue";

export interface TotalsLocation {
  vaultKey: string;
  displayName: string;
  quantity: number;
}

defineProps<{
  itemName: string;
  totalQuantity: number;
  locations: TotalsLocation[];
}>();

const expanded = ref(false);
</script>
