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
          <span class="text-xs">{{ showFilters ? '▾' : '▸' }}</span>
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
            <label class="text-text-muted min-w-14 shrink-0">Keyword</label>
            <div class="flex-1 relative" v-if="!filterKeyword">
              <input
                v-model="keywordQuery"
                class="input w-full text-xs py-0.5"
                placeholder="Search keywords…"
                @focus="keywordDropdownOpen = true"
                @blur="onKeywordBlur" />
              <div
                v-if="keywordDropdownOpen && filteredKeywords.length"
                class="absolute z-10 top-full left-0 right-0 max-h-40 overflow-y-auto bg-surface-dark border border-surface-elevated mt-0.5">
                <div
                  v-for="kw in filteredKeywords"
                  :key="kw"
                  class="px-2 py-1 cursor-pointer text-[0.72rem] hover:bg-[#1e1e1e]"
                  :class="{ 'text-[#887040]': kw.startsWith('Lint_'), 'text-[#7ec8e3]': !kw.startsWith('Lint_') }"
                  @mousedown.prevent="selectKeyword(kw)">
                  {{ kw }}
                </div>
              </div>
            </div>
            <div v-else class="flex items-center gap-1 flex-1">
              <span
                class="text-[0.72rem] px-1.5 py-0.5 border"
                :class="filterKeyword.startsWith('Lint_') ? 'bg-[#1e1a10] border-[#3a3010] text-[#887040]' : 'bg-[#1a1a2e] border-[#2a2a4e] text-[#7ec8e3]'">
                {{ filterKeyword }}
              </span>
              <button
                class="bg-transparent border-none text-text-dim cursor-pointer text-xs hover:text-accent-red"
                @click="filterKeyword = ''; keywordQuery = ''">✕</button>
            </div>
          </div>
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

        <ul v-else ref="listRef" class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(item, idx) in results"
            :key="item.id"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.id === item.id, 'bg-surface-elevated': selectedIndex === idx && selected?.id !== item.id }"
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
              <div v-if="selected.skill_reqs" class="text-xs flex gap-2 items-center flex-wrap">
                <span class="text-text-muted min-w-20">Requires:</span>
                <span v-for="(level, skill) in selected.skill_reqs" :key="skill" class="flex items-center gap-1">
                  <SkillInline :reference="String(skill)" /> <span class="text-text-secondary">{{ level }}</span>
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
              <div v-if="selected.bestow_ability" class="text-xs flex gap-2 items-center px-2 py-0.5">
                <span class="text-text-muted min-w-16">Ability:</span>
                <button
                  class="bg-transparent border-none text-[#e08060] cursor-pointer p-0 text-xs hover:underline"
                  @click="navigateToEntity({ type: 'ability', id: selected.bestow_ability })">
                  {{ selected.bestow_ability }}
                </button>
              </div>
              <div v-if="selected.bestow_quest" class="text-xs flex gap-2 items-center px-2 py-0.5">
                <span class="text-text-muted min-w-16">Quest:</span>
                <QuestInline :reference="selected.bestow_quest" />
              </div>
              <div v-if="selected.bestow_recipes?.length" class="text-xs flex gap-2 items-center px-2 py-0.5 flex-wrap">
                <span class="text-text-muted min-w-16">Recipes:</span>
                <RecipeInline v-for="recipe in selected.bestow_recipes" :key="String(recipe)" :reference="String(recipe)" />
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

          <!-- Recipes Producing This Item -->
          <div v-if="recipesProducing.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Produced By ({{ recipesProducing.length }})</div>
            <div class="flex flex-col gap-1">
              <div
                v-for="recipe in recipesProducing"
                :key="recipe.id"
                class="flex gap-2 items-center text-xs px-2 py-0.5 bg-[#151515] border-l-2 border-l-[#2a4a2a]">
                <span class="text-text-muted text-[0.72rem] min-w-14 shrink-0">[Lv {{ recipe.skill_level_req || 0 }}]</span>
                <RecipeInline :reference="recipe.name" />
                <span v-if="recipe.skill" class="text-text-dim text-[0.65rem] ml-auto">{{ recipe.skill }}</span>
              </div>
            </div>
          </div>

          <!-- Recipes Using This Item -->
          <div v-if="recipesUsing.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Used In ({{ recipesUsing.length }})</div>
            <div class="flex flex-col gap-1">
              <div
                v-for="recipe in recipesUsing"
                :key="recipe.id"
                class="flex gap-2 items-center text-xs px-2 py-0.5 bg-[#151515] border-l-2 border-l-[#4a3a1a]">
                <span class="text-text-muted text-[0.72rem] min-w-14 shrink-0">[Lv {{ recipe.skill_level_req || 0 }}]</span>
                <RecipeInline :reference="recipe.name" />
                <span v-if="recipe.skill" class="text-text-dim text-[0.65rem] ml-auto">{{ recipe.skill }}</span>
              </div>
            </div>
          </div>

          <!-- NPCs Who Want This Item -->
          <div v-if="npcsWantingItem.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">NPC Favor ({{ npcsWantingItem.length }})</div>
            <div class="flex flex-col gap-1 max-h-60 overflow-y-auto">
              <div
                v-for="entry in npcsWantingItem"
                :key="entry.npc_key"
                class="flex gap-2 items-center text-xs px-2 py-0.5 bg-[#151515] border-l-2"
                :class="{
                  'border-l-[#ff69b4]': entry.desire.toLowerCase() === 'love',
                  'border-l-[#7ec8e3]': entry.desire.toLowerCase() === 'like',
                  'border-l-accent-red': entry.desire.toLowerCase() === 'dislike',
                  'border-l-[#aa4444]': entry.desire.toLowerCase() === 'hate',
                }">
                <span
                  class="text-[0.65rem] uppercase font-bold px-1 py-0.5 min-w-12 text-center shrink-0"
                  :class="{
                    'bg-[#4a1a3a] text-[#ff69b4] border border-[#6a2a5a]': entry.desire.toLowerCase() === 'love',
                    'bg-[#1a3a1a] text-[#7ec8e3] border border-[#2a5a2a]': entry.desire.toLowerCase() === 'like',
                    'bg-[#3a2a1a] text-accent-red border border-[#5a3a2a]': entry.desire.toLowerCase() === 'dislike',
                    'bg-[#3a1a1a] text-[#aa4444] border border-[#5a2a2a]': entry.desire.toLowerCase() === 'hate',
                  }">
                  {{ entry.desire }}
                </span>
                <NpcInline :reference="entry.npc_key" />
                <span class="text-[#7ec8e3] font-bold ml-auto shrink-0">+{{ entry.pref.toFixed(0) }}</span>
                <span v-if="entry.match_type !== 'name'" class="text-text-dim text-[0.6rem] italic shrink-0">({{ entry.match_type }})</span>
              </div>
            </div>
          </div>

          <!-- Keyword Recipe Uses -->
          <div v-if="keywordRecipes.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Could Fill Keyword Slots In ({{ keywordRecipes.length }})</div>
            <div class="flex flex-col gap-1">
              <div
                v-for="recipe in keywordRecipes"
                :key="recipe.id"
                class="flex gap-2 items-center text-xs px-2 py-0.5 bg-[#151515] border-l-2 border-l-[#4a4a1a]">
                <span class="text-text-muted text-[0.72rem] min-w-14 shrink-0">[Lv {{ recipe.skill_level_req || 0 }}]</span>
                <RecipeInline :reference="recipe.name" />
                <span v-if="recipe.skill" class="text-text-dim text-[0.65rem] ml-auto">{{ recipe.skill }}</span>
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
import { ref, computed, watch, onMounted } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import { useEntityNavigation } from "../../composables/useEntityNavigation";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import type { ItemInfo, RecipeInfo, EntitySources, NpcFavorEntry } from "../../types/gameData";
import SourcesPanel from "../Shared/SourcesPanel.vue";
import RecipeInline from "../Shared/Recipe/RecipeInline.vue";
import QuestInline from "../Shared/Quest/QuestInline.vue";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import NpcInline from "../Shared/NPC/NpcInline.vue";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const store = useGameDataStore();
const settingsStore = useSettingsStore();
const { navigateToEntity } = useEntityNavigation();

