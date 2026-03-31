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
      <!-- Left panel: search + filters + results -->
      <div class="w-90 shrink-0 flex flex-col gap-2 overflow-hidden">
        <!-- Search bar -->
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search quests…"
            autofocus />
          <span v-if="loading" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="filteredQuests.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredQuests.length
          }}</span>
        </div>

        <!-- Filters -->
        <div class="flex flex-col gap-1.5 p-2 bg-surface-base border border-surface-elevated">
          <div class="flex items-center gap-2">
            <label class="text-xs text-text-muted min-w-18">Area:</label>
            <select v-model="filterArea" class="input flex-1 text-sm">
              <option value="all">All Areas</option>
              <option v-for="area in availableAreas" :key="area" :value="area">
                {{ area }}
              </option>
            </select>
          </div>

          <div class="flex items-center gap-2">
            <label class="text-xs text-text-muted min-w-18">NPC:</label>
            <select v-model="filterNpc" class="input flex-1 text-sm">
              <option value="all">All NPCs</option>
              <option v-for="npc in availableNpcs" :key="npc.key" :value="npc.key">
                {{ npc.displayName }}
              </option>
            </select>
          </div>

          <div class="flex items-center gap-2">
            <label class="text-xs text-text-muted min-w-18">Sort:</label>
            <select v-model="sortBy" class="input flex-1 text-sm">
              <option value="name">Name</option>
              <option value="level">Level</option>
              <option value="area">Area</option>
            </select>
          </div>

          <div class="flex items-center gap-2">
            <label class="text-xs text-text-muted min-w-18">Cancellable:</label>
            <select v-model="filterCancellable" class="input flex-1 text-sm">
              <option value="all">All</option>
              <option value="yes">Yes</option>
              <option value="no">No</option>
            </select>
          </div>
        </div>

        <!-- Results -->
        <div v-if="!query && filterArea === 'all' && filterNpc === 'all'" class="text-text-dim text-xs italic py-1">
          Start typing to search {{ allQuests.length ? allQuests.length.toLocaleString() : '…' }} quests, or select a filter
        </div>

        <div v-else-if="filteredQuests.length === 0 && !loading" class="text-text-dim text-xs italic py-1">
          No quests found
        </div>

        <ul ref="listRef" v-else class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(quest, idx) in filteredQuests"
            :key="quest.internal_name"
            class="flex flex-col gap-1 px-2.5 py-2 cursor-pointer border-b border-surface-dark hover:bg-[#1e1e1e]"
            :class="{
              'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.internal_name === quest.internal_name,
              'bg-surface-elevated': selectedIndex === idx && selected?.internal_name !== quest.internal_name
            }"
            @click="selectQuest(quest)">
            <div class="flex flex-col gap-0.5">
              <span class="text-text-primary/75 text-[0.85rem]">{{ getDisplayName(quest) }}</span>
              <div class="flex gap-1.5 flex-wrap">
                <span v-if="getLevel(quest)" class="text-[0.7rem] px-1 py-0.5 rounded-sm bg-[#2a2a1a] text-text-secondary">Lv {{ getLevel(quest) }}</span>
                <span v-if="getArea(quest)" class="text-[0.7rem] px-1 py-0.5 rounded-sm bg-[#1a2a2a] text-[#6a9fb5]">{{ getArea(quest) }}</span>
              </div>
            </div>
          </li>
        </ul>
      </div>

      <!-- Right panel: quest detail -->
      <div
        class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
        :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select a quest to view details
        </div>

        <template v-else>
          <!-- Header -->
          <div class="flex gap-3 items-start pb-3 border-b border-surface-elevated">
            <div class="flex-1 min-w-0">
              <div class="text-accent-gold text-lg font-bold mb-1">{{ getDisplayName(selected) }}</div>
              <div class="text-xs text-text-dim">
                <span class="text-text-secondary font-mono">{{ selected.internal_name }}</span>
                <template v-if="getLevel(selected)">
                  · Level {{ getLevel(selected) }}
                </template>
                <template v-if="getArea(selected)">
                  · {{ getArea(selected) }}
                </template>
              </div>
            </div>
            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Description -->
          <div v-if="selected.raw?.Description" class="flex flex-col gap-2">
            <div class="text-sm text-text-primary/75 italic leading-relaxed">
              {{ selected.raw.Description }}
            </div>
          </div>

          <!-- Preface Text -->
          <div v-if="selected.raw?.PrefaceText" class="flex flex-col gap-2">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Quest Giver Dialog</div>
            <div class="text-[0.85rem] text-text-secondary leading-relaxed px-3 py-2 bg-surface-base border-l-3 border-l-[#4a4a2a]">{{ selected.raw.PrefaceText }}</div>
          </div>

          <!-- Quest Info -->
          <div class="flex flex-col gap-2">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Quest Info</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-2">
              <div v-if="selected.raw?.IsCancellable !== undefined" class="text-sm flex gap-2">
                <span class="text-text-muted min-w-24">Cancellable:</span>
                <span class="text-text-secondary">{{ selected.raw.IsCancellable ? "Yes" : "No" }}</span>
              </div>
              <div v-if="formatReuseTime(selected)" class="text-sm flex gap-2">
                <span class="text-text-muted min-w-24">Reuse Time:</span>
                <span class="text-text-secondary">{{ formatReuseTime(selected) }}</span>
              </div>
              <div v-if="selected.raw?.FavorNpc" class="text-sm flex gap-2">
                <span class="text-text-muted min-w-24">Favor NPC:</span>
                <span class="text-text-secondary">{{ selected.raw.FavorNpc.split('/').pop() }}</span>
              </div>
              <div v-if="selected.raw?.WorkOrderSkill" class="text-sm flex gap-2">
                <span class="text-text-muted min-w-24">Work Order:</span>
                <span class="text-text-secondary">{{ selected.raw.WorkOrderSkill }}</span>
              </div>
            </div>
          </div>

          <!-- Requirements -->
          <div v-if="selected.raw?.Requirements?.length" class="flex flex-col gap-2">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Requirements</div>
            <ul class="list-none m-0 p-0 flex flex-col gap-1">
              <li v-for="(req, idx) in selected.raw.Requirements" :key="idx" class="text-[0.82rem] text-[#e08060] pl-4 relative before:content-['◆'] before:absolute before:left-0 before:text-[#e08060]">
                {{ getRequirementDisplay(req) }}
              </li>
            </ul>
          </div>

          <!-- Objectives -->
          <div v-if="selected.raw?.Objectives?.length" class="flex flex-col gap-2">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Objectives</div>
            <ul class="list-none m-0 p-0 flex flex-col gap-1">
              <li v-for="(obj, idx) in selected.raw.Objectives" :key="idx" class="text-[0.82rem] text-text-secondary flex gap-2 items-baseline">
                <span class="text-[#6a9fb5] font-bold min-w-20 text-xs">{{ getObjectiveTypeDisplay(obj.Type) }}</span>
                <span class="flex-1">{{ obj.Description }}</span>
                <span v-if="obj.Number" class="text-text-muted text-xs">({{ obj.Number }})</span>
              </li>
            </ul>
          </div>

          <!-- Rewards -->
          <div v-if="selected.raw?.Rewards?.length || selected.raw?.Rewards_Items?.length || selected.raw?.Reward_Favor" class="flex flex-col gap-2">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Rewards</div>

            <div v-if="selected.raw?.Reward_Favor" class="text-[0.85rem] text-[#c0a0e0] font-bold mb-1">
              Favor: {{ selected.raw.Reward_Favor }}
            </div>

            <ul v-if="selected.raw?.Rewards?.length" class="list-none m-0 p-0 flex flex-col gap-1 mb-1">
              <li v-for="(reward, idx) in selected.raw.Rewards" :key="idx" class="text-[0.82rem] text-[#60e090] pl-4 relative before:content-['✦'] before:absolute before:left-0 before:text-[#60e090]">
                {{ getRewardTypeDisplay(reward) }}
              </li>
            </ul>

            <ul v-if="selected.raw?.Rewards_Items?.length" class="list-none m-0 p-0 flex flex-col gap-1">
              <li v-for="(item, idx) in selected.raw.Rewards_Items" :key="idx" class="text-[0.82rem] text-[#60e090] pl-4 relative before:content-['✦'] before:absolute before:left-0 before:text-[#60e090]">
                {{ item.Item }} × {{ item.StackSize }}
              </li>
            </ul>

            <div v-if="selected.raw?.Rewards_NamedLootProfile" class="text-sm text-text-secondary italic mt-1">
              Loot Profile: {{ selected.raw.Rewards_NamedLootProfile }}
            </div>
          </div>

          <!-- Success Text -->
          <div v-if="selected.raw?.SuccessText" class="flex flex-col gap-2">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Completion Dialog</div>
            <div class="text-[0.85rem] text-text-secondary leading-relaxed px-3 py-2 bg-surface-base border-l-3 border-l-[#4a4a2a]">{{ selected.raw.SuccessText }}</div>
          </div>

          <!-- Sources -->
          <SourcesPanel :sources="sources" :loading="sourcesLoading" />

          <!-- Keywords (if any) -->
          <div v-if="selected.raw?.Keywords?.length" class="flex flex-col gap-2">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span v-for="kw in selected.raw.Keywords" :key="kw" class="text-[0.7rem] px-2 py-0.5 bg-[#1a1a2a] text-[#8888bb] rounded-sm">{{ kw }}</span>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useKeyboard } from "../../composables/useKeyboard";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import type { QuestInfo, QuestReward, QuestRequirement, EntitySources } from "../../types/gameData";
