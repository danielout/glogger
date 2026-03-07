<script setup lang="ts">
import { computed } from "vue";
import { useSurveyStore } from "../stores/surveyStore";

const store = useSurveyStore();
const s = computed(() => store.session);
</script>

<template>
  <div v-if="!store.sessionActive" class="inactive">
    No active survey session. Start watching a log or parse a file.
  </div>

  <div v-else-if="s" class="session-card">
    <div class="session-header">
      <span class="session-title">Survey Session</span>
      <span class="session-time"
        >Started {{ s.startTime }} · {{ store.elapsed }} elapsed</span
      >
    </div>

    <div class="stat-row">
      <div class="stat-block">
        <div class="stat-label">Maps Started</div>
        <div class="stat-value">{{ s.mapsStarted }}</div>
      </div>
      <div class="stat-block">
        <div class="stat-label">Located</div>
        <div class="stat-value">{{ s.surveysLocated }}</div>
      </div>
      <div class="stat-block">
        <div class="stat-label">Completed</div>
        <div class="stat-value">{{ s.surveysCompleted }}</div>
      </div>
      <div class="stat-block">
        <div class="stat-label">Completion Rate</div>
        <div class="stat-value">
          {{
            s.surveysLocated > 0
              ? Math.round((s.surveysCompleted / s.surveysLocated) * 100) + "%"
              : "—"
          }}
        </div>
      </div>
      <div class="stat-block">
        <div class="stat-label">Avg Time / Survey</div>
        <div class="stat-value">{{ store.avgSurveyTime }}</div>
      </div>
    </div>

    <div class="xp-row">
      <div class="xp-block surveying">
        <div class="xp-label">Surveying XP</div>
        <div class="xp-value">+{{ s.surveyingXpGained.toLocaleString() }}</div>
      </div>
      <div class="xp-block mining">
        <div class="xp-label">Mining XP</div>
        <div class="xp-value">+{{ s.miningXpGained.toLocaleString() }}</div>
      </div>
      <div class="xp-block geology">
        <div class="xp-label">Geology XP</div>
        <div class="xp-value">+{{ s.geologyXpGained.toLocaleString() }}</div>
      </div>
    </div>

    <div v-if="store.lootSummary.length > 0" class="loot-section">
      <div class="loot-header">Items Found</div>
      <div class="loot-grid">
        <div
          v-for="entry in store.lootSummary"
          :key="entry.item"
          class="loot-row">
          <span class="loot-name">{{ entry.item }}</span>
          <span class="loot-count">×{{ entry.count }}</span>
          <div class="loot-bar-wrap">
            <div class="loot-bar" :style="{ width: entry.pct + '%' }"></div>
          </div>
          <span class="loot-pct">{{ entry.pct }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inactive {
  color: #555;
  font-style: italic;
  padding: 1rem 0;
}

.session-card {
  background: #1a1a2e;
  border: 1px solid #444;
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1rem;
}
.session-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 0.75rem;
}
.session-title {
  font-size: 1rem;
  font-weight: bold;
  color: #7ec8e3;
}
.session-time {
  font-size: 0.75rem;
  color: #666;
}

.stat-row {
  display: flex;
  gap: 1.5rem;
  margin-bottom: 0.75rem;
  flex-wrap: wrap;
}
.stat-block {
  text-align: center;
}
.stat-label {
  font-size: 0.65rem;
  color: #666;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.stat-value {
  font-size: 1.1rem;
  font-weight: bold;
  color: #ccc;
}

.xp-row {
  display: flex;
  gap: 1rem;
  margin-bottom: 1rem;
}
.xp-block {
  flex: 1;
  padding: 0.5rem;
  border-radius: 4px;
  text-align: center;
}
.xp-block.surveying {
  background: #1a2e1a;
  border: 1px solid #3a5a3a;
}
.xp-block.mining {
  background: #2e1a1a;
  border: 1px solid #5a3a3a;
}
.xp-block.geology {
  background: #2e2a1a;
  border: 1px solid #5a4a2a;
}
.xp-label {
  font-size: 0.65rem;
  color: #888;
  text-transform: uppercase;
}
.xp-value {
  font-size: 0.95rem;
  font-weight: bold;
}
.xp-block.surveying .xp-value {
  color: #7ec87e;
}
.xp-block.mining .xp-value {
  color: #c87e7e;
}
.xp-block.geology .xp-value {
  color: #c8b47e;
}

.loot-section {
  border-top: 1px solid #2a2a3e;
  padding-top: 0.75rem;
}
.loot-header {
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #555;
  margin-bottom: 0.5rem;
}
.loot-grid {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.loot-row {
  display: grid;
  grid-template-columns: 1fr auto 120px auto;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.82rem;
}
.loot-name {
  color: #bbb;
}
.loot-count {
  color: #888;
  min-width: 2.5rem;
  text-align: right;
}
.loot-bar-wrap {
  height: 4px;
  background: #222;
  border-radius: 2px;
  overflow: hidden;
}
.loot-bar {
  height: 100%;
  background: #7ec8e3;
  border-radius: 2px;
  transition: width 0.3s ease;
}
.loot-pct {
  color: #555;
  font-size: 0.75rem;
  min-width: 2.5rem;
  text-align: right;
}
</style>