const query = ref("");
const results = ref<ItemInfo[]>([]);
const selected = ref<ItemInfo | null>(null);
const sources = ref<EntitySources | null>(null);
const sourcesLoading = ref(false);
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const searching = ref(false);
const selectedIndex = ref(0);
const recipesProducing = ref<RecipeInfo[]>([]);
const recipesUsing = ref<RecipeInfo[]>([]);
const npcsWantingItem = ref<NpcFavorEntry[]>([]);
const keywordRecipes = ref<RecipeInfo[]>([]);
const listRef = ref<HTMLElement | null>(null);

// Advanced filters
const showFilters = ref(false);
const filterSlot = ref("");
const filterKeyword = ref("");
const keywordQuery = ref("");
const keywordDropdownOpen = ref(false);
const allKeywords = ref<string[]>([]);
const filterLevelMin = ref<number | undefined>(undefined);
const filterLevelMax = ref<number | undefined>(undefined);
const equipSlots = ref<string[]>([]);

const filteredKeywords = computed(() => {
  const q = keywordQuery.value.toLowerCase();
  if (!q) return allKeywords.value.slice(0, 50);
  return allKeywords.value.filter(kw => kw.toLowerCase().includes(q)).slice(0, 50);
});

function selectKeyword(kw: string) {
  filterKeyword.value = kw;
  keywordQuery.value = kw;
  keywordDropdownOpen.value = false;
}

function onKeywordBlur() {
  // Delay to allow mousedown on dropdown items
  setTimeout(() => { keywordDropdownOpen.value = false; }, 150);
}