import SourcesPanel from "../Shared/SourcesPanel.vue";
import { extractNpcKeyFromFavorPath } from "../../utils/questDisplay";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const store = useGameDataStore();

const query = ref("");
const allQuests = ref<QuestInfo[]>([]);
const selected = ref<QuestInfo | null>(null);
const sources = ref<EntitySources | null>(null);
const sourcesLoading = ref(false);
const loading = ref(false);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);

// Filters
const filterArea = ref<string>("all");
const filterNpc = ref<string>("all");
const filterCancellable = ref<string>("all");
const sortBy = ref<"name" | "level" | "area">("name");

onMounted(async () => {
  if (store.status === "ready") {
    await loadAllQuests();
  }
});

watch(() => store.status, async (newStatus) => {
  if (newStatus === "ready") {
    await loadAllQuests();
  }
});

async function loadAllQuests() {
  loading.value = true;
  try {
    const quests = await store.getAllQuests();
    allQuests.value = quests;
  } finally {
    loading.value = false;
  }
}

// Get unique areas for filtering
const availableAreas = computed(() => {
  const areas = new Set<string>();
  allQuests.value.forEach(q => {
    const area = q.raw?.DisplayedLocation;
    if (area) areas.add(area);
  });
  return Array.from(areas).sort();
});

