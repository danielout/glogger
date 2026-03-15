<template>
  <div class="h-[calc(100vh-130px)] flex flex-col">
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
      <div class="w-75 shrink-0 flex flex-col gap-2 overflow-hidden">
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search items…"
            autofocus />
          <span v-if="searching" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="(query || hasActiveFilters) && results.length" class="text-text-dim text-xs min-w-6 text-right">{{
            results.length
          }}</span>
        </div>

        <!-- Advanced filters toggle -->
        <button
          class="bg-transparent border-none text-text-dim text-[0.7rem] cursor-pointer px-0 py-0 flex items-center gap-1 hover:text-text-secondary"
          @click="showFilters = !showFilters">
          <span class="text-[0.6rem]">{{ showFilters ? '▾' : '▸' }}</span>
          Filters
          <span
            v-if="hasActiveFilters"
            class="text-[0.6rem] text-accent-gold ml-1"
            >● active</span
          >
        </button>

        <!-- Collapsible filter panel -->
        <div v-if="showFilters" class="flex flex-col gap-1.5 px-1 py-1.5 border border-surface-elevated bg-surface-dark text-xs">
          <div class="flex items-center gap-2">
            <label class="text-text-muted min-w-14 shrink-0">Slot</label>
            <select
              v-model="filterSlot"
              class="input flex-1 text-xs py-0.5">
              <option value="">Any</option>
              <option v-for="slot in equipSlots" :key="slot" :value="slot">{{ slot }}</option>
            </select>
          </div>
          <div class="flex items-center gap-2">
            <label class="text-text-muted min-w-14 shrink-0">Level</label>
            <input
              v-model.number="filterLevelMin"
              type="number"
              class="input w-16 text-xs py-0.5"
              placeholder="min"
              min="0" />
            <span class="text-text-muted">–</span>
            <input
              v-model.number="filterLevelMax"
              type="number"
              class="input w-16 text-xs py-0.5"
              placeholder="max"
              min="0" />
          </div>
          <button
            v-if="hasActiveFilters"
            class="bg-transparent border border-surface-elevated text-text-dim text-[0.65rem] cursor-pointer px-2 py-0.5 self-end hover:text-text-secondary hover:border-border-default"
            @click="clearFilters">
            Clear filters
          </button>
        </div>

        <div v-if="!query && !hasActiveFilters" class="text-text-dim text-xs italic py-1">
          Start typing to search
          {{ store.cacheStatus?.item_count?.toLocaleString() ?? "…" }} items
        </div>

        <div v-else-if="results.length === 0 && !searching && (query || hasActiveFilters)" class="text-text-dim text-xs italic py-1">
          No items found{{ query ? ` for "${query}"` : '' }}{{ hasActiveFilters ? ' with current filters' : '' }}
        </div>

        <ul v-else class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="item in results"
            :key="item.id"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.id === item.id }"
            @click="selectItem(item)">
            <span class="text-text-dim text-[0.72rem] min-w-12 shrink-0">#{{ item.id }}</span>
            <span class="text-text-primary/75 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ item.name }}</span>
            <span
              v-if="item.keywords.includes('Lint_NotObtainable')"
              class="text-[0.65rem] text-[#664] border border-[#443] px-1 shrink-0"
              >unobtainable</span
            >
          </li>
        </ul>
      </div>

      <!-- Right panel: item detail -->
      <div
        class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
        :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select an item to inspect
        </div>

        <template v-else>
          <div class="flex gap-3 items-start">
            <!-- Icon -->
            <div class="shrink-0">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="w-12 h-12 [image-rendering:pixelated] border border-border-default"
                alt="item icon" />
              <div v-else-if="iconLoading" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-accent-gold animate-spin">
                ⟳
              </div>
              <div v-else-if="selected.icon_id" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-text-dim">
                {{ selected.icon_id }}
              </div>
              <div v-else class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-border-default">—</div>
            </div>

            <div class="flex-1 min-w-0">
              <div class="text-accent-gold text-base font-bold mb-1">{{ selected.name }}</div>
              <div class="text-xs text-text-dim mb-1">
                ID: <span class="text-text-secondary font-mono">{{ selected.id }}</span>
                <template v-if="selected.icon_id">
                  · Icon:
                  <span class="text-text-secondary font-mono">{{ selected.icon_id }}</span></template
                >
                <template v-if="selected.value">
                  · Value:
                  <span class="text-text-secondary font-mono">{{ selected.value }}c</span></template
                >
                <template v-if="selected.max_stack_size">
                  · Stack:
                  <span class="text-text-secondary font-mono">{{
                    selected.max_stack_size
                  }}</span></template
                >
              </div>
              <div v-if="selected.description" class="text-xs text-text-secondary italic">
                {{ selected.description }}
              </div>
            </div>

            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Equipment Info -->
          <div v-if="selected.equip_slot || selected.skill_reqs" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Equipment</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
              <div v-if="selected.equip_slot" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Slot:</span>
                <span class="text-text-secondary">{{ selected.equip_slot }}</span>
              </div>
              <div v-if="selected.skill_reqs" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Requires:</span>
                <span class="text-text-secondary">
                  <template v-for="(level, skill) in selected.skill_reqs" :key="skill">
                    {{ skill }} {{ level }}
                  </template>
                </span>
              </div>
            </div>
          </div>

          <!-- Crafting Info -->
          <div v-if="selected.tsys_profile || selected.craft_points" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Crafting</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
              <div v-if="selected.tsys_profile" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">TSys Profile:</span>
                <span class="text-text-secondary font-mono">{{ selected.tsys_profile }}</span>
              </div>
              <div v-if="selected.craft_points" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Craft Points:</span>
                <span class="text-text-secondary">{{ selected.craft_points }}</span>
              </div>
              <div v-if="selected.crafting_target_level" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Target Level:</span>
                <span class="text-text-secondary">{{ selected.crafting_target_level }}</span>
              </div>
            </div>
          </div>

          <!-- Food Description -->
          <div v-if="selected.food_desc" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Food</div>
            <div class="text-xs text-[#c8a86e] italic px-2 py-1 bg-[#151515] border-l-2 border-l-[#4a3a1a]">
              {{ selected.food_desc }}
            </div>
          </div>

          <!-- Bestow Info -->
          <div v-if="selected.bestow_ability || selected.bestow_quest || selected.bestow_recipes?.length || selected.bestow_title" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Bestows</div>
            <div class="flex flex-col gap-1">
              <div v-if="selected.bestow_ability" class="text-xs flex gap-2 px-2 py-0.5">
                <span class="text-text-muted min-w-16">Ability:</span>
                <span class="text-text-secondary">{{ selected.bestow_ability }}</span>
              </div>
              <div v-if="selected.bestow_quest" class="text-xs flex gap-2 px-2 py-0.5">
                <span class="text-text-muted min-w-16">Quest:</span>
                <span class="text-text-secondary">{{ selected.bestow_quest }}</span>
              </div>
              <div v-if="selected.bestow_recipes?.length" class="text-xs flex gap-2 px-2 py-0.5">
                <span class="text-text-muted min-w-16">Recipes:</span>
                <span class="text-text-secondary">{{ selected.bestow_recipes.join(', ') }}</span>
              </div>
              <div v-if="selected.bestow_title" class="text-xs flex gap-2 px-2 py-0.5">
                <span class="text-text-muted min-w-16">Title ID:</span>
                <span class="text-text-secondary">{{ selected.bestow_title }}</span>
              </div>
            </div>
          </div>

          <!-- Uses -->
          <div v-if="selected.num_uses" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Usage</div>
            <div class="text-xs text-text-secondary px-2 py-1">
              {{ selected.num_uses }} use{{ selected.num_uses > 1 ? 's' : '' }}
            </div>
          </div>

          <!-- Sources -->
          <SourcesPanel :sources="sources" :loading="sourcesLoading" />

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-[#7ec8e3]"
                :class="{ 'bg-[#1e1a10]! border-[#3a3010]! text-[#887040]!': kw.startsWith('Lint_') }"
                >{{ kw }}</span
              >
            </div>
          </div>

          <!-- Effect descs -->
          <div v-if="selected.effect_descs.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Effects</div>
            <ul class="m-0 pl-4 p-0">
              <li
                v-for="(eff, i) in selected.effect_descs"
                :key="i"
                class="text-xs text-[#9a9] py-0.5">
                {{ eff }}
              </li>
            </ul>
          </div>

          <!-- Raw JSON -->
          <div class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-[0.72rem] text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import type { ItemInfo, EntitySources } from "../../types/gameData";
import SourcesPanel from "../Shared/SourcesPanel.vue";

const store = useGameDataStore();

const query = ref("");
const results = ref<ItemInfo[]>([]);
const selected = ref<ItemInfo | null>(null);
const sources = ref<EntitySources | null>(null);
const sourcesLoading = ref(false);
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const searching = ref(false);

// Advanced filters
const showFilters = ref(false);
const filterSlot = ref("");
const filterLevelMin = ref<number | undefined>(undefined);
const filterLevelMax = ref<number | undefined>(undefined);
const equipSlots = ref<string[]>([]);

const hasActiveFilters = computed(
  () => filterSlot.value !== "" || filterLevelMin.value != null || filterLevelMax.value != null,
);

function clearFilters() {
  filterSlot.value = "";
  filterLevelMin.value = undefined;
  filterLevelMax.value = undefined;
}

onMounted(async () => {
  try {
    equipSlots.value = await store.getEquipSlots();
  } catch (e) {
    console.warn("Failed to load equip slots:", e);
  }
});

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch([query, filterSlot, filterLevelMin, filterLevelMax], () => {
  if (debounceTimer) clearTimeout(debounceTimer);
  const q = query.value.trim();
  if (!q && !hasActiveFilters.value) {
    results.value = [];
    return;
  }
  debounceTimer = setTimeout(() => doSearch(q), 250);
});

async function doSearch(q: string) {
  searching.value = true;
  try {
    results.value = await store.searchItems(q, 30, {
      equipSlot: filterSlot.value || undefined,
      levelMin: filterLevelMin.value,
      levelMax: filterLevelMax.value,
    });
  } finally {
    searching.value = false;
  }
}

async function selectItem(item: ItemInfo) {
  selected.value = item;
  iconSrc.value = null;
  sources.value = null;

  // Load sources
  sourcesLoading.value = true;
  store.getItemSources(item.id)
    .then(s => { sources.value = s; })
    .catch(e => { console.warn("Sources fetch failed:", e); })
    .finally(() => { sourcesLoading.value = false; });

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
  sources.value = null;
}
</script>
