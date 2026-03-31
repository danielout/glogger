<template>
  <div class="h-full flex flex-col">
    <!-- Status banner if data not ready -->
    <div v-if="store.status !== 'ready'" class="p-4 text-sm">
      <span v-if="store.status === 'loading'" class="text-accent-gold">⟳ Loading game data…</span>
      <span v-else-if="store.status === 'error'" class="text-accent-red">✕ {{ store.errorMessage }}</span>
    </div>

    <div v-else class="flex gap-4 h-full overflow-hidden">
      <!-- Left panel: search + results -->
      <div class="w-90 shrink-0 flex flex-col gap-2 overflow-hidden">
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search treasure mods…"
            autofocus />
          <span v-if="searching" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="results.length" class="text-text-dim text-xs min-w-6 text-right">{{
            results.length
          }}</span>
        </div>

        <!-- Skill filter -->
        <select v-model="skillFilter" class="input text-xs">
          <option value="">All Skills</option>
          <option v-for="skill in availableSkills" :key="skill" :value="skill">{{ skill }}</option>
        </select>

        <div v-if="!query && !skillFilter" class="text-text-dim text-xs italic py-1">
          Start typing or pick a skill to browse treasure mods
        </div>

        <div v-else-if="filteredResults.length === 0 && !searching" class="text-text-dim text-xs italic py-1">
          No treasure mods found
        </div>

        <ul v-else ref="listRef" class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(entry, idx) in filteredResults"
            :key="entry.key"
            class="flex flex-col px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.key === entry.key, 'bg-surface-elevated': selectedIndex === idx && selected?.key !== entry.key }"
            @click="selectEntry(entry)">
            <div class="flex items-baseline gap-2">
              <span class="text-text-primary/75 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ entry.internal_name || entry.key }}</span>
              <span v-if="entry.skill" class="text-text-muted text-[0.65rem] shrink-0">{{ entry.skill }}</span>
            </div>
            <div v-if="entry.prefix || entry.suffix" class="text-text-dim text-[0.65rem]">
              <span v-if="entry.prefix">{{ entry.prefix }}</span>
              <span v-if="entry.prefix && entry.suffix"> · </span>
              <span v-if="entry.suffix">{{ entry.suffix }}</span>
            </div>
          </li>
        </ul>
      </div>

      <!-- Right panel: detail -->
      <div
        class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
        :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select a treasure mod to inspect
        </div>

        <template v-else>
          <!-- Header -->
          <div class="flex gap-3 items-start">
            <div class="flex-1 min-w-0">
              <div class="text-accent-gold text-base font-bold mb-1">{{ selected.internal_name || selected.key }}</div>
              <div class="text-xs text-text-dim mb-1">
                Key: <span class="text-text-secondary font-mono">{{ selected.key }}</span>
                <template v-if="selected.skill">
                  · Skill: <SkillInline :reference="selected.skill" />
                </template>
                <template v-if="selected.tier_count">
                  · Tiers: <span class="text-text-secondary">{{ selected.tier_count }}</span>
                </template>
              </div>
            </div>

            <div class="flex gap-1 shrink-0">
              <span v-if="selected.is_unavailable" class="text-[0.65rem] px-1.5 py-0.5 bg-accent-red/20 border border-accent-red/40 text-accent-red">Unavailable</span>
              <span v-if="selected.is_hidden_from_transmutation" class="text-[0.65rem] px-1.5 py-0.5 bg-yellow-900/30 border border-yellow-700/40 text-yellow-400">Hidden from Transmute</span>
              <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm hover:text-accent-red" @click="clearSelection">✕</button>
            </div>
          </div>

          <!-- Prefix / Suffix -->
          <div v-if="selected.prefix || selected.suffix" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Naming</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
              <div v-if="selected.prefix" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-16">Prefix:</span>
                <span class="text-text-secondary">{{ selected.prefix }}</span>
              </div>
              <div v-if="selected.suffix" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-16">Suffix:</span>
                <span class="text-text-secondary">{{ selected.suffix }}</span>
              </div>
            </div>
          </div>

          <!-- Slots -->
          <div v-if="selected.slots.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Equipment Slots</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="slot in selected.slots"
                :key="slot"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-item">
                {{ slot }}
              </span>
            </div>
          </div>

          <!-- Tiers -->
          <div v-if="selected.tiers && Object.keys(selected.tiers).length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
              Tiers ({{ Object.keys(selected.tiers).length }})
            </div>
            <div class="flex flex-col gap-2">
              <div
                v-for="(tier, tierKey) in sortedTiers"
                :key="tierKey"
                class="bg-surface-dark border border-surface-card p-2 text-xs">
                <div class="flex items-baseline gap-3 mb-1">
                  <span class="text-text-dim font-mono text-[0.65rem]">{{ tierKey }}</span>
                  <span v-if="tierMinLevel(tier) != null" class="text-text-muted">
                    Lv {{ tierMinLevel(tier) }}–{{ tierMaxLevel(tier) }}
                  </span>
                  <span v-if="tierRarity(tier)" class="text-entity-item text-[0.65rem]">{{ tierRarity(tier) }}</span>
                  <span v-if="tierSkillPrereq(tier)" class="text-text-muted text-[0.65rem]">
                    Prereq: {{ tierSkillPrereq(tier) }}
                  </span>
                </div>
                <div v-if="tierEffects(tier).length" class="flex flex-col gap-0.5 pl-2">
                  <span
                    v-for="(effect, i) in tierEffects(tier)"
                    :key="i"
                    class="text-text-secondary font-mono text-[0.72rem]">
                    {{ effect }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Raw JSON -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-[0.72rem] text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected.raw_json, null, 2) }}</pre>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import type { TsysBrowserEntry } from "../../types/gameData";

