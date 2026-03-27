<template>
  <button
    class="bg-bg-secondary border rounded p-3 text-left cursor-pointer transition-all hover:bg-bg-tertiary"
    :class="selected ? 'border-accent-gold/50 bg-bg-tertiary' : 'border-border-primary'"
    @click="emit('select')"
  >
    <!-- Area name -->
    <div class="text-sm font-medium text-text-primary mb-1 truncate">
      {{ areaName }}
    </div>

    <!-- Stats row -->
    <div class="flex items-center justify-between text-xs text-text-muted mb-2">
      <span>{{ vaults.length }} vaults</span>
      <span v-if="totalUsed > 0" class="text-text-secondary">
        {{ totalUsed }} items stored
      </span>
    </div>

    <!-- Capacity info -->
    <div v-if="totalMaxPossible" class="flex flex-col gap-1">
      <!-- Capacity bar (based on unlocked slots if known, else max possible) -->
      <div class="flex items-center gap-2">
        <div class="flex-1 h-1.5 bg-bg-primary rounded-full overflow-hidden">
          <div
            class="h-full rounded-full transition-all"
            :class="capacityColor"
            :style="{ width: capacityPercent + '%' }"
          />
        </div>
        <span class="text-[0.65rem] text-text-secondary whitespace-nowrap">
          {{ totalUsed }} / {{ totalUnlocked ?? '?' }}
        </span>
      </div>
      <!-- Max possible note -->
      <div v-if="totalUnlocked != null && totalUnlocked < totalMaxPossible" class="text-[0.6rem] text-text-muted">
        {{ totalMaxPossible }} max possible
      </div>
    </div>
    <div v-else class="h-1.5" />
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { VaultEntry } from "./VaultDatabaseTab.vue";

const props = defineProps<{
  areaName: string;
  areaKey: string | null;
  vaults: VaultEntry[];
  totalUsed: number;
  totalUnlocked: number | null;
  totalMaxPossible: number | null;
  searchQuery: string;
  selected: boolean;
}>();

const emit = defineEmits<{
  select: [];
}>();

const capacityPercent = computed(() => {
  const cap = props.totalUnlocked ?? props.totalMaxPossible;
  if (!cap) return 0;
  return Math.min(100, (props.totalUsed / cap) * 100);
});

const capacityColor = computed(() => {
  const pct = capacityPercent.value;
  if (pct >= 90) return "bg-red-500";
  if (pct >= 70) return "bg-yellow-500";
  return "bg-accent-primary";
});
</script>
