<template>
  <div class="flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-4">
        <h2 class="text-lg font-semibold text-text-primary">
          {{ store.selectedCharacter?.character_name ?? 'Character' }}
        </h2>
        <span v-if="store.selectedCharacter" class="text-sm text-text-muted">
          {{ store.selectedCharacter.server_name }}
        </span>
      </div>
      <div class="flex items-center gap-4">
        <span v-if="store.selectedSnapshot" class="text-xs text-text-muted font-mono">
          Report generated {{ formatTimestamp(store.selectedSnapshot.snapshot_timestamp) }}
        </span>
        <button
          class="px-4 py-2 bg-accent-gold/20 border border-accent-gold/40 text-accent-gold rounded cursor-pointer text-sm font-medium transition-all hover:bg-accent-gold/30"
          :disabled="store.loading"
          @click="handleImport">
          Import Report
        </button>
      </div>
    </div>

    <!-- Import feedback -->
    <div v-if="store.lastImport && !store.lastImport.was_duplicate" class="p-3 bg-green-900/20 border border-green-700/40 rounded text-sm text-green-300">
      Imported {{ store.lastImport.character_name }} ({{ store.lastImport.server_name }}) —
      {{ store.lastImport.skills_imported }} skills,
      {{ store.lastImport.npcs_imported }} NPCs,
      {{ store.lastImport.recipes_imported }} recipes
    </div>

    <div v-if="store.lastImport?.was_duplicate" class="p-3 bg-yellow-900/20 border border-yellow-700/40 rounded text-sm text-yellow-300">
      Snapshot already imported (duplicate).
    </div>

    <div v-if="store.error" class="p-3 bg-red-900/20 border border-red-700/40 rounded text-sm text-red-300">
      {{ store.error }}
    </div>

    <!-- No data state -->
    <div v-if="!store.selectedCharacter && !store.loading" class="text-text-muted text-sm py-8 text-center">
      No character data found. Import a character report to get started.
    </div>

    <!-- Snapshot selector + content -->
    <template v-if="store.selectedCharacter">
      <div class="flex items-center gap-4">
        <label class="text-sm text-text-secondary">Snapshot</label>
        <select
          class="bg-surface-elevated border border-border-default rounded px-3 py-1.5 text-sm text-text-primary font-mono cursor-pointer min-w-70"
          :value="store.selectedSnapshot?.id"
          @change="onSnapshotChange">
          <option v-for="snap in store.snapshots" :key="snap.id" :value="snap.id">
            {{ formatTimestamp(snap.snapshot_timestamp) }} — {{ snap.race }} · {{ snap.skill_count }} skills
          </option>
        </select>
        <span v-if="store.snapshots.length > 1" class="text-xs text-text-muted">
          {{ store.snapshots.length }} snapshots
        </span>
      </div>

      <!-- Tabs -->
      <div class="flex gap-2 border-b border-border-default pb-2">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="px-4 py-1.5 bg-transparent border-none text-text-secondary cursor-pointer text-sm transition-all rounded hover:bg-surface-elevated hover:text-text-primary"
          :class="{ 'bg-surface-elevated! text-accent-gold!': activeTab === tab.key }"
          @click="activeTab = tab.key">
          {{ tab.label }}
        </button>
      </div>

      <!-- Tab content -->
      <div v-if="store.selectedSnapshot" class="flex-1 flex flex-col gap-4 min-h-0">
        <SkillTable v-if="activeTab === 'skills'" :skills="store.skills" />
        <NpcFavorTable v-if="activeTab === 'favor'" :favor="store.npcFavor" />
        <RecipeTable v-if="activeTab === 'recipes'" :recipes="store.recipes" />
        <StatsTable v-if="activeTab === 'stats'" :stats="store.stats" />
        <CurrencyTable v-if="activeTab === 'currencies'" :currencies="store.currencies" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useCharacterStore } from '../../stores/characterStore'
import SkillTable from './SkillTable.vue'
import NpcFavorTable from './NpcFavorTable.vue'
import RecipeTable from './RecipeTable.vue'
import StatsTable from './StatsTable.vue'
import CurrencyTable from './CurrencyTable.vue'

const store = useCharacterStore()

type TabKey = 'skills' | 'favor' | 'recipes' | 'stats' | 'currencies'

const tabs: { key: TabKey; label: string }[] = [
  { key: 'skills', label: 'Skills' },
  { key: 'favor', label: 'NPC Favor' },
  { key: 'recipes', label: 'Recipes' },
  { key: 'stats', label: 'Stats' },
  { key: 'currencies', label: 'Currencies' },
]

const activeTab = ref<TabKey>('skills')

function onSnapshotChange(event: Event) {
  const id = Number((event.target as HTMLSelectElement).value)
  const snap = store.snapshots.find(s => s.id === id)
  if (snap) {
    store.selectSnapshot(snap)
  }
}

async function handleImport() {
  await store.importCharacterReport()
  // After import, reload for the active character
  if (store.lastImport && !store.lastImport.was_duplicate) {
    await store.initForActiveCharacter()
  }
}

function formatTimestamp(ts: string): string {
  return ts.replace('T', ' ').replace('Z', '').substring(0, 19)
}

onMounted(() => {
  // If we don't already have a selected character loaded, init for active
  if (!store.selectedCharacter) {
    store.initForActiveCharacter()
  }
})
</script>
