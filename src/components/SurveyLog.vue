<script setup lang="ts">
import { useSurveyStore, type SurveyLogEntry } from "../stores/surveyStore";

const store = useSurveyStore();

const kindIcon: Record<SurveyLogEntry["kind"], string> = {
  "session-start": "🗺",
  located: "📍",
  completed: "⛏",
};
const kindColor: Record<SurveyLogEntry["kind"], string> = {
  "session-start": "#7ec8e3",
  located: "#aaa",
  completed: "#7ec87e",
};
</script>

<template>
  <div class="survey-log">
    <div class="log-header">Activity Log</div>
    <div v-if="store.log.length === 0" class="empty">No events yet.</div>
    <div
      v-for="(entry, i) in store.log"
      :key="i"
      class="log-entry"
      :style="{ borderLeftColor: kindColor[entry.kind] }">
      <span class="log-time">{{ entry.timestamp }}</span>
      <span class="log-icon">{{ kindIcon[entry.kind] }}</span>
      <span class="log-label">{{ entry.label }}</span>
      <div v-if="entry.lootText" class="log-loot">{{ entry.lootText }}</div>
    </div>
  </div>
</template>

<style scoped>
.survey-log {
  background: #111;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 0.75rem;
  max-height: 400px;
  overflow-y: auto;
}
.log-header {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #555;
  margin-bottom: 0.5rem;
}
.empty {
  color: #444;
  font-style: italic;
  font-size: 0.85rem;
}
.log-entry {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.3rem 0.5rem;
  border-left: 3px solid #444;
  margin-bottom: 0.25rem;
  font-size: 0.82rem;
}
.log-time {
  color: #555;
  font-size: 0.75rem;
  flex-shrink: 0;
}
.log-icon {
  flex-shrink: 0;
}
.log-label {
  color: #bbb;
}
.log-loot {
  color: #888;
  font-size: 0.75rem;
  width: 100%;
  margin-top: 0.1rem;
  padding-left: 1rem;
}
</style>
