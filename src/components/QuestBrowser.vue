<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useGameDataStore } from "../stores/gameDataStore";
import type { QuestInfo, QuestObjective, QuestReward, QuestRewardItem, QuestRequirement } from "../types/gameData";

const store = useGameDataStore();

const query = ref("");
const allQuests = ref<QuestInfo[]>([]);
const selected = ref<QuestInfo | null>(null);
const loading = ref(false);

// Filters
const filterArea = ref<string>("all");
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

// Filtered and sorted quests
const filteredQuests = computed(() => {
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

function selectQuest(quest: QuestInfo) {
  selected.value = quest;
}

function clearSelection() {
  selected.value = null;
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
</script>

<template>
  <div class="quest-browser">
    <!-- Status banner if data not ready -->
    <div v-if="store.status !== 'ready'" class="status-banner">
      <span v-if="store.status === 'loading'" class="status-loading"
        >⟳ Loading game data…</span
      >
      <span v-else-if="store.status === 'error'" class="status-error"
        >✕ {{ store.errorMessage }}</span
      >
    </div>

    <div v-else class="browser-layout">
      <!-- Left panel: search + filters + results -->
      <div class="search-panel">
        <!-- Search bar -->
        <div class="search-bar">
          <input
            v-model="query"
            class="search-input"
            placeholder="Search quests…"
            autofocus />
          <span v-if="loading" class="search-spinner">⟳</span>
          <span v-else-if="filteredQuests.length" class="search-count">{{
            filteredQuests.length
          }}</span>
        </div>

        <!-- Filters -->
        <div class="filters">
          <div class="filter-row">
            <label class="filter-label">Area:</label>
            <select v-model="filterArea" class="filter-select">
              <option value="all">All Areas</option>
              <option v-for="area in availableAreas" :key="area" :value="area">
                {{ area }}
              </option>
            </select>
          </div>

          <div class="filter-row">
            <label class="filter-label">Sort:</label>
            <select v-model="sortBy" class="filter-select">
              <option value="name">Name</option>
              <option value="level">Level</option>
              <option value="area">Area</option>
            </select>
          </div>

          <div class="filter-row">
            <label class="filter-label">Cancellable:</label>
            <select v-model="filterCancellable" class="filter-select">
              <option value="all">All</option>
              <option value="yes">Yes</option>
              <option value="no">No</option>
            </select>
          </div>
        </div>

        <!-- Results -->
        <div v-if="!allQuests.length && !loading" class="search-hint">
          No quests loaded
        </div>

        <div v-else-if="filteredQuests.length === 0" class="search-hint">
          No quests found
        </div>

        <ul v-else class="results-list">
          <li
            v-for="quest in filteredQuests"
            :key="quest.internal_name"
            class="result-row"
            :class="{ active: selected?.internal_name === quest.internal_name }"
            @click="selectQuest(quest)">
            <div class="result-main">
              <span class="result-name">{{ getDisplayName(quest) }}</span>
              <div class="result-meta">
                <span v-if="getLevel(quest)" class="meta-tag level">Lv {{ getLevel(quest) }}</span>
                <span v-if="getArea(quest)" class="meta-tag area">{{ getArea(quest) }}</span>
              </div>
            </div>
          </li>
        </ul>
      </div>

      <!-- Right panel: quest detail -->
      <div class="detail-panel" :class="{ empty: !selected }">
        <div v-if="!selected" class="detail-empty">
          Select a quest to view details
        </div>

        <template v-else>
          <!-- Header -->
          <div class="detail-header">
            <div class="detail-title-block">
              <div class="detail-name">{{ getDisplayName(selected) }}</div>
              <div class="detail-meta">
                <span class="mono">{{ selected.internal_name }}</span>
                <template v-if="getLevel(selected)">
                  · Level {{ getLevel(selected) }}
                </template>
                <template v-if="getArea(selected)">
                  · {{ getArea(selected) }}
                </template>
              </div>
            </div>
            <button class="close-btn" @click="clearSelection">✕</button>
          </div>

          <!-- Description -->
          <div v-if="selected.raw?.Description" class="detail-section">
            <div class="quest-description">
              {{ selected.raw.Description }}
            </div>
          </div>

          <!-- Preface Text -->
          <div v-if="selected.raw?.PrefaceText" class="detail-section">
            <div class="section-label">Quest Giver Dialog</div>
            <div class="quest-dialog">{{ selected.raw.PrefaceText }}</div>
          </div>

          <!-- Quest Info -->
          <div class="detail-section">
            <div class="section-label">Quest Info</div>
            <div class="info-grid">
              <div v-if="selected.raw?.IsCancellable !== undefined" class="info-item">
                <span class="info-key">Cancellable:</span>
                <span class="info-value">{{ selected.raw.IsCancellable ? "Yes" : "No" }}</span>
              </div>
              <div v-if="formatReuseTime(selected)" class="info-item">
                <span class="info-key">Reuse Time:</span>
                <span class="info-value">{{ formatReuseTime(selected) }}</span>
              </div>
              <div v-if="selected.raw?.FavorNpc" class="info-item">
                <span class="info-key">Favor NPC:</span>
                <span class="info-value">{{ selected.raw.FavorNpc.split('/').pop() }}</span>
              </div>
              <div v-if="selected.raw?.WorkOrderSkill" class="info-item">
                <span class="info-key">Work Order:</span>
                <span class="info-value">{{ selected.raw.WorkOrderSkill }}</span>
              </div>
            </div>
          </div>

          <!-- Requirements -->
          <div v-if="selected.raw?.Requirements?.length" class="detail-section">
            <div class="section-label">Requirements</div>
            <ul class="requirement-list">
              <li v-for="(req, idx) in selected.raw.Requirements" :key="idx" class="requirement-item">
                {{ getRequirementDisplay(req) }}
              </li>
            </ul>
          </div>

          <!-- Objectives -->
          <div v-if="selected.raw?.Objectives?.length" class="detail-section">
            <div class="section-label">Objectives</div>
            <ul class="objective-list">
              <li v-for="(obj, idx) in selected.raw.Objectives" :key="idx" class="objective-item">
                <span class="objective-type">{{ getObjectiveTypeDisplay(obj.Type) }}</span>
                <span class="objective-desc">{{ obj.Description }}</span>
                <span v-if="obj.Number" class="objective-number">({{ obj.Number }})</span>
              </li>
            </ul>
          </div>

          <!-- Rewards -->
          <div v-if="selected.raw?.Rewards?.length || selected.raw?.Rewards_Items?.length || selected.raw?.Reward_Favor" class="detail-section">
            <div class="section-label">Rewards</div>

            <div v-if="selected.raw?.Reward_Favor" class="reward-favor">
              Favor: {{ selected.raw.Reward_Favor }}
            </div>

            <ul v-if="selected.raw?.Rewards?.length" class="reward-list">
              <li v-for="(reward, idx) in selected.raw.Rewards" :key="idx" class="reward-item">
                {{ getRewardTypeDisplay(reward) }}
              </li>
            </ul>

            <ul v-if="selected.raw?.Rewards_Items?.length" class="reward-list">
              <li v-for="(item, idx) in selected.raw.Rewards_Items" :key="idx" class="reward-item">
                {{ item.Item }} × {{ item.StackSize }}
              </li>
            </ul>

            <div v-if="selected.raw?.Rewards_NamedLootProfile" class="reward-loot">
              Loot Profile: {{ selected.raw.Rewards_NamedLootProfile }}
            </div>
          </div>

          <!-- Success Text -->
          <div v-if="selected.raw?.SuccessText" class="detail-section">
            <div class="section-label">Completion Dialog</div>
            <div class="quest-dialog">{{ selected.raw.SuccessText }}</div>
          </div>

          <!-- Keywords (if any) -->
          <div v-if="selected.raw?.Keywords?.length" class="detail-section">
            <div class="section-label">Keywords</div>
            <div class="keyword-list">
              <span v-for="kw in selected.raw.Keywords" :key="kw" class="keyword-tag">{{ kw }}</span>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quest-browser {
  height: calc(100vh - 130px);
  display: flex;
  flex-direction: column;
}

.status-banner {
  padding: 1rem;
  font-size: 0.9rem;
}
.status-loading {
  color: #e0c060;
}
.status-error {
  color: #f66;
}

.browser-layout {
  display: flex;
  gap: 1rem;
  height: 100%;
  overflow: hidden;
}

/* ── Left panel ── */
.search-panel {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow: hidden;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  position: relative;
}
.search-input {
  flex: 1;
  padding: 0.45rem 0.6rem;
  background: #1a1a1a;
  border: 1px solid #444;
  color: #ccc;
  font-family: monospace;
  font-size: 0.9rem;
  outline: none;
}
.search-input:focus {
  border-color: #e0c060;
}
.search-spinner {
  color: #e0c060;
  font-size: 0.9rem;
  animation: spin 1s linear infinite;
}
.search-count {
  color: #555;
  font-size: 0.8rem;
  min-width: 1.5rem;
  text-align: right;
}

/* Filters */
.filters {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  padding: 0.5rem;
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
}
.filter-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.filter-label {
  font-size: 0.75rem;
  color: #777;
  min-width: 4.5rem;
}
.filter-select {
  flex: 1;
  padding: 0.3rem 0.4rem;
  background: #0d0d0d;
  border: 1px solid #333;
  color: #aaa;
  font-size: 0.8rem;
  font-family: monospace;
  outline: none;
}
.filter-select:focus {
  border-color: #e0c060;
}

.search-hint {
  color: #444;
  font-size: 0.8rem;
  font-style: italic;
  padding: 0.25rem 0;
}

.results-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
  border: 1px solid #2a2a2a;
}
.result-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.5rem 0.6rem;
  cursor: pointer;
  border-bottom: 1px solid #1a1a1a;
}
.result-row:hover {
  background: #1e1e1e;
}
.result-row.active {
  background: #1a1a2e;
  border-left: 2px solid #e0c060;
}

.result-main {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}
.result-name {
  color: #bbb;
  font-size: 0.85rem;
}
.result-meta {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}
.meta-tag {
  font-size: 0.7rem;
  padding: 0.1rem 0.3rem;
  border-radius: 2px;
}
.meta-tag.level {
  background: #2a2a1a;
  color: #888;
}
.meta-tag.area {
  background: #1a2a2a;
  color: #6a9fb5;
}

/* ── Right panel ── */
.detail-panel {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #2a2a2a;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.detail-panel.empty {
  align-items: center;
  justify-content: center;
}
.detail-empty {
  color: #333;
  font-style: italic;
}

.detail-header {
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid #2a2a2a;
}

.detail-title-block {
  flex: 1;
  min-width: 0;
}
.detail-name {
  color: #e0c060;
  font-size: 1.1rem;
  font-weight: bold;
  margin-bottom: 0.25rem;
}
.detail-meta {
  font-size: 0.75rem;
  color: #555;
}
.mono {
  color: #888;
  font-family: monospace;
}

.close-btn {
  background: none;
  border: none;
  color: #444;
  cursor: pointer;
  padding: 0 0.25rem;
  font-size: 0.9rem;
  flex-shrink: 0;
}
.close-btn:hover {
  color: #f66;
  background: none;
  border: none;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.section-label {
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #555;
  border-bottom: 1px solid #222;
  padding-bottom: 0.2rem;
}

.quest-description {
  font-size: 0.9rem;
  color: #bbb;
  font-style: italic;
  line-height: 1.4;
}

.quest-dialog {
  font-size: 0.85rem;
  color: #999;
  line-height: 1.5;
  padding: 0.5rem 0.75rem;
  background: #1a1a1a;
  border-left: 3px solid #4a4a2a;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 0.5rem;
}
.info-item {
  font-size: 0.8rem;
  display: flex;
  gap: 0.5rem;
}
.info-key {
  color: #666;
  min-width: 6rem;
}
.info-value {
  color: #aaa;
}

.requirement-list,
.objective-list,
.reward-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.requirement-item {
  font-size: 0.82rem;
  color: #e08060;
  padding-left: 1rem;
  position: relative;
}
.requirement-item::before {
  content: "◆";
  position: absolute;
  left: 0;
  color: #e08060;
}

.objective-item {
  font-size: 0.82rem;
  color: #aaa;
  display: flex;
  gap: 0.5rem;
  align-items: baseline;
}
.objective-type {
  color: #6a9fb5;
  font-weight: bold;
  min-width: 5rem;
  font-size: 0.75rem;
}
.objective-desc {
  flex: 1;
}
.objective-number {
  color: #666;
  font-size: 0.75rem;
}

.reward-list {
  margin-bottom: 0.3rem;
}
.reward-item {
  font-size: 0.82rem;
  color: #60e090;
  padding-left: 1rem;
  position: relative;
}
.reward-item::before {
  content: "✦";
  position: absolute;
  left: 0;
  color: #60e090;
}

.reward-favor {
  font-size: 0.85rem;
  color: #c0a0e0;
  font-weight: bold;
  margin-bottom: 0.3rem;
}

.reward-loot {
  font-size: 0.8rem;
  color: #888;
  font-style: italic;
  margin-top: 0.3rem;
}

.keyword-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.keyword-tag {
  font-size: 0.7rem;
  padding: 0.2rem 0.5rem;
  background: #1a1a2a;
  color: #8888bb;
  border-radius: 3px;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
