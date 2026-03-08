<script setup lang="ts">
import { ref, onMounted, watch, computed } from "vue";
import { useGameDataStore } from "../stores/gameDataStore";
import type { NpcInfo } from "../types/gameData";

const store = useGameDataStore();

const query = ref("");
const selectedArea = ref<string>("All Areas");
const allNpcs = ref<NpcInfo[]>([]);
const filteredNpcs = ref<NpcInfo[]>([]);
const selected = ref<NpcInfo | null>(null);
const loading = ref(false);

onMounted(async () => {
  if (store.status === "ready") {
    await loadAllNpcs();
  }
});

watch(() => store.status, async (newStatus) => {
  if (newStatus === "ready") {
    await loadAllNpcs();
  }
});

async function loadAllNpcs() {
  loading.value = true;
  try {
    const npcs = await store.getAllNpcs();
    allNpcs.value = npcs.sort((a, b) => a.name.localeCompare(b.name));
    filteredNpcs.value = allNpcs.value;
  } finally {
    loading.value = false;
  }
}

// Get unique areas for the filter dropdown
const availableAreas = computed(() => {
  const areas = new Set<string>();
  allNpcs.value.forEach(npc => {
    if (npc.area_friendly_name) {
      areas.add(npc.area_friendly_name);
    }
  });
  return Array.from(areas).sort();
});

// Filter NPCs based on search query and area
watch([query, selectedArea], () => {
  let results = allNpcs.value;

  // Filter by area
  if (selectedArea.value !== "All Areas") {
    results = results.filter(npc => npc.area_friendly_name === selectedArea.value);
  }

  // Filter by search query
  if (query.value.trim()) {
    const q = query.value.toLowerCase();
    results = results.filter(npc =>
      npc.name.toLowerCase().includes(q) ||
      npc.desc?.toLowerCase().includes(q)
    );
  }

  filteredNpcs.value = results;
});

async function selectNpc(npc: NpcInfo) {
  selected.value = npc;
}

function clearSelection() {
  selected.value = null;
}
</script>

<template>
  <div class="npc-browser">
    <!-- Status banner if data not ready -->
    <div v-if="store.status !== 'ready'" class="status-banner">
      <span v-if="store.status === 'loading'" class="status-loading"
        >⟳ Loading game data…</span
      >
      <span v-else-if="store.status === 'error'" class="status-error"
        >✕ {{ store.errorMessage }}</span
      >
    </div>

    <div v-else class="browser-layout">
      <!-- Left panel: filters + results -->
      <div class="search-panel">
        <!-- Area filter -->
        <div class="filter-bar">
          <select v-model="selectedArea" class="area-select">
            <option value="All Areas">All Areas</option>
            <option v-for="area in availableAreas" :key="area" :value="area">
              {{ area }}
            </option>
          </select>
        </div>

        <!-- Search bar -->
        <div class="search-bar">
          <input
            v-model="query"
            class="search-input"
            placeholder="Search NPCs…"
            autofocus />
          <span v-if="loading" class="search-spinner">⟳</span>
          <span v-else-if="filteredNpcs.length" class="search-count">{{
            filteredNpcs.length
          }}</span>
        </div>

        <div v-if="!allNpcs.length && !loading" class="search-hint">
          No NPCs loaded
        </div>

        <div v-else-if="filteredNpcs.length === 0" class="search-hint">
          No NPCs found
        </div>

        <ul v-else class="results-list">
          <li
            v-for="npc in filteredNpcs"
            :key="npc.key"
            class="result-row"
            :class="{ active: selected?.key === npc.key }"
            @click="selectNpc(npc)">
            <span class="result-name">{{ npc.name }}</span>
            <span v-if="npc.area_friendly_name" class="result-area">{{
              npc.area_friendly_name
            }}</span>
          </li>
        </ul>
      </div>

      <!-- Right panel: NPC detail -->
      <div class="detail-panel" :class="{ empty: !selected }">
        <div v-if="!selected" class="detail-empty">
          Select an NPC to inspect
        </div>

        <template v-else>
          <div class="detail-header">
            <div class="detail-title-block">
              <div class="detail-name">{{ selected.name }}</div>
              <div class="detail-meta">
                Key: <span class="mono">{{ selected.key }}</span>
                <template v-if="selected.area_name">
                  · Area:
                  <span class="mono">{{ selected.area_name }}</span></template
                >
              </div>
              <div v-if="selected.area_friendly_name" class="detail-location">
                📍 {{ selected.area_friendly_name }}
              </div>
              <div v-if="selected.desc" class="detail-desc">
                {{ selected.desc }}
              </div>
            </div>

            <button class="close-btn" @click="clearSelection">✕</button>
          </div>

          <!-- Training Section -->
          <div v-if="selected.trains_skills.length" class="detail-section">
            <div class="section-label">Trains Skills ({{ selected.trains_skills.length }})</div>
            <div class="skill-list">
              <span
                v-for="skill in selected.trains_skills"
                :key="skill"
                class="skill-tag">
                {{ skill }}
              </span>
            </div>
          </div>

          <!-- Favor Preferences -->
          <div v-if="selected.preferences.length" class="detail-section">
            <div class="section-label">Favor Preferences ({{ selected.preferences.length }})</div>
            <div class="preferences-list">
              <div
                v-for="(pref, idx) in selected.preferences.slice().sort((a, b) => b.pref - a.pref)"
                :key="idx"
                class="preference-row">
                <span class="pref-desire" :class="`desire-${pref.desire.toLowerCase()}`">
                  {{ pref.desire }}
                </span>
                <span v-if="pref.name" class="pref-name">{{ pref.name }}</span>
                <span v-else-if="pref.keywords.length" class="pref-keywords">
                  {{ pref.keywords.join(', ') }}
                </span>
                <span class="pref-value">+{{ pref.pref.toFixed(0) }}</span>
              </div>
            </div>
          </div>

          <!-- Gift Items -->
          <div v-if="selected.item_gifts.length" class="detail-section">
            <div class="section-label">Favorite Gift Items ({{ selected.item_gifts.length }})</div>
            <div class="gift-list">
              <span
                v-for="gift in selected.item_gifts"
                :key="gift"
                class="gift-tag">
                {{ gift }}
              </span>
            </div>
          </div>

          <!-- Raw JSON -->
          <div class="detail-section">
            <div class="section-label">Raw JSON</div>
            <pre class="json-dump">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.npc-browser {
  height: calc(100vh - 130px);
  display: flex;
  flex-direction: column;
}

