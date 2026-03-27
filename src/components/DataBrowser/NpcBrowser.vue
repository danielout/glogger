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
      <!-- Left panel: filters + results -->
      <div class="w-75 shrink-0 flex flex-col gap-2 overflow-hidden">
        <!-- Area filter -->
        <div class="flex gap-2">
          <select v-model="selectedArea" class="input flex-1 cursor-pointer">
            <option value="All Areas">All Areas</option>
            <option v-for="area in availableAreas" :key="area" :value="area">
              {{ area }}
            </option>
          </select>
        </div>

        <!-- Search bar -->
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search NPCs…"
            autofocus />
          <span v-if="loading" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="filteredNpcs.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredNpcs.length
          }}</span>
        </div>

        <div v-if="!allNpcs.length && !loading" class="text-text-dim text-xs italic py-1">
          No NPCs loaded
        </div>

        <div v-else-if="filteredNpcs.length === 0" class="text-text-dim text-xs italic py-1">
          No NPCs found
        </div>

        <ul v-else ref="listRef" class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(npc, idx) in filteredNpcs"
            :key="npc.key"
            class="flex flex-col gap-0.5 px-2 py-1.5 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{
              'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.key === npc.key,
              'bg-surface-elevated': selectedIndex === idx && selected?.key !== npc.key,
            }"
            @click="selectNpc(npc)">
            <span class="text-text-primary/75 flex-1">{{ npc.name }}</span>
            <span v-if="npc.area_friendly_name" class="text-text-dim text-[0.72rem]">{{
              npc.area_friendly_name
            }}</span>
          </li>
        </ul>
      </div>

      <!-- Right panel: NPC detail -->
      <div
        class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
        :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select an NPC to inspect
        </div>

        <template v-else>
          <div class="flex gap-3 items-start">
            <div class="flex-1 min-w-0">
              <div class="text-accent-gold text-base font-bold mb-1">{{ selected.name }}</div>
              <div class="text-xs text-text-dim mb-1">
                Key: <span class="text-text-secondary font-mono">{{ selected.key }}</span>
                <template v-if="selected.area_name">
                  · Area:
                  <span class="text-text-secondary font-mono">{{ selected.area_name }}</span></template
                >
              </div>
              <div v-if="selected.area_friendly_name" class="text-sm text-text-secondary mb-1">
                📍 {{ selected.area_friendly_name }}
              </div>
              <div v-if="selected.desc" class="text-xs text-text-secondary italic">
                {{ selected.desc }}
              </div>
            </div>

            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Training Section -->
          <div v-if="selected.trains_skills.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Trains Skills ({{ selected.trains_skills.length }})</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="skill in selected.trains_skills"
                :key="skill"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-[#7ec8e3]">
                {{ skill }}
              </span>
            </div>
          </div>

          <!-- Favor Preferences -->
          <div v-if="selected.preferences.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Favor Preferences ({{ selected.preferences.length }})</div>
            <div class="flex flex-col gap-1 max-h-100 overflow-y-auto border border-surface-dark p-2">
              <div
                v-for="(pref, idx) in selected.preferences.slice().sort((a, b) => b.pref - a.pref)"
                :key="idx"
                class="flex items-center gap-2 text-sm py-0.5">
                <span
                  class="text-[0.7rem] uppercase font-bold px-1 py-0.5 rounded-sm min-w-16 text-center shrink-0"
                  :class="{
                    'bg-[#4a1a3a] text-[#ff69b4] border border-[#6a2a5a]': pref.desire.toLowerCase() === 'love',
                    'bg-[#1a3a1a] text-[#7ec8e3] border border-[#2a5a2a]': pref.desire.toLowerCase() === 'like',
                    'bg-[#3a2a1a] text-accent-red border border-[#5a3a2a]': pref.desire.toLowerCase() === 'dislike',
                    'bg-[#3a1a1a] text-channel-combat border border-[#5a2a2a]': pref.desire.toLowerCase() === 'hate',
                  }">
                  {{ pref.desire }}
                </span>
                <span v-if="pref.name" class="text-text-secondary flex-1">{{ pref.name }}</span>
                <span v-else-if="pref.keywords.length" class="text-text-secondary flex-1">
                  {{ pref.keywords.join(', ') }}
                </span>
                <span class="text-[#7ec8e3] font-bold min-w-12 text-right shrink-0">+{{ pref.pref.toFixed(0) }}</span>
              </div>
            </div>
          </div>

          <!-- Gift Items -->
          <div v-if="selected.item_gifts.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Favorite Gift Items ({{ selected.item_gifts.length }})</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="gift in selected.item_gifts"
                :key="gift"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1e1a2e] border border-[#3a2a4e] text-[#b8a8c8]">
                {{ gift }}
              </span>
            </div>
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
import { ref, watch, computed } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useKeyboard } from "../../composables/useKeyboard";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import type { NpcInfo } from "../../types/gameData";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const store = useGameDataStore();

const query = ref("");
const selectedArea = ref<string>("All Areas");
const selected = ref<NpcInfo | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);

const allNpcs = computed(() =>
  Object.values(store.npcsByKey).sort((a, b) => a.name.localeCompare(b.name))
);
const loading = computed(() => allNpcs.value.length === 0 && store.status === 'loading');
const filteredNpcs = ref<NpcInfo[]>([]);

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

// Filter NPCs based on search query, area, and when data loads
watch([query, selectedArea, allNpcs], () => {
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
  selectedIndex.value = 0;
}, { immediate: true });

async function selectNpc(npc: NpcInfo) {
  selected.value = npc;
}

function clearSelection() {
  selected.value = null;
}

useKeyboard({
  listNavigation: {
    items: filteredNpcs,
    selectedIndex,
    onConfirm: (index: number) => selectNpc(filteredNpcs.value[index]),
    scrollContainerRef: listRef,
  },
});

// Navigate to a specific NPC when navTarget changes
watch(() => props.navTarget, (target) => {
  if (!target || target.type !== 'npc') return;
  const key = String(target.id);
  if (selected.value?.key === key) return;

  const match = store.resolveNpcSync(key);
  if (match) {
    query.value = match.name;
    selectNpc(match);
  }
}, { immediate: true });
</script>
