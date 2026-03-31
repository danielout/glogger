<template>
  <div class="h-full flex flex-col">
    <!-- Status banner if data not ready -->
    <div v-if="store.status !== 'ready'" class="p-4 text-sm">
      <span v-if="store.status === 'loading'" class="text-accent-gold"
        >⟳ Loading game data…</span
      >
      <span v-else-if="store.status === 'error'" class="text-accent-red"
        >✕ {{ store.errorMessage }}</span
      >
    </div>

    <div v-else class="flex gap-4 h-full overflow-hidden">
      <!-- Left panel: search + results -->
      <div class="w-90 shrink-0 flex flex-col gap-2 overflow-hidden">
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search effects…"
            autofocus />
          <span v-if="searching" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="query && results.length" class="text-text-dim text-xs min-w-6 text-right">{{
            results.length
          }}</span>
        </div>

        <div v-if="!query" class="text-text-dim text-xs italic py-1">
          Start typing to search effects
        </div>

        <div v-else-if="results.length === 0 && !searching" class="text-text-dim text-xs italic py-1">
          No effects found for "{{ query }}"
        </div>

        <ul v-else ref="listRef" class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(effect, idx) in results"
            :key="effect.id"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.id === effect.id, 'bg-surface-elevated': selectedIndex === idx && selected?.id !== effect.id }"
            @click="selectEffect(effect)">
            <span class="text-text-dim text-[0.72rem] min-w-12 shrink-0">#{{ effect.id }}</span>
            <span class="text-text-primary/75 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ effect.name || 'Unnamed' }}</span>
            <span v-if="effect.display_mode" class="text-text-muted text-[0.65rem] shrink-0">{{ effect.display_mode }}</span>
          </li>
        </ul>
      </div>

      <!-- Right panel: effect detail -->
      <div
        class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
        :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select an effect to inspect
        </div>

        <template v-else>
          <div class="flex gap-3 items-start">
            <!-- Icon -->
            <div class="shrink-0">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="w-12 h-12 [image-rendering:pixelated] border border-border-default"
                alt="effect icon" />
              <div v-else-if="iconLoading" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-accent-gold animate-spin">
                ⟳
              </div>
              <div v-else-if="selected.icon_id" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-text-dim">
                {{ selected.icon_id }}
              </div>
              <div v-else class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-border-default">—</div>
            </div>

            <div class="flex-1 min-w-0">
              <div class="text-accent-gold text-base font-bold mb-1">{{ selected.name || 'Unnamed Effect' }}</div>
              <div class="text-xs text-text-dim mb-1">
                ID: <span class="text-text-secondary font-mono">{{ selected.id }}</span>
                <template v-if="selected.display_mode">
                  · Mode:
                  <span class="text-text-secondary font-mono">{{ selected.display_mode }}</span></template
                >
                <template v-if="selected.icon_id">
                  · Icon:
                  <span class="text-text-secondary font-mono">{{ selected.icon_id }}</span></template
                >
              </div>
              <div v-if="selected.desc" class="text-xs text-text-secondary italic">
                {{ selected.desc }}
              </div>
            </div>

            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Duration & Stacking -->
          <div v-if="selected.duration || selected.stacking_type" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Details</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
              <div v-if="selected.duration" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Duration:</span>
                <span class="text-text-secondary">{{ formatDuration(selected.duration) }}</span>
              </div>
              <div v-if="selected.stacking_type" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Stacking:</span>
                <span class="text-text-secondary">{{ selected.stacking_type }}</span>
              </div>
              <div v-if="selected.stacking_priority" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Priority:</span>
                <span class="text-text-secondary">{{ selected.stacking_priority }}</span>
              </div>
              <div v-if="selected.particle" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Particle:</span>
                <span class="text-text-secondary font-mono">{{ selected.particle }}</span>
              </div>
            </div>
          </div>

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-item">
                {{ kw }}
              </span>
            </div>
          </div>

          <!-- Ability Keywords -->
          <div v-if="selected.ability_keywords.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Ability Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="kw in selected.ability_keywords"
                :key="kw"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a2a1a] border border-[#2a4a2a] text-[#8ab88a]">
                {{ kw }}
              </span>
            </div>
          </div>

          <!-- Raw JSON -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-[0.72rem] text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import type { EffectInfo } from "../../types/gameData";

const store = useGameDataStore();
const settingsStore = useSettingsStore();

const query = ref("");
const results = ref<EffectInfo[]>([]);
const selected = ref<EffectInfo | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);
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
    results.value = await store.searchEffects(q, 50);
  } finally {
    searching.value = false;
  }
}

watch(results, () => {
  selectedIndex.value = 0;
});

useKeyboard({
  listNavigation: {
    items: results,
    selectedIndex,
    onConfirm: (index: number) => {
      if (results.value[index]) selectEffect(results.value[index]);
    },
    scrollContainerRef: listRef,
  },
});

async function selectEffect(effect: EffectInfo) {
  selected.value = effect;
  iconSrc.value = null;

  if (effect.icon_id) {
    iconLoading.value = true;
    try {
      const path = await store.getIconPath(effect.icon_id);
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

function formatDuration(duration: unknown): string {
  if (typeof duration === "number") {
    if (duration >= 3600) {
      const hours = Math.floor(duration / 3600);
      const mins = Math.floor((duration % 3600) / 60);
      return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
    }
    if (duration >= 60) {
      const mins = Math.floor(duration / 60);
      const secs = duration % 60;
      return secs > 0 ? `${mins}m ${secs}s` : `${mins}m`;
    }
    return `${duration}s`;
  }
  return String(duration);
}
</script>
