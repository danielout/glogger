<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../stores/gameDataStore";
import type { SkillInfo, AbilityInfo } from "../types/gameData";

const store = useGameDataStore();

const query = ref("");
const allSkills = ref<SkillInfo[]>([]);
const filteredSkills = ref<SkillInfo[]>([]);
const selected = ref<SkillInfo | null>(null);
const relatedAbilities = ref<AbilityInfo[]>([]);
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const loading = ref(false);

onMounted(async () => {
  if (store.status === "ready") {
    await loadAllSkills();
  }
});

watch(() => store.status, async (newStatus) => {
  if (newStatus === "ready") {
    await loadAllSkills();
  }
});

async function loadAllSkills() {
  loading.value = true;
  try {
    const skills = await store.getAllSkills();
    allSkills.value = skills.sort((a, b) => a.name.localeCompare(b.name));
    filteredSkills.value = allSkills.value;
  } finally {
    loading.value = false;
  }
}

watch(query, (val) => {
  if (!val.trim()) {
    filteredSkills.value = allSkills.value;
    return;
  }
  const q = val.toLowerCase();
  filteredSkills.value = allSkills.value.filter(skill =>
    skill.name.toLowerCase().includes(q) ||
    skill.description?.toLowerCase().includes(q)
  );
});

async function selectSkill(skill: SkillInfo) {
  selected.value = skill;
  iconSrc.value = null;
  relatedAbilities.value = [];

  // Load icon if present
  if (skill.icon_id) {
    iconLoading.value = true;
    try {
      const path = await store.getIconPath(skill.icon_id);
      iconSrc.value = convertFileSrc(path);
    } catch (e) {
      console.warn("Icon fetch failed:", e);
    } finally {
      iconLoading.value = false;
    }
  }

  // Load related abilities
  try {
    relatedAbilities.value = await store.getAbilitiesForSkill(skill.name);
    relatedAbilities.value.sort((a, b) => (a.level || 0) - (b.level || 0));
  } catch (e) {
    console.warn("Failed to load abilities:", e);
  }
}

function clearSelection() {
  selected.value = null;
  iconSrc.value = null;
  relatedAbilities.value = [];
}
</script>

<template>
  <div class="skill-browser">
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
      <!-- Left panel: search + results -->
      <div class="search-panel">
        <div class="search-bar">
          <input
            v-model="query"
            class="search-input"
            placeholder="Search skills…"
            autofocus />
          <span v-if="loading" class="search-spinner">⟳</span>
          <span v-else-if="filteredSkills.length" class="search-count">{{
            filteredSkills.length
          }}</span>
        </div>

        <div v-if="!allSkills.length && !loading" class="search-hint">
          No skills loaded
        </div>

        <div v-else-if="filteredSkills.length === 0 && query" class="search-hint">
          No skills found for "{{ query }}"
        </div>

        <ul v-else class="results-list">
          <li
            v-for="skill in filteredSkills"
            :key="skill.id"
            class="result-row"
            :class="{ active: selected?.id === skill.id }"
            @click="selectSkill(skill)">
            <span class="result-id">#{{ skill.id }}</span>
            <span class="result-name">{{ skill.name }}</span>
          </li>
        </ul>
      </div>

      <!-- Right panel: skill detail -->
      <div class="detail-panel" :class="{ empty: !selected }">
        <div v-if="!selected" class="detail-empty">
          Select a skill to inspect
        </div>

        <template v-else>
          <div class="detail-header">
            <!-- Icon -->
            <div class="icon-wrap">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="item-icon"
                alt="skill icon" />
              <div v-else-if="iconLoading" class="icon-placeholder loading">
                ⟳
              </div>
              <div v-else-if="selected.icon_id" class="icon-placeholder">
                {{ selected.icon_id }}
              </div>
              <div v-else class="icon-placeholder muted">—</div>
            </div>

            <div class="detail-title-block">
              <div class="detail-name">{{ selected.name }}</div>
              <div class="detail-meta">
                ID: <span class="mono">{{ selected.id }}</span>
                <template v-if="selected.icon_id">
                  · Icon:
                  <span class="mono">{{ selected.icon_id }}</span></template
                >
                <template v-if="selected.xp_table">
                  · XP Table:
                  <span class="mono">{{ selected.xp_table }}</span></template
                >
              </div>
              <div v-if="selected.description" class="detail-desc">
                {{ selected.description }}
              </div>
            </div>

            <button class="close-btn" @click="clearSelection">✕</button>
          </div>

          <!-- Related Abilities -->
          <div v-if="relatedAbilities.length" class="detail-section">
            <div class="section-label">Related Abilities ({{ relatedAbilities.length }})</div>
            <ul class="ability-list">
              <li
                v-for="ability in relatedAbilities"
                :key="ability.id"
                class="ability-line">
                <span class="ability-level">[Lv {{ ability.level || 0 }}]</span>
                <span class="ability-name">{{ ability.name }}</span>
              </li>
            </ul>
          </div>

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="detail-section">
            <div class="section-label">Keywords</div>
            <div class="keyword-list">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="keyword"
                :class="{ lint: kw.startsWith('Lint_') }"
                >{{ kw }}</span
              >
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
.skill-browser {
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
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.3rem 0.5rem;
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

.result-id {
  color: #444;
  font-size: 0.72rem;
  min-width: 3rem;
  flex-shrink: 0;
}
.result-name {
  color: #bbb;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

.icon-wrap {
  flex-shrink: 0;
}
.item-icon {
  width: 48px;
  height: 48px;
  image-rendering: pixelated;
  border: 1px solid #333;
}
.icon-placeholder {
  width: 48px;
  height: 48px;
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.65rem;
  color: #444;
}
.icon-placeholder.loading {
  color: #e0c060;
  animation: spin 1s linear infinite;
}
.icon-placeholder.muted {
  color: #333;
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

.ability-list {
  margin: 0;
  padding: 0;
  list-style: none;
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid #1a1a1a;
}
.ability-line {
  font-size: 0.8rem;
  color: #aaa;
  padding: 0.2rem 0.5rem;
  display: flex;
  gap: 0.5rem;
  border-bottom: 1px solid #151515;
}
.ability-line:hover {
  background: #1a1a1a;
}
.ability-level {
  color: #666;
  font-size: 0.72rem;
  min-width: 3.5rem;
  flex-shrink: 0;
}
.ability-name {
  color: #7ec8e3;
  flex: 1;
}

.keyword-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.keyword {
  font-size: 0.72rem;
  padding: 0.15rem 0.4rem;
  background: #1a1a2e;
  border: 1px solid #2a2a4e;
  color: #7ec8e3;
}
.keyword.lint {
  background: #1e1a10;
  border-color: #3a3010;
  color: #887040;
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
