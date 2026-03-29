<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <!-- Summary bar -->
    <div v-if="store.skills.length > 0" class="flex items-center justify-between">
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
          <span class="text-text-muted">Total Levels:</span>
          <span class="text-accent-gold font-bold ml-1">{{ totalLevels.toLocaleString() }}</span>
          <span v-if="totalBonusLevels > 0" class="text-text-dim ml-1">({{ totalBonusLevels }} from bonuses)</span>
        </div>
      </div>
    </div>

    <!-- Tracked skills -->
    <TrackedSkillsBar
      :selected-skill="selectedSkill"
      :show-empty="store.skills.length > 0"
      @select="selectSkill" />

    <!-- Empty state -->
    <EmptyState
      v-if="store.skills.length === 0 && store.sessionSkillList.length === 0"
      primary="No skill data loaded."
      secondary="Log in to a character to see your skills." />

    <!-- Two-panel layout -->
    <div v-else class="flex gap-3 flex-1 min-h-0">
      <div class="w-80 shrink-0 flex flex-col min-h-0">
        <SkillListPanel
          :selected-skill="selectedSkill"
          :cdn-skills="cdnSkillMap"
          @select="selectSkill" />
      </div>

      <SkillDetailPanel :skill="selectedGameStateSkill" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import { useSettingsStore } from '../../stores/settingsStore'
import type { SkillInfo } from '../../types/gameData'
import type { GameStateSkill } from '../../types/gameState'
import EmptyState from '../Shared/EmptyState.vue'
import TrackedSkillsBar from './TrackedSkillsBar.vue'
import SkillListPanel from './SkillListPanel.vue'
import SkillDetailPanel from './SkillDetailPanel.vue'

const store = useGameStateStore()
const gameData = useGameDataStore()
const settingsStore = useSettingsStore()

const selectedSkill = ref<string | null>(null)

// CDN skill data cache for the list panel
const cdnSkillMap = ref<Record<string, SkillInfo>>({})

const totalLevels = computed(() =>
  store.skills.reduce((sum, s) => sum + s.level, 0)
)

const totalBonusLevels = computed(() =>
  store.skills.reduce((sum, s) => sum + s.bonus_levels, 0)
)

const selectedGameStateSkill = computed<GameStateSkill | null>(() => {
  if (!selectedSkill.value) return null
  // Try persisted skills first
  const gs = store.skillsByName[selectedSkill.value]
  if (gs) return gs
  // Fall back to session-only skill
  const session = store.sessionSkills[selectedSkill.value]
  if (session) {
    return {
      skill_id: 0,
      skill_name: selectedSkill.value,
      level: session.currentLevel,
      base_level: session.currentLevel,
      bonus_levels: 0,
      xp: 0,
      tnl: session.tnl,
      max_level: 0,
      last_confirmed_at: session.lastTimestamp,
      source: 'log',
    }
  }
  return null
})

function selectSkill(name: string) {
  selectedSkill.value = name
}

// Load all CDN skills for grouping/icons
async function loadCdnSkills() {
  if (gameData.status !== 'ready') return
  try {
    const all = await gameData.getAllSkills()
    const map: Record<string, SkillInfo> = {}
    for (const s of all) {
      map[s.name] = s
    }
    cdnSkillMap.value = map
  } catch (e) {
    console.warn('Failed to load CDN skills:', e)
  }
}

onMounted(() => {
  loadCdnSkills()
})

watch(() => gameData.status, (s) => {
  if (s === 'ready') loadCdnSkills()
})
</script>