const store = useGameDataStore();
const settingsStore = useSettingsStore();

const query = ref("");
const results = ref<TsysBrowserEntry[]>([]);
const allEntries = ref<TsysBrowserEntry[]>([]);
const selected = ref<TsysBrowserEntry | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);
const searching = ref(false);
const skillFilter = ref("");

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// Compute available skills from loaded entries
const availableSkills = computed(() => {
  const skills = new Set<string>();
  for (const entry of allEntries.value) {
    if (entry.skill) skills.add(entry.skill);
  }
  return [...skills].sort();
});

// Filter results by skill
const filteredResults = computed(() => {
  if (!skillFilter.value) return results.value;
  return results.value.filter((e) => e.skill === skillFilter.value);
});

// Sort tiers by their numeric id
const sortedTiers = computed(() => {
  if (!selected.value?.tiers) return {};
  const tiers = selected.value.tiers as Record<string, unknown>;
  const entries = Object.entries(tiers);
  entries.sort((a, b) => {
    const aNum = parseInt(a[0].replace("id_", ""));
    const bNum = parseInt(b[0].replace("id_", ""));
    return aNum - bNum;
  });
  return Object.fromEntries(entries);
});

// Load all entries on mount for skill filter
onMounted(async () => {
  if (store.status === "ready") {
    await loadAll();
  }
});

watch(() => store.status, async (val) => {
  if (val === "ready" && allEntries.value.length === 0) {
    await loadAll();
  }
});

async function loadAll() {
  searching.value = true;
  try {
    allEntries.value = await store.getAllTsys();
    results.value = allEntries.value;
  } finally {
    searching.value = false;
  }
}

watch(query, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  if (!val.trim()) {
    results.value = allEntries.value;
    return;
  }
  debounceTimer = setTimeout(() => doSearch(val.trim()), 250);
});

async function doSearch(q: string) {
  searching.value = true;
  try {
    results.value = await store.searchTsys(q, 200);
  } finally {
    searching.value = false;
  }
}

watch(filteredResults, () => {
  selectedIndex.value = 0;
});

useKeyboard({
  listNavigation: {
    items: filteredResults,
    selectedIndex,
    onConfirm: (index: number) => {
      if (filteredResults.value[index]) selectEntry(filteredResults.value[index]);
    },
    scrollContainerRef: listRef,
  },
});

function selectEntry(entry: TsysBrowserEntry) {
  selected.value = entry;
}

function clearSelection() {
  selected.value = null;
}

// Tier field helpers
function tierMinLevel(tier: unknown): number | null {
  const t = tier as Record<string, unknown>;
  return typeof t?.MinLevel === "number" ? t.MinLevel : null;
}

function tierMaxLevel(tier: unknown): number | null {
  const t = tier as Record<string, unknown>;
  return typeof t?.MaxLevel === "number" ? t.MaxLevel : null;
}

function tierRarity(tier: unknown): string | null {
  const t = tier as Record<string, unknown>;
  return typeof t?.MinRarity === "string" ? t.MinRarity : null;
}

function tierSkillPrereq(tier: unknown): number | null {
  const t = tier as Record<string, unknown>;
  return typeof t?.SkillLevelPrereq === "number" ? t.SkillLevelPrereq : null;
}

function tierEffects(tier: unknown): string[] {
  const t = tier as Record<string, unknown>;
  if (Array.isArray(t?.EffectDescs)) {
    return t.EffectDescs.filter((d: unknown) => typeof d === "string") as string[];
  }
  return [];
}
</script>
