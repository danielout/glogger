<template>
  <div class="bg-surface-dark border border-border-default rounded-lg p-3 overflow-y-auto">
    <div class="text-[0.65rem] uppercase tracking-widest text-text-dim mb-2 font-bold">Activity Log</div>
    <div v-if="store.log.length === 0" class="text-text-dim italic text-xs">No events yet.</div>
    <div
      v-for="(entry, i) in store.log"
      :key="i"
      class="px-2 py-1 border-l-3 border-border-light mb-1 text-xs"
      :style="{ borderLeftColor: kindColor[entry.kind] }">
      <div class="flex items-baseline gap-2">
        <span class="text-text-dim text-[0.65rem] shrink-0">{{ formatTs(entry.timestamp) }}</span>
        <span class="shrink-0">{{ kindIcon[entry.kind] }}</span>
        <span class="text-text-primary/75">{{ entry.label }}</span>
      </div>
      <div v-if="entry.detail" class="text-text-secondary text-xs mt-0.5 pl-12">{{ entry.detail }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useFarmingStore } from "../../stores/farmingStore";
import { formatAnyTimestamp as formatTs } from "../../composables/useTimestamp";
import type { FarmingLogKind } from "../../types/farming";

const store = useFarmingStore();

const kindIcon: Record<FarmingLogKind, string> = {
  "session-start": "\u25B6",
  "item-gained": "\u{1F4E6}",
  "item-lost": "\u274C",
  "xp-gain": "\u2B50",
  "level-up": "\u{1F3C6}",
  "favor-change": "\u2764",
  "vendor-sale": "\u{1F4B0}",
  "session-end": "\u23F9",
};

const kindColor: Record<FarmingLogKind, string> = {
  "session-start": "#7ec8e3",
  "item-gained": "#7ec87e",
  "item-lost": "#c87e7e",
  "xp-gain": "#c8b47e",
  "level-up": "#d4af37",
  "favor-change": "#c87ec8",
  "vendor-sale": "#d4af37",
  "session-end": "#7ec8e3",
};
</script>
