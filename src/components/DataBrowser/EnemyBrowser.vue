<template>
  <PaneLayout screen-key="db-enemies" :left-pane="{ title: 'Enemies', defaultWidth: 360, minWidth: 280, maxWidth: 500 }">
    <template #left>
      <!-- Status banner if data not ready -->
      <div v-if="store.status !== 'ready'" class="p-4 text-sm">
        <span v-if="store.status === 'loading'" class="text-accent-gold"
          >⟳ Loading game data…</span
        >
        <span v-else-if="store.status === 'error'" class="text-accent-red"
          >✕ {{ store.errorMessage }}</span
        >
      </div>

      <template v-else>
      <div class="flex flex-col gap-2 h-full overflow-hidden">
        <!-- Strategy filter -->
        <div class="flex gap-2">
          <select v-model="selectedStrategy" class="input flex-1 cursor-pointer">
            <option value="All">All Strategies</option>
            <option v-for="s in availableStrategies" :key="s" :value="s">
              {{ s }}
            </option>
          </select>
        </div>

        <!-- Search bar -->
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search enemies…"
            autofocus />
          <span v-if="loading" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="filteredEnemies.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredEnemies.length
          }}</span>
        </div>

        <div v-if="!allEnemies.length && !loading" class="text-text-dim text-xs italic py-1">
          No enemies loaded
        </div>

        <div v-else-if="filteredEnemies.length === 0" class="text-text-dim text-xs italic py-1">
          No enemies found
        </div>

        <ul v-else ref="listRef" class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(enemy, idx) in filteredEnemies"
            :key="enemy.key"
            class="flex items-baseline gap-2 px-2 py-1.5 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{
              'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.key === enemy.key,
              'bg-surface-elevated': selectedIndex === idx && selected?.key !== enemy.key,
            }"
            @click="selectEnemy(enemy)">
            <span class="text-entity-enemy flex-1">{{ formatName(enemy.key) }}</span>
            <span v-if="enemy.strategy" class="text-text-dim text-[0.65rem] shrink-0">{{ enemy.strategy }}</span>
            <span v-if="enemy.mobility_type" class="text-text-muted text-[0.65rem] shrink-0">{{ enemy.mobility_type }}</span>
          </li>
        </ul>
      </div>
      </template>
    </template>

    <!-- Right panel: enemy detail -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated p-4 flex flex-col gap-4"
      :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select an enemy to inspect
        </div>

        <template v-else>
          <!-- Header -->
          <div class="flex gap-3 items-start">
            <div class="flex-1 min-w-0">
              <div class="text-entity-enemy text-base font-bold mb-1">{{ formatName(selected.key) }}</div>
              <div class="text-xs text-text-dim mb-1">
                Key: <span class="text-text-secondary font-mono">{{ selected.key }}</span>
              </div>
              <div v-if="selected.comment" class="text-xs text-text-secondary italic">
                {{ selected.comment }}
              </div>
            </div>

            <button
              class="bg-transparent border-none cursor-pointer px-1 py-0 text-sm shrink-0 transition-colors"
              :class="isFav ? 'text-accent-gold' : 'text-text-dim hover:text-accent-gold'"
              :title="isFav ? 'Remove from favorites' : 'Add to favorites'"
              @click="dataBrowserStore.toggleFavorite({ type: 'enemy', reference: selected.key, label: formatName(selected.key) })"
            >&#x2605;</button>
            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">&#x2715;</button>
          </div>

          <!-- Properties -->
          <div class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Properties</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
              <div v-if="selected.strategy" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-24">Strategy:</span>
                <span class="text-text-secondary">{{ selected.strategy }}</span>
              </div>
              <div v-if="selected.mobility_type" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-24">Mobility:</span>
                <span class="text-text-secondary">{{ selected.mobility_type }}</span>
              </div>
              <div class="text-xs flex gap-2">
                <span class="text-text-muted min-w-24">Swimming:</span>
                <span class="text-text-secondary">{{ selected.swimming ? 'Yes' : 'No' }}</span>
              </div>
              <div v-if="selected.uncontrolled_pet" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-24">Uncontrolled Pet:</span>
                <span class="text-text-secondary">Yes</span>
              </div>
            </div>
          </div>

          <!-- Abilities -->
          <div v-if="selected.ability_names.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
              Abilities ({{ selected.ability_count }})
            </div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="ability in selected.ability_names"
                :key="ability"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-ability">
                {{ ability }}
              </span>
            </div>
          </div>

          <!-- Raw JSON (via settings toggle) -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-[0.72rem] text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </template>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from "../Shared/PaneLayout.vue";
import { ref, watch, computed } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useDataBrowserStore } from "../../stores/dataBrowserStore";
import { useKeyboard } from "../../composables/useKeyboard";
import { useDataBrowserSearch } from "../../composables/useDataBrowserSearch";
import { combineFields } from "../../utils/SearchParser";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import type { EnemyInfo } from "../../types/gameData";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const store = useGameDataStore();
const settingsStore = useSettingsStore();
const dataBrowserStore = useDataBrowserStore();

const selectedStrategy = ref<string>("All");
const allEnemies = ref<EnemyInfo[]>([]);
const selected = ref<EnemyInfo | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);
const loading = ref(false);

const isFav = computed(() =>
  selected.value ? dataBrowserStore.isFavorite("enemy", selected.value.key) : false
);

// Load all enemies once
async function loadEnemies() {
  if (allEnemies.value.length > 0) return;
  loading.value = true;
  try {
    allEnemies.value = await store.getAllEnemies();
  } finally {
    loading.value = false;
  }
}

// Load on mount when data is ready
watch(() => store.status, (s) => {
  if (s === "ready") loadEnemies();
}, { immediate: true });

// Get unique strategies for the filter dropdown
const availableStrategies = computed(() => {
  const strategies = new Set<string>();
  allEnemies.value.forEach(e => {
    if (e.strategy) strategies.add(e.strategy);
  });
  return Array.from(strategies).sort();
});

// Pre-filter by strategy dropdown, then use unified search for text
const strategyFiltered = computed(() => {
  if (selectedStrategy.value === "All") return allEnemies.value;
  return allEnemies.value.filter(e => e.strategy === selectedStrategy.value);
});

const { query, filtered: filteredEnemies } = useDataBrowserSearch(strategyFiltered, {
  searchText: (e) => combineFields(e.key, formatName(e.key), e.comment, e.ability_names.join(" ")),
});

watch(filteredEnemies, () => {
  selectedIndex.value = 0;
});

function selectEnemy(enemy: EnemyInfo) {
  selected.value = enemy;
  dataBrowserStore.addToHistory({ type: "enemy", reference: enemy.key, label: formatName(enemy.key) });
}

function clearSelection() {
  selected.value = null;
}

/** Convert a CamelCase or underscore key to a more readable name */
function formatName(key: string): string {
  // Replace underscores with spaces, then add spaces before capitals
  return key
    .replace(/_/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2');
}

useKeyboard({
  listNavigation: {
    items: filteredEnemies,
    selectedIndex,
    onConfirm: (index: number) => selectEnemy(filteredEnemies.value[index]),
    scrollContainerRef: listRef,
  },
});

// Navigate to a specific enemy when navTarget changes
watch(() => props.navTarget, (target) => {
  if (!target || target.type !== 'enemy') return;
  const key = String(target.id);
  if (selected.value?.key === key) return;

  const match = allEnemies.value.find(e => e.key === key || formatName(e.key) === key);
  if (match) {
    query.value = formatName(match.key);
    selectEnemy(match);
  }
}, { immediate: true });
</script>