// Get unique NPCs for filtering (scoped to selected area if one is chosen)
const availableNpcs = computed(() => {
  const npcMap = new Map<string, string>(); // key -> displayName
  let quests = allQuests.value;
  if (filterArea.value !== "all") {
    quests = quests.filter(q => q.raw?.DisplayedLocation === filterArea.value);
  }
  quests.forEach(q => {
    const favorNpc = q.raw?.FavorNpc;
    if (!favorNpc) return;
    const npcKey = extractNpcKeyFromFavorPath(favorNpc);
    if (npcMap.has(npcKey)) return;
    const resolved = store.resolveNpcSync(npcKey);
    const displayName = resolved?.name || npcKey.replace(/^NPC_/, '').replace(/_/g, ' ');
    npcMap.set(npcKey, displayName);
  });
  return Array.from(npcMap.entries())
    .map(([key, displayName]) => ({ key, displayName }))
    .sort((a, b) => a.displayName.localeCompare(b.displayName));
});

// Filtered and sorted quests
const filteredQuests = computed(() => {
  // Don't show anything until user searches or picks a filter
  if (!query.value.trim() && filterArea.value === "all" && filterNpc.value === "all") {
    return [];
  }

  let filtered = allQuests.value;

  // Text search
  if (query.value.trim()) {
    const q = query.value.toLowerCase();
    filtered = filtered.filter(quest => {
      const name = quest.raw?.Name?.toLowerCase() || "";
      const desc = quest.raw?.Description?.toLowerCase() || "";
      const internal = quest.internal_name.toLowerCase();
      const area = quest.raw?.DisplayedLocation?.toLowerCase() || "";
      return name.includes(q) || desc.includes(q) || internal.includes(q) || area.includes(q);
    });
  }

  // Area filter
  if (filterArea.value !== "all") {
    filtered = filtered.filter(q => q.raw?.DisplayedLocation === filterArea.value);
  }

  // NPC filter
  if (filterNpc.value !== "all") {
    filtered = filtered.filter(q => {
      const favorNpc = q.raw?.FavorNpc;
      if (!favorNpc) return false;
      return extractNpcKeyFromFavorPath(favorNpc) === filterNpc.value;
    });
  }

  // Cancellable filter
  if (filterCancellable.value !== "all") {
    const shouldBeCancellable = filterCancellable.value === "yes";
    filtered = filtered.filter(q => q.raw?.IsCancellable === shouldBeCancellable);
  }

  // Sort
  filtered = [...filtered].sort((a, b) => {
    if (sortBy.value === "name") {
      const aName = a.raw?.Name || a.internal_name;
      const bName = b.raw?.Name || b.internal_name;
      return aName.localeCompare(bName);
    } else if (sortBy.value === "level") {
      const aLevel = a.raw?.Level ?? 999;
      const bLevel = b.raw?.Level ?? 999;
      return aLevel - bLevel;
    } else if (sortBy.value === "area") {
      const aArea = a.raw?.DisplayedLocation || "zzz";
      const bArea = b.raw?.DisplayedLocation || "zzz";
      return aArea.localeCompare(bArea);
    }
    return 0;
  });

  return filtered;
});

