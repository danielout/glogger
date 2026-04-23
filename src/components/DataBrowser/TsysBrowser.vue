<template>
  <PaneLayout screen-key="db-treasure" :left-pane="{ title: 'Treasure Mods', defaultWidth: 360, minWidth: 280, maxWidth: 500 }">
    <template #left>
      <!-- Status banner if data not ready -->
      <div v-if="store.status !== 'ready'" class="p-4 text-sm">
        <span v-if="store.status === 'loading'" class="text-accent-gold">⟳ Loading game data…</span>
        <span v-else-if="store.status === 'error'" class="text-accent-red">✕ {{ store.errorMessage }}</span>
      </div>

      <template v-else>
      <div class="flex flex-col gap-2 h-full overflow-hidden">
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
              <span v-if="entry.skill" class="text-text-muted text-[10px] shrink-0">{{ entry.skill }}</span>
            </div>
            <div v-if="entry.prefix || entry.suffix" class="text-text-dim text-[10px]">
              <span v-if="entry.prefix">{{ entry.prefix }}</span>
              <span v-if="entry.prefix && entry.suffix"> · </span>
              <span v-if="entry.suffix">{{ entry.suffix }}</span>
            </div>
          </li>
        </ul>
      </div>
      </template>
    </template>

    <!-- Right panel: detail -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated p-4 flex flex-col gap-4"
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
                Key: <span class="text-text-secondary">{{ selected.key }}</span>
                <template v-if="selected.skill">
                  · Skill: <SkillInline :reference="selected.skill" />
                </template>
                <template v-if="selected.tier_count">
                  · Tiers: <span class="text-text-secondary">{{ selected.tier_count }}</span>
                </template>
              </div>
            </div>

            <div class="flex gap-1 shrink-0">
              <span v-if="selected.is_unavailable" class="text-[10px] px-1.5 py-0.5 bg-accent-red/20 border border-accent-red/40 text-accent-red">Unavailable</span>
              <span v-if="selected.is_hidden_from_transmutation" class="text-[10px] px-1.5 py-0.5 bg-yellow-900/30 border border-yellow-700/40 text-yellow-400">Hidden from Transmute</span>
              <button
                class="bg-transparent border-none cursor-pointer px-1 py-0 text-sm transition-colors"
                :class="isFav ? 'text-accent-gold' : 'text-text-dim hover:text-accent-gold'"
                :title="isFav ? 'Remove from favorites' : 'Add to favorites'"
                @click="dataBrowserStore.toggleFavorite({ type: 'treasure', reference: selected.key, label: selected.internal_name || selected.key })"
              >&#x2605;</button>
              <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm hover:text-accent-red" @click="clearSelection">✕</button>
            </div>
          </div>

          <!-- Prefix / Suffix -->
          <div v-if="selected.prefix || selected.suffix" class="flex flex-col gap-1.5">
            <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Naming</div>
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
            <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Equipment Slots</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="slot in selected.slots"
                :key="slot"
                class="text-xs px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-item">
                {{ slot }}
              </span>
            </div>
          </div>

          <!-- Tiers -->
          <div v-if="selected.tiers && Object.keys(selected.tiers).length" class="flex flex-col gap-1.5">
            <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
              Tiers ({{ Object.keys(selected.tiers).length }})
            </div>
            <div class="flex flex-col gap-2">
              <div
                v-for="(tier, tierKey) in sortedTiers"
                :key="tierKey"
                class="bg-surface-dark border border-surface-card p-2 text-xs">
                <div class="flex items-baseline gap-3 mb-1">
                  <span class="text-text-dim text-[10px]">{{ tierKey }}</span>
                  <span v-if="tier.min_level != null" class="text-text-muted">
                    Lv {{ tier.min_level }}–{{ tier.max_level }}
                  </span>
                  <span v-if="tier.min_rarity" class="text-entity-item text-[10px]">{{ tier.min_rarity }}</span>
                  <span v-if="tier.skill_level_prereq" class="text-text-muted text-[10px]">
                    Prereq: {{ tier.skill_level_prereq }}
                  </span>
                </div>
                <div v-if="tier.effect_descs.length" class="flex flex-col gap-0.5 pl-2">
                  <span
                    v-for="(effect, i) in tier.effect_descs"
                    :key="i"
                    class="text-text-secondary text-xs">
                    {{ effect }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Related Abilities -->
          <div v-if="relatedAbilitiesLoading || relatedAbilities.length" class="flex flex-col gap-1.5">
            <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
              Related Abilities
              <span v-if="relatedAbilities.length" class="text-text-muted">({{ relatedAbilities.length }})</span>
            </div>
            <div v-if="relatedAbilitiesLoading" class="text-accent-gold text-xs animate-spin">⟳</div>
            <div v-else class="flex flex-col gap-1">
              <div
                v-for="ab in relatedAbilities"
                :key="ab.id"
                class="flex items-baseline gap-2 px-2 py-1 bg-surface-dark border border-surface-card text-xs">
                <AbilityInline :reference="ab.name" />
                <span v-if="ab.skill" class="text-text-muted text-[10px]">{{ ab.skill }}</span>
                <span v-if="ab.level" class="text-text-dim text-[10px]">Lv {{ ab.level }}</span>
              </div>
            </div>
          </div>

          <!-- Raw JSON -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-xs text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected.raw_json, null, 2) }}</pre>
          </div>
        </template>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from "../Shared/PaneLayout.vue";
import { ref, watch, computed, onMounted } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import { useDataBrowserStore } from "../../stores/dataBrowserStore";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import AbilityInline from "../Shared/Ability/AbilityInline.vue";
import type { TsysBrowserEntry, TsysTierInfo, AbilityTsysXref } from "../../types/gameData";

const store = useGameDataStore();
const settingsStore = useSettingsStore();
const dataBrowserStore = useDataBrowserStore();

const query = ref("");
const results = ref<TsysBrowserEntry[]>([]);
const allEntries = ref<TsysBrowserEntry[]>([]);
const selected = ref<TsysBrowserEntry | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);
const searching = ref(false);
const skillFilter = ref("");
const relatedAbilities = ref<AbilityTsysXref[]>([]);
const relatedAbilitiesLoading = ref(false);

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
const sortedTiers = computed<Record<string, TsysTierInfo>>(() => {
  if (!selected.value?.tiers) return {};
  const entries = Object.entries(selected.value.tiers);
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

const isFav = computed(() =>
  selected.value ? dataBrowserStore.isFavorite("treasure", selected.value.key) : false
);

function selectEntry(entry: TsysBrowserEntry) {
  selected.value = entry;
  relatedAbilities.value = [];
  dataBrowserStore.addToHistory({ type: "treasure", reference: entry.key, label: entry.internal_name || entry.key });

  relatedAbilitiesLoading.value = true;
  store.getAbilitiesForTsys(entry.key)
    .then(a => { relatedAbilities.value = a; })
    .catch(e => { console.warn("Ability xref fetch failed:", e); })
    .finally(() => { relatedAbilitiesLoading.value = false; });
}

function clearSelection() {
  selected.value = null;
  relatedAbilities.value = [];
}

</script>
