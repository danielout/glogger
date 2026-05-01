<template>
  <PaneLayout screen-key="db-areas" :left-pane="{ title: 'Areas', defaultWidth: 300, minWidth: 220, maxWidth: 420 }">
    <template #left>
      <div v-if="store.status !== 'ready'" class="p-4 text-sm">
        <span v-if="store.status === 'loading'" class="text-accent-gold">&#x27F3; Loading game data…</span>
        <span v-else-if="store.status === 'error'" class="text-accent-red">&#x2715; {{ store.errorMessage }}</span>
      </div>

      <template v-else>
        <div class="flex flex-col gap-2 h-full overflow-hidden">
          <!-- Search bar -->
          <div class="flex items-center gap-2 relative">
            <input
              v-model="query"
              class="input flex-1"
              placeholder="Search areas…"
              autofocus />
            <span v-if="filteredAreas.length" class="text-text-dim text-xs min-w-6 text-right">{{
              filteredAreas.length
            }}</span>
          </div>

          <div v-if="!allAreas.length && !loading" class="text-text-dim text-xs italic py-1">
            No areas loaded
          </div>

          <div v-else-if="filteredAreas.length === 0" class="text-text-dim text-xs italic py-1">
            No areas found
          </div>

          <ul v-else ref="listRef" class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
            <li
              v-for="(area, idx) in filteredAreas"
              :key="area.key"
              class="flex flex-col gap-0.5 px-2 py-1.5 cursor-pointer border-b border-surface-dark text-xs hover:bg-surface-row-hover"
              :class="{
                'bg-surface-card border-l-2 border-l-accent-gold': selected?.key === area.key,
                'bg-surface-elevated': selectedIndex === idx && selected?.key !== area.key,
              }"
              @click="selectArea(area)">
              <span class="text-text-primary/75 flex-1">{{ area.friendly_name }}</span>
              <div class="flex gap-2 text-text-dim text-[0.65rem]">
                <span v-if="area.npc_count">{{ area.npc_count }} NPCs</span>
                <span v-if="area.monster_count">{{ area.monster_count }} monsters</span>
              </div>
            </li>
          </ul>
        </div>
      </template>
    </template>

    <!-- Center: area detail -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated flex flex-col"
      :class="{ 'items-center justify-center': !selected }">
      <div v-if="!selected" class="text-border-default italic">
        Select an area to inspect
      </div>

      <template v-else>
        <!-- Toolbar: favorite + close -->
        <div class="flex items-center justify-end gap-1 px-4 pt-3 pb-0">
          <button
            class="bg-transparent border-none cursor-pointer px-1 py-0 text-sm shrink-0 transition-colors"
            :class="isFav ? 'text-accent-gold' : 'text-text-dim hover:text-accent-gold'"
            :title="isFav ? 'Remove from favorites' : 'Add to favorites'"
            @click="dataBrowserStore.toggleFavorite({ type: 'area', reference: selected.key, label: selected.friendly_name })"
          >&#x2605;</button>
          <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">&#x2715;</button>
        </div>

        <div class="flex flex-col gap-4 px-4 py-4">
          <!-- Header -->
          <div>
            <h2 class="text-lg font-bold text-entity-area m-0">{{ selected.friendly_name }}</h2>
            <div v-if="selected.short_friendly_name && selected.short_friendly_name !== selected.friendly_name" class="text-text-muted text-xs mt-0.5">
              {{ selected.short_friendly_name }}
            </div>
            <div class="text-text-dim text-[0.65rem] mt-1 font-mono">{{ selected.key }}</div>
          </div>

          <!-- 3-column layout: NPCs | Storage Vaults | Monsters -->
          <div class="grid grid-cols-3 gap-4">
            <!-- NPCs -->
            <div class="flex flex-col gap-1.5">
              <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
                NPCs ({{ areaNpcs.length }})
              </div>
              <ul v-if="areaNpcs.length" class="list-disc pl-4 m-0 text-xs flex flex-col gap-0.5">
                <li v-for="npc in areaNpcs" :key="npc.key">
                  <NpcInline :reference="npc.key" :npc="npc" />
                </li>
              </ul>
              <div v-else class="text-text-dim text-xs italic">None</div>
            </div>

            <!-- Storage Vaults -->
            <div class="flex flex-col gap-1.5">
              <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
                Storage Vaults ({{ areaVaults.length }})
              </div>
              <ul v-if="areaVaults.length" class="list-disc pl-4 m-0 text-xs flex flex-col gap-0.5">
                <li v-for="vault in areaVaults" :key="vault.key" class="text-text-secondary">
                  <span>{{ vault.npc_friendly_name }}</span>
                  <span v-if="vault.max_slots" class="text-text-dim"> — up to {{ vault.max_slots }}</span>
                </li>
              </ul>
              <div v-else class="text-text-dim text-xs italic">None</div>
            </div>

            <!-- Monsters -->
            <div class="flex flex-col gap-1.5">
              <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
                Monsters ({{ areaMonsters.length }})
              </div>
              <ul v-if="areaMonsters.length" class="list-disc pl-4 m-0 text-xs flex flex-col gap-0.5">
                <li v-for="name in areaMonsters" :key="name">
                  <EnemyInline :reference="name" />
                </li>
              </ul>
              <div v-else class="text-text-dim text-xs italic">None</div>
            </div>
          </div>

          <!-- Raw JSON -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-[0.72rem] text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </div>
      </template>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from "../Shared/PaneLayout.vue";