// Reset NPC filter when area changes (selected NPC may not exist in new area)
watch(filterArea, () => {
  filterNpc.value = "all";
});

watch(filteredQuests, () => {
  selectedIndex.value = 0;
});

useKeyboard({
  listNavigation: {
    items: filteredQuests,
    selectedIndex,
    onConfirm: (index: number) => {
      const quest = filteredQuests.value[index];
      if (quest) selectQuest(quest);
    },
    scrollContainerRef: listRef,
  },
});

function selectQuest(quest: QuestInfo) {
  selected.value = quest;
  sources.value = null;

  // Load sources (items that bestow this quest)
  sourcesLoading.value = true;
  store.getQuestSources(quest.internal_name)
    .then(s => { sources.value = s; })
    .catch(e => { console.warn("Sources fetch failed:", e); })
    .finally(() => { sourcesLoading.value = false; });
}

function clearSelection() {
  selected.value = null;
  sources.value = null;
}

function getDisplayName(quest: QuestInfo): string {
  return quest.raw?.Name || quest.internal_name || "Unknown Quest";
}

function getLevel(quest: QuestInfo): number | null {
  return quest.raw?.Level ?? null;
}

function getArea(quest: QuestInfo): string | null {
  return quest.raw?.DisplayedLocation ?? null;
}

function getObjectiveTypeDisplay(type: string): string {
  const typeMap: Record<string, string> = {
    Kill: "Kill",
    Collect: "Collect",
    Scripted: "Scripted Event",
    Deliver: "Deliver",
    Harvest: "Harvest",
    Loot: "Loot",
    UseItem: "Use Item",
    InteractionFlag: "Interaction",
    BeAttacked: "Be Attacked",
    GuildEventComplete: "Guild Event"
  };
  return typeMap[type] || type;
}

function getRewardTypeDisplay(reward: QuestReward): string {
  if (reward.T === "SkillXp" && reward.Skill) {
    return `${reward.Skill}: ${reward.Xp} XP`;
  }
  if (reward.T === "CombatXp") {
    return `Combat XP: ${reward.Xp}`;
  }
  if (reward.T === "Currency" && reward.Currency) {
    return `${reward.Amount} ${reward.Currency}`;
  }
  return reward.T;
}

function getRequirementDisplay(req: QuestRequirement): string {
  if (req.T === "QuestCompleted" && req.Quest) {
    return `Quest: ${req.Quest}`;
  }
  if (req.T === "MinFavorLevel" && req.Npc) {
    const npcName = req.Npc.split('/').pop() || req.Npc;
    return `${npcName}: ${req.Level} favor`;
  }
  if (req.T === "MinSkillLevel" && req.Skill) {
    return `${req.Skill} level ${req.MinSkillLevel}`;
  }
  if (req.T === "ActiveCombatSkill" && req.Skill) {
    return `Active skill: ${req.Skill}`;
  }
  return req.T;
}

function formatReuseTime(quest: QuestInfo): string | null {
  if (quest.raw?.ReuseTime_Days) {
    return `${quest.raw.ReuseTime_Days} days`;
  }
  if (quest.raw?.ReuseTime_Minutes) {
    const hours = Math.floor(quest.raw.ReuseTime_Minutes / 60);
    const mins = quest.raw.ReuseTime_Minutes % 60;
    if (hours > 0 && mins > 0) return `${hours}h ${mins}m`;
    if (hours > 0) return `${hours}h`;
    return `${mins}m`;
  }
  return null;
}

// Navigate to a specific quest when navTarget changes
watch(() => props.navTarget, async (target) => {
  if (!target || target.type !== 'quest') return;
  const key = String(target.id);
  if (selected.value?.internal_name === key) return;

  const quest = await store.resolveQuest(key);
  if (quest) {
    query.value = quest.raw?.Name || quest.internal_name;
    selectQuest(quest);
  }
}, { immediate: true });
</script>