const hasActiveFilters = computed(
  () => filterSlot.value !== "" || filterKeyword.value !== "" || filterLevelMin.value != null || filterLevelMax.value != null,
);

function clearFilters() {
  filterSlot.value = "";
  filterKeyword.value = "";
  keywordQuery.value = "";
  filterLevelMin.value = undefined;
  filterLevelMax.value = undefined;
}

onMounted(async () => {
  try {
    const [slots, keywords] = await Promise.all([
      store.getEquipSlots(),
      store.getAllItemKeywords(),
    ]);
    equipSlots.value = slots;
    allKeywords.value = keywords;
  } catch (e) {
    console.warn("Failed to load filter data:", e);
  }
});

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch([query, filterSlot, filterKeyword, filterLevelMin, filterLevelMax], () => {
  if (debounceTimer) clearTimeout(debounceTimer);
  selectedIndex.value = 0;
  const q = query.value.trim();
  if (!q && !hasActiveFilters.value) {
    results.value = [];
    return;
  }
  debounceTimer = setTimeout(() => doSearch(q), 250);
});

useKeyboard({
  listNavigation: {
    items: results,
    selectedIndex,
    onConfirm: (idx) => {
      const item = results.value[idx];
      if (item) selectItem(item);
    },
    scrollContainerRef: listRef,
  },
});

function filterUnobtainable(items: ItemInfo[]): ItemInfo[] {
  if (settingsStore.settings.showUnobtainableItems) return items;
  return items.filter(i => !i.keywords.includes('Lint_NotObtainable'));
}

async function doSearch(q: string) {
  searching.value = true;
  try {
    if (filterKeyword.value) {
      // Keyword filter: fetch all items with this keyword, then apply text + other filters client-side
      let items = await store.getItemsByKeyword(filterKeyword.value);
      if (q) {
        const lower = q.toLowerCase();
        items = items.filter(i => i.name.toLowerCase().includes(lower));
      }
      if (filterSlot.value) {
        items = items.filter(i => i.equip_slot === filterSlot.value);
      }
      if (filterLevelMin.value != null) {
        items = items.filter(i => i.crafting_target_level != null && i.crafting_target_level >= filterLevelMin.value!);
      }
      if (filterLevelMax.value != null) {
        items = items.filter(i => i.crafting_target_level != null && i.crafting_target_level <= filterLevelMax.value!);
      }
      results.value = filterUnobtainable(items);
    } else {
      results.value = filterUnobtainable(await store.searchItems(q, {
        equipSlot: filterSlot.value || undefined,
        levelMin: filterLevelMin.value,
        levelMax: filterLevelMax.value,
      }));
    }
  } finally {
    searching.value = false;
  }
}

async function selectItem(item: ItemInfo) {
  selected.value = item;
  iconSrc.value = null;
  sources.value = null;
  recipesProducing.value = [];
  recipesUsing.value = [];
  npcsWantingItem.value = [];
  keywordRecipes.value = [];

  // Load sources
  sourcesLoading.value = true;
  store.getItemSources(item.id)
    .then(s => { sources.value = s; })
    .catch(e => { console.warn("Sources fetch failed:", e); })
    .finally(() => { sourcesLoading.value = false; });

  // Load related recipes
  Promise.all([
    store.getRecipesForItem(item.id),
    store.getRecipesUsingItem(item.id),
  ]).then(([producing, using]) => {
    recipesProducing.value = producing;
    recipesUsing.value = using;
  }).catch(e => { console.warn("Recipe cross-ref fetch failed:", e); });

  // Load NPCs wanting this item
  store.getNpcsWantingItem(item.id)
    .then(entries => { npcsWantingItem.value = entries; })
    .catch(e => { console.warn("NPC favor fetch failed:", e); });

  // Load keyword-based recipe matches
  const nonLintKeywords = item.keywords.filter(kw => !kw.startsWith('Lint_'));
  if (nonLintKeywords.length) {
    Promise.all(nonLintKeywords.map(kw => store.getRecipesForKeyword(kw)))
      .then(results => {
        // Deduplicate by recipe ID
        const seen = new Set<number>();
        const merged: RecipeInfo[] = [];
        for (const list of results) {
          for (const recipe of list) {
            if (!seen.has(recipe.id)) {
              seen.add(recipe.id);
              merged.push(recipe);
            }
          }
        }
        keywordRecipes.value = merged;
      })
      .catch(e => { console.warn("Keyword recipe fetch failed:", e); });
  }

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
  recipesProducing.value = [];
  recipesUsing.value = [];
  npcsWantingItem.value = [];
  keywordRecipes.value = [];
}

// Navigate to a specific item when navTarget changes
watch(() => props.navTarget, async (target) => {
  if (!target || target.type !== 'item') return;
  const name = String(target.id);

  // If already selected, nothing to do
  if (selected.value?.name === name) return;

  const item = await store.resolveItem(name);
  if (item) {
    query.value = item.name;
    selectItem(item);
  }
}, { immediate: true });
</script>
