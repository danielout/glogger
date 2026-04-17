<template>
  <div :class="!embedded && 'bg-surface-dark border border-border-default rounded-lg p-3 max-h-100 overflow-y-auto'">
    <div v-if="!embedded" class="text-[0.7rem] uppercase tracking-widest text-text-dim mb-2">Activity Log</div>
    <div v-if="store.log.length === 0" class="text-text-dim italic text-sm">No events yet.</div>
    <div
      v-for="(entry, i) in store.log"
      :key="i"
      class="px-2 py-1 border-l-3 border-border-light mb-1 text-xs"
      :style="{ borderLeftColor: kindColor[entry.kind] }">
      <div class="flex items-baseline gap-2">
        <span class="text-text-dim text-xs shrink-0">{{ formatTs(entry.timestamp) }}</span>
        <span class="shrink-0">{{ kindIcon[entry.kind] }}</span>
        <span class="text-text-primary/75">{{ entry.label }}</span>
      </div>
      <div v-if="entry.lootText" class="text-text-secondary text-xs mt-0.5 pl-12">{{ entry.lootText }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useSurveyStore, type SurveyLogEntry } from "../../stores/surveyStore";
import { formatAnyTimestamp as formatTs } from "../../composables/useTimestamp";

withDefaults(defineProps<{
  embedded?: boolean;
}>(), {
  embedded: false,
});

const store = useSurveyStore();

const kindIcon: Record<SurveyLogEntry["kind"], string> = {
  "map-crafted": "\u{1F5FA}",
  "survey-used": "\u{1F50D}",
  completed: "\u26CF",
};
const kindColor: Record<SurveyLogEntry["kind"], string> = {
  "map-crafted": "#7ec8e3",
  "survey-used": "#aaa",
  completed: "#7ec87e",
};
</script>
