<template>
  <div class="flex items-center gap-4 px-3 py-2 shrink-0 border-b border-border-default">
    <h3 class="text-lg text-[#7ec8e3] m-0 shrink-0">Survey Analytics</h3>

    <!-- Inline overview stats -->
    <div v-if="speedStats" class="flex items-center gap-4 text-xs font-mono flex-1 min-w-0">
      <span class="text-text-muted">
        <span class="text-text-primary font-bold">{{ speedStats.total_surveys }}</span> surveys
      </span>
      <span class="text-text-muted">
        <span class="text-[#c8b47e] font-bold">{{ speedStats.speed_bonus_count }}</span> bonuses
        <span class="text-[#c8b47e] font-semibold">({{ speedStats.speed_bonus_rate.toFixed(1) }}%)</span>
      </span>
      <span class="text-text-muted">
        <span class="text-text-primary font-bold">{{ speedStats.total_bonus_items }}</span> bonus items
      </span>
      <span class="text-text-muted">
        <span class="text-text-primary font-bold">{{ zoneCount }}</span> zones
      </span>
      <span v-if="includeImports && hasImports" class="text-[0.6rem] text-accent-gold uppercase tracking-wider">
        combined data
      </span>
    </div>

    <!-- Controls -->
    <div class="flex items-center gap-2 shrink-0">
      <div v-if="hasImports" class="flex rounded border border-border-default overflow-hidden text-xs">
        <button
          @click="$emit('update:includeImports', false)"
          :class="[
            'px-2.5 py-1 transition-all',
            !includeImports
              ? 'bg-accent-gold/20 text-accent-gold border-r border-accent-gold/30'
              : 'bg-surface-elevated text-text-muted hover:text-text-secondary border-r border-border-default'
          ]"
        >My Data</button>
        <button
          @click="$emit('update:includeImports', true)"
          :class="[
            'px-2.5 py-1 transition-all',
            includeImports
              ? 'bg-accent-gold/20 text-accent-gold'
              : 'bg-surface-elevated text-text-muted hover:text-text-secondary'
          ]"
        >All Data</button>
      </div>

      <button @click="$emit('export')" :disabled="exporting"
        class="px-2.5 py-1 text-xs bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all"
        title="Export your survey data to share with others">
        {{ exporting ? "Exporting..." : "Export" }}
      </button>
      <button @click="$emit('import')" :disabled="importing"
        class="px-2.5 py-1 text-xs bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all"
        title="Import survey data from another player">
        {{ importing ? "Importing..." : "Import" }}
      </button>
      <button v-if="hasImports" @click="$emit('manage-imports')"
        class="px-2.5 py-1 text-xs bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all"
        title="Manage imported data sets">
        Imports ({{ importCount }})
      </button>
      <button @click="$emit('refresh')" :disabled="loading"
        class="px-2.5 py-1 text-xs bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all">
        {{ loading ? "Loading..." : "Refresh" }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SpeedBonusStats } from "../../../types/database";

defineProps<{
  speedStats: SpeedBonusStats | null;
  zoneCount: number;
  includeImports: boolean;
  hasImports: boolean;
  importCount: number;
  loading: boolean;
  exporting: boolean;
  importing: boolean;
}>();

defineEmits<{
  "update:includeImports": [value: boolean];
  export: [];
  import: [];
  "manage-imports": [];
  refresh: [];
}>();
</script>