.status-banner {
  padding: 1rem;
  font-size: 0.9rem;
}
.status-loading {
  color: #e0c060;
}
.status-error {
  color: #f66;
}

.browser-layout {
  display: flex;
  gap: 1rem;
  height: 100%;
  overflow: hidden;
}

/* ── Left panel ── */
.search-panel {
  width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow: hidden;
}

.filter-bar {
  display: flex;
  gap: 0.5rem;
}
.area-select {
  flex: 1;
  padding: 0.45rem 0.6rem;
  background: #1a1a1a;
  border: 1px solid #444;
  color: #ccc;
  font-family: monospace;
  font-size: 0.9rem;
  outline: none;
  cursor: pointer;
}
.area-select:focus {
  border-color: #e0c060;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  position: relative;
}
.search-input {
  flex: 1;
  padding: 0.45rem 0.6rem;
  background: #1a1a1a;
  border: 1px solid #444;
  color: #ccc;
  font-family: monospace;
  font-size: 0.9rem;
  outline: none;
}
.search-input:focus {
  border-color: #e0c060;
}
.search-spinner {
  color: #e0c060;
  font-size: 0.9rem;
  animation: spin 1s linear infinite;
}
.search-count {
  color: #555;
  font-size: 0.8rem;
  min-width: 1.5rem;
  text-align: right;
}

.search-hint {
  color: #444;
  font-size: 0.8rem;
  font-style: italic;
  padding: 0.25rem 0;
}

.results-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
  border: 1px solid #2a2a2a;
}
.result-row {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  padding: 0.4rem 0.5rem;
  cursor: pointer;
  border-bottom: 1px solid #1a1a1a;
  font-size: 0.82rem;
}
.result-row:hover {
  background: #1e1e1e;
}
.result-row.active {
  background: #1a1a2e;
  border-left: 2px solid #e0c060;
}

.result-name {
  color: #bbb;
  flex: 1;
}
.result-area {
  color: #555;
  font-size: 0.72rem;
}

/* ── Right panel ── */
.detail-panel {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #2a2a2a;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.detail-panel.empty {
  align-items: center;
  justify-content: center;
}
.detail-empty {
  color: #333;
  font-style: italic;
}

.detail-header {
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
}

.detail-title-block {
  flex: 1;
  min-width: 0;
}
.detail-name {
  color: #e0c060;
  font-size: 1rem;
  font-weight: bold;
  margin-bottom: 0.25rem;
}
.detail-meta {
  font-size: 0.75rem;
  color: #555;
  margin-bottom: 0.3rem;
}
.detail-location {
  font-size: 0.8rem;
  color: #888;
  margin-bottom: 0.3rem;
}
.mono {
  color: #888;
  font-family: monospace;
}
.detail-desc {
  font-size: 0.82rem;
  color: #888;
  font-style: italic;
}

.close-btn {
  background: none;
  border: none;
  color: #444;
  cursor: pointer;
  padding: 0 0.25rem;
  font-size: 0.9rem;
  flex-shrink: 0;
}
.close-btn:hover {
  color: #f66;
  background: none;
  border: none;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
.section-label {
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #555;
  border-bottom: 1px solid #222;
  padding-bottom: 0.2rem;
}

.skill-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.skill-tag {
  font-size: 0.72rem;
  padding: 0.15rem 0.4rem;
  background: #1a1a2e;
  border: 1px solid #2a2a4e;
  color: #7ec8e3;
}

.preferences-list {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid #1a1a1a;
  padding: 0.5rem;
}
.preference-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  padding: 0.2rem 0;
}
.pref-desire {
  font-size: 0.7rem;
  text-transform: uppercase;
  font-weight: bold;
  padding: 0.1rem 0.3rem;
  border-radius: 2px;
  min-width: 4rem;
  text-align: center;
  flex-shrink: 0;
}
.desire-love {
  background: #4a1a3a;
  color: #ff69b4;
  border: 1px solid #6a2a5a;
}
.desire-like {
  background: #1a3a1a;
  color: #7ec8e3;
  border: 1px solid #2a5a2a;
}
.desire-dislike {
  background: #3a2a1a;
  color: #f66;
  border: 1px solid #5a3a2a;
}
.desire-hate {
  background: #3a1a1a;
  color: #ff4444;
  border: 1px solid #5a2a2a;
}
.pref-name,
.pref-keywords {
  color: #aaa;
  flex: 1;
}
.pref-value {
  color: #7ec8e3;
  font-weight: bold;
  min-width: 3rem;
  text-align: right;
  flex-shrink: 0;
}

.gift-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.gift-tag {
  font-size: 0.72rem;
  padding: 0.15rem 0.4rem;
  background: #1e1a2e;
  border: 1px solid #3a2a4e;
  color: #b8a8c8;
}

.json-dump {
  background: #0d0d0d;
  border: 1px solid #222;
  padding: 0.75rem;
  font-size: 0.72rem;
  color: #666;
  overflow-x: auto;
  white-space: pre;
  margin: 0;
  line-height: 1.5;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
