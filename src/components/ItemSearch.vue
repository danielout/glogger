<script setup lang="ts">
import { ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../stores/gameDataStore";
import type { ItemInfo } from "../types/gameData";

const store = useGameDataStore();

const query = ref("");
const results = ref<ItemInfo[]>([]);
const selected = ref<ItemInfo | null>(null);
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const searching = ref(false);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  if (!val.trim()) {
    results.value = [];
    return;
  }
  debounceTimer = setTimeout(() => doSearch(val.trim()), 250);
});

async function doSearch(q: string) {
  searching.value = true;
  try {
    results.value = await store.searchItems(q, 30);
  } finally {
    searching.value = false;
  }
}

async function selectItem(item: ItemInfo) {
  selected.value = item;
  iconSrc.value = null;

  if (item.icon_id) {
    iconLoading.value = true;
    try {
      const path = await store.getIconPath(item.icon_id);
      iconSrc.value = convertFileSrc(path);
    } catch (e) {
      console.warn("Icon fetch failed:", e);
    } finally {
      iconLoading.value = false;
    }
  }
}

function clearSelection() {
  selected.value = null;
  iconSrc.value = null;
}
</script>

<template>
  <div class="item-search">
    <!-- Status banner if data not ready -->
    <div v-if="store.status !== 'ready'" class="status-banner">
      <span v-if="store.status === 'loading'" class="status-loading"
        >⟳ Loading game data…</span
      >
      <span v-else-if="store.status === 'error'" class="status-error"
        >✕ {{ store.errorMessage }}</span
      >
    </div>

    <div v-else class="search-layout">
      <!-- Left panel: search + results -->
      <div class="search-panel">
        <div class="search-bar">
          <input
            v-model="query"
            class="search-input"
            placeholder="Search items…"
            autofocus />
          <span v-if="searching" class="search-spinner">⟳</span>
          <span v-else-if="query && results.length" class="search-count">{{
            results.length
          }}</span>
        </div>

        <div v-if="!query" class="search-hint">
          Start typing to search
          {{ store.cacheStatus?.item_count?.toLocaleString() ?? "…" }} items
        </div>

        <div v-else-if="results.length === 0 && !searching" class="search-hint">
          No items found for "{{ query }}"
        </div>

        <ul v-else class="results-list">
          <li
            v-for="item in results"
            :key="item.id"
            class="result-row"
            :class="{ active: selected?.id === item.id }"
            @click="selectItem(item)">
            <span class="result-id">#{{ item.id }}</span>
            <span class="result-name">{{ item.name }}</span>
            <span
              v-if="item.keywords.includes('Lint_NotObtainable')"
              class="tag-unobtainable"
              >unobtainable</span
            >
          </li>
        </ul>
      </div>

      <!-- Right panel: item detail -->
      <div class="detail-panel" :class="{ empty: !selected }">
        <div v-if="!selected" class="detail-empty">
          Select an item to inspect
        </div>

        <template v-else>
          <div class="detail-header">
            <!-- Icon -->
            <div class="icon-wrap">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="item-icon"
                alt="item icon" />
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
                <template v-if="selected.value">
                  · Value:
                  <span class="mono">{{ selected.value }}c</span></template
                >
                <template v-if="selected.max_stack_size">
                  · Stack:
                  <span class="mono">{{
                    selected.max_stack_size
                  }}</span></template
                >
              </div>
              <div v-if="selected.description" class="detail-desc">
                {{ selected.description }}
              </div>
            </div>

            <button class="close-btn" @click="clearSelection">✕</button>
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

          <!-- Effect descs -->
          <div v-if="selected.effect_descs.length" class="detail-section">
            <div class="section-label">Effects</div>
            <ul class="effect-list">
              <li
                v-for="(eff, i) in selected.effect_descs"
                :key="i"
                class="effect-line">
                {{ eff }}
              </li>
            </ul>
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
.item-search {
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

.search-layout {
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
.tag-unobtainable {
  font-size: 0.65rem;
  color: #664;
  border: 1px solid #443;
  padding: 0 3px;
  flex-shrink: 0;
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

.effect-list {
  margin: 0;
  padding: 0 0 0 1rem;
}
.effect-line {
  font-size: 0.8rem;
  color: #9a9;
  padding: 0.1rem 0;
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