import NpcInline from "../Shared/NPC/NpcInline.vue";
import EnemyInline from "../Shared/Enemy/EnemyInline.vue";
import { ref, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import { useDataBrowserSearch } from "../../composables/useDataBrowserSearch";
import { combineFields } from "../../utils/SearchParser";
import { useDataBrowserStore } from "../../stores/dataBrowserStore";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import type { NpcInfo } from "../../types/gameData";

interface AreaSummary {
  key: string;
  friendly_name: string;
  short_friendly_name: string | null;
  npc_count: number;
  monster_count: number;
}

interface VaultSummary {
  key: string;
  npc_friendly_name: string;
  max_slots: number;
}

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const store = useGameDataStore();
const settingsStore = useSettingsStore();
const dataBrowserStore = useDataBrowserStore();

const isFav = computed(() =>
  selected.value ? dataBrowserStore.isFavorite("area", selected.value.key) : false
);

const selected = ref<AreaSummary | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);
const areaNpcs = ref<NpcInfo[]>([]);
const areaMonsters = ref<string[]>([]);
const areaVaults = ref<VaultSummary[]>([]);

// Load all areas on mount
const allAreas = ref<AreaSummary[]>([]);
const loading = ref(true);

invoke<AreaSummary[]>("get_all_areas")
  .then((areas) => { allAreas.value = areas; })
  .catch((e) => { console.warn("Failed to load areas:", e); })
  .finally(() => { loading.value = false; });

const { query, filtered: filteredAreas } = useDataBrowserSearch(allAreas, {
  searchText: (area) => combineFields(area.friendly_name, area.short_friendly_name, area.key),
});

watch(filteredAreas, () => {
  selectedIndex.value = 0;
});

function selectArea(area: AreaSummary) {
  selected.value = area;
  dataBrowserStore.addToHistory({ type: "area", reference: area.key, label: area.friendly_name });
  loadAreaDetails(area.key);
}

function clearSelection() {
  selected.value = null;
  areaNpcs.value = [];
  areaMonsters.value = [];
  areaVaults.value = [];
}

async function loadAreaDetails(areaKey: string) {
  areaNpcs.value = [];
  areaMonsters.value = [];
  areaVaults.value = [];

  const [npcs, monsters, vaults] = await Promise.all([
    invoke<NpcInfo[]>("get_npcs_in_area", { area: areaKey }).catch(() => [] as NpcInfo[]),
    invoke<string[]>("get_monsters_in_area", { area: areaKey }).catch(() => [] as string[]),
    invoke<VaultSummary[]>("get_storage_vaults_in_area", { area: areaKey }).catch(() => [] as VaultSummary[]),
  ]);

  // Only update if still viewing the same area
  if (selected.value?.key === areaKey) {
    areaNpcs.value = npcs;
    areaMonsters.value = monsters;
    areaVaults.value = vaults;
  }
}

useKeyboard({
  listNavigation: {
    items: filteredAreas,
    selectedIndex,
    onConfirm: (index: number) => selectArea(filteredAreas.value[index]),
    scrollContainerRef: listRef,
  },
});

// Navigate to a specific area when navTarget changes
watch(() => props.navTarget, (target) => {
  if (!target || target.type !== "area") return;
  const key = String(target.id);
  if (selected.value?.key === key) return;

  const match = allAreas.value.find(a => a.key === key || a.friendly_name === key);
  if (match) {
    query.value = match.friendly_name;
    selectArea(match);
  }
}, { immediate: true });
</script>
