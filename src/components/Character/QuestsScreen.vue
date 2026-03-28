<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <!-- Summary bar -->
    <div v-if="enrichedQuests.length > 0" class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <span class="text-sm text-text-primary font-semibold">
          {{ settingsStore.settings.activeCharacterName ?? 'Character' }}
        </span>
        <span v-if="settingsStore.settings.activeServerName" class="text-xs text-text-muted">
          {{ settingsStore.settings.activeServerName }}
        </span>
      </div>
      <div class="flex items-center gap-4 text-xs">
        <div>
          <span class="text-text-muted">Active Quests:</span>
          <span class="text-accent-gold font-bold ml-1">{{ activeCount }}</span>
        </div>
        <div>
          <span class="text-text-muted">Work Orders:</span>
          <span class="text-entity-area font-bold ml-1">{{ workOrderCount }}</span>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <EmptyState
      v-if="!characterStore.selectedSnapshot && !characterStore.loading"
      primary="No character data loaded."
      secondary="Import a character report to see your active quests." />

    <EmptyState
      v-else-if="characterStore.loading || loading"
      primary="Loading quest data..."
      secondary="Resolving quest details from game data." />

    <EmptyState
      v-else-if="characterStore.activeQuests.length === 0"
      primary="No active quests found."
      secondary="Your character report didn't include any active quests. Try re-importing." />

    <!-- Two-panel layout -->
    <div v-else class="flex gap-3 flex-1 min-h-0">
      <div class="w-80 shrink-0 flex flex-col min-h-0">
        <QuestListPanel
          :quests="enrichedQuests"
          :skills-by-name="gameState.skillsByName"
          :favor-by-npc="gameState.favorByNpc"
          :selected-quest-key="selectedQuestKey"
          :quest-categories="questCategories"
          @select="selectQuest" />
      </div>

      <QuestDetailPanel
        :quest="selectedQuest"
        :skills-by-name="gameState.skillsByName"
        :favor-by-npc="gameState.favorByNpc" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useCharacterStore } from '../../stores/characterStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useSettingsStore } from '../../stores/settingsStore'
import type { QuestInfo } from '../../types/gameData'
import EmptyState from '../Shared/EmptyState.vue'
import QuestListPanel from './QuestListPanel.vue'
import QuestDetailPanel from './QuestDetailPanel.vue'

const characterStore = useCharacterStore()
const gameData = useGameDataStore()
const gameState = useGameStateStore()
const settingsStore = useSettingsStore()

const selectedQuestKey = ref<string | null>(null)
const loading = ref(false)
const resolvedQuests = ref<Map<string, QuestInfo>>(new Map())

const enrichedQuests = computed<QuestInfo[]>(() => {
  const quests: QuestInfo[] = []
  for (const aq of characterStore.activeQuests) {
    const resolved = resolvedQuests.value.get(aq.quest_key)
    if (resolved) {
      quests.push(resolved)
    } else {
      // Fallback: show quest key even if CDN data unavailable
      quests.push({
        internal_name: aq.quest_key,
        raw: { InternalName: aq.quest_key },
      })
    }
  }
  return quests
})

const selectedQuest = computed<QuestInfo | null>(() => {
  if (!selectedQuestKey.value) return null
  return enrichedQuests.value.find(q => q.internal_name === selectedQuestKey.value) ?? null
})

const activeCount = computed(() =>
  characterStore.activeQuests.filter(q => q.category === 'active').length
)

const workOrderCount = computed(() =>
  characterStore.activeQuests.filter(q => q.category === 'work_order').length
)

const questCategories = computed(() => {
  const map = new Map<string, string>()
  for (const aq of characterStore.activeQuests) {
    map.set(aq.quest_key, aq.category)
  }
  return map
})

function selectQuest(key: string) {
  selectedQuestKey.value = key
}

async function resolveActiveQuests() {
  if (gameData.status !== 'ready' || characterStore.activeQuests.length === 0) return

  loading.value = true
  try {
    const map = new Map<string, QuestInfo>()
    // Resolve all quests in parallel batches
    const results = await Promise.all(
      characterStore.activeQuests.map(aq => gameData.resolveQuest(aq.quest_key))
    )
    for (let i = 0; i < results.length; i++) {
      if (results[i]) {
        map.set(characterStore.activeQuests[i].quest_key, results[i]!)
      }
    }
    resolvedQuests.value = map
  } catch (e) {
    console.error('[QuestsScreen] Failed to resolve quests:', e)
  } finally {
    loading.value = false
  }
}

// Resolve when active quests or game data changes
watch(
  [() => characterStore.activeQuests, () => gameData.status],
  () => { resolveActiveQuests() },
  { immediate: true },
)
</script>
