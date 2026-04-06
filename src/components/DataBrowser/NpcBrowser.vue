<template>
  <PaneLayout screen-key="db-npcs" :left-pane="{ title: 'NPCs', defaultWidth: 360, minWidth: 280, maxWidth: 500 }">
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
      </template>
    </template>

    <!-- Right panel: NPC detail -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated p-4 flex flex-col gap-4"
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
                  <AreaInline :reference="selected.area_name" /></template
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
            <div class="flex flex-wrap gap-1.5">
              <SkillInline v-for="skill in selected.trains_skills" :key="skill" :reference="skill" />
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
                <span v-if="pref.name" class="flex-1">
                  <ItemInline :reference="pref.name" />
                </span>
                <span v-else-if="pref.keywords.length" class="text-text-secondary flex-1">
                  {{ pref.keywords.join(', ') }}
                </span>
                <span class="text-[#7ec8e3] font-bold min-w-12 text-right shrink-0">+{{ pref.pref.toFixed(0) }}</span>
              </div>
            </div>
          </div>

          <!-- Gift Favor Tiers -->
          <div v-if="selected.gift_favor_tiers.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Gives Gifts At</div>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="tier in selected.gift_favor_tiers"
                :key="tier"
                class="text-[0.72rem] px-1.5 py-0.5 rounded-sm border"
                :class="favorBadgeClasses(tier)">
                {{ tierDisplayName(tier) }}
              </span>
            </div>
          </div>

          <!-- Associated Quests -->
          <div v-if="associatedQuests.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Associated Quests ({{ associatedQuests.length }})</div>
            <ul class="m-0 p-0 list-none max-h-60 overflow-y-auto border border-surface-dark">
              <li
                v-for="quest in associatedQuests"
                :key="quest.internal_name"
                class="text-xs px-2 py-0.5 flex gap-2 items-center border-b border-[#151515] hover:bg-surface-base">
                <QuestInline :reference="quest.internal_name" />
              </li>
            </ul>
          </div>

          <!-- Vendor Inventory -->
          <div v-if="vendorItems.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Sells Items ({{ vendorItems.length }})</div>
            <div class="max-h-80 overflow-y-auto border border-surface-dark">
              <table class="w-full text-xs border-collapse">
                <thead class="sticky top-0 bg-surface-base">
                  <tr class="text-left text-text-dim border-b border-border-default">
                    <th class="py-1 px-2 font-normal">Item</th>
                    <th class="py-1 px-2 font-normal text-right">Value</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="item in vendorItems"
                    :key="item.item_id"
                    class="border-b border-[#151515] hover:bg-surface-base">
                    <td class="py-0.5 px-2">
                      <ItemInline :reference="String(item.item_id)" />
                    </td>
                    <td class="py-0.5 px-2 text-right text-text-muted font-mono">
                      <template v-if="item.value">{{ Math.ceil(item.value * 1.5) }}c</template>
                      <span v-else class="text-text-dim">--</span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Raw JSON -->
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
import { invoke } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import type { NpcInfo, QuestInfo } from "../../types/gameData";

interface VendorItemSummary {
  item_id: number;
  name: string;
  value: number | null;
  icon_id: number | null;
}
import QuestInline from "../Shared/Quest/QuestInline.vue";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import AreaInline from "../Shared/Area/AreaInline.vue";
import ItemInline from "../Shared/Item/ItemInline.vue";
import { tierDisplayName, favorBadgeClasses } from "../../composables/useFavorTiers";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const store = useGameDataStore();
const settingsStore = useSettingsStore();

const query = ref("");
const selectedArea = ref<string>("All Areas");
const selected = ref<NpcInfo | null>(null);
const associatedQuests = ref<QuestInfo[]>([]);
const vendorItems = ref<VendorItemSummary[]>([]);
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
  associatedQuests.value = [];
  vendorItems.value = [];

  store.getQuestsForNpc(npc.key)
    .then(quests => { associatedQuests.value = quests; })
    .catch(e => { console.warn("Quest cross-ref fetch failed:", e); });

  invoke<VendorItemSummary[]>('get_npc_vendor_items', { npcKey: npc.key })
    .then(items => { vendorItems.value = items; })
    .catch(e => { console.warn("Vendor items fetch failed:", e); });
}

function clearSelection() {
  selected.value = null;
  associatedQuests.value = [];
  vendorItems.value = [];
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
