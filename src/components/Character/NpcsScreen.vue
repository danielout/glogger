<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <!-- Summary bar -->
    <div v-if="hasData" class="flex items-center justify-between">
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
          <span class="text-text-muted">Known NPCs:</span>
          <span class="text-accent-gold font-bold ml-1">{{ totalNpcs }}</span>
        </div>
        <div>
          <span class="text-text-muted">Above Neutral:</span>
          <span class="text-green-400 font-bold ml-1">{{ aboveNeutral }}</span>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <EmptyState
      v-if="!hasData"
      primary="No NPC data loaded."
      secondary="Log in to a character or import a character report to see NPC relationships." />

    <!-- Two-panel layout -->
    <PaneLayout
      v-else
      screen-key="npcs"
      :left-pane="{ title: 'NPC List', defaultWidth: 320, minWidth: 240, maxWidth: 500 }">
      <template #left>
        <NpcListPanel
          :snapshot-favor="characterStore.npcFavor"
          :favor-by-npc="gameState.favorByNpc"
          :npcs-by-key="gameData.npcsByKey"
          :selected-npc-key="selectedNpcKey"
          @select="selectNpc" />
      </template>

      <NpcDetailPanel
        :npc-key="selectedNpcKey"
        :snapshot-tier="selectedSnapshotTier"
        :gamestate-favor="selectedGamestateFavor"
        :cdn-data="selectedCdnData" />
    </PaneLayout>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useCharacterStore } from '../../stores/characterStore'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import { useSettingsStore } from '../../stores/settingsStore'
import EmptyState from '../Shared/EmptyState.vue'
import PaneLayout from '../Shared/PaneLayout.vue'
import NpcListPanel from './NpcListPanel.vue'
import NpcDetailPanel from './NpcDetailPanel.vue'

const characterStore = useCharacterStore()
const gameState = useGameStateStore()
const gameData = useGameDataStore()
const settingsStore = useSettingsStore()

const selectedNpcKey = ref<string | null>(null)

const hasData = computed(() =>
  characterStore.npcFavor.length > 0 || gameState.favor.length > 0
)

const totalNpcs = computed(() => {
  const keys = new Set<string>()
  for (const f of characterStore.npcFavor) keys.add(f.npc_key)
  for (const f of gameState.favor) keys.add(f.npc_key)
  return keys.size
})

const aboveNeutral = computed(() => {
  const keys = new Set<string>()
  for (const f of characterStore.npcFavor) {
    if (f.favor_level !== 'Neutral') keys.add(f.npc_key)
  }
  for (const f of gameState.favor) {
    if (f.favor_tier && f.favor_tier !== 'Neutral') keys.add(f.npc_key)
  }
  return keys.size
})

const selectedSnapshotTier = computed(() => {
  if (!selectedNpcKey.value) return null
  const snap = characterStore.npcFavor.find(f => f.npc_key === selectedNpcKey.value)
  return snap?.favor_level ?? null
})

const selectedGamestateFavor = computed(() => {
  if (!selectedNpcKey.value) return null
  return gameState.favorByNpc[selectedNpcKey.value] ?? null
})

const selectedCdnData = computed(() => {
  if (!selectedNpcKey.value) return null
  return gameData.npcsByKey[selectedNpcKey.value] ?? null
})

function selectNpc(key: string) {
  selectedNpcKey.value = key
}
</script>
