<template>
  <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
    <h2 class="text-sm font-bold text-text-secondary uppercase tracking-wide mb-3">Moon Phase</h2>

    <div v-if="moonLoading" class="text-text-dim text-sm italic">Calculating moon phase...</div>

    <template v-else-if="phase">
      <!-- Current phase display -->
      <div class="flex items-center gap-3 mb-3">
        <span class="text-4xl leading-none">{{ phase.emoji }}</span>
        <div>
          <div class="text-sm font-medium text-text-primary">{{ phase.label }}</div>
          <div v-if="nextPhaseText" class="text-xs text-text-dim">{{ nextPhaseText }}</div>
        </div>
      </div>

      <!-- Phase cycle bar -->
      <div class="flex gap-0.5 mb-3">
        <div
          v-for="p in ALL_PHASES"
          :key="p.name"
          class="flex-1 h-1.5 rounded-full transition-colors"
          :class="p.name === phase.name ? 'bg-accent-gold' : 'bg-surface-elevated'"
          :title="p.label"
        />
      </div>

      <!-- Active moon quests -->
      <div v-if="questsLoading" class="text-text-dim text-xs italic">Loading quest data...</div>
      <template v-else-if="moonQuests.length > 0">
        <div class="text-xs text-text-muted uppercase tracking-wide mb-1.5">Active Moon Quests</div>
        <div class="flex flex-col gap-1">
          <QuestInline
            v-for="quest in moonQuests"
            :key="quest.internal_name"
            :reference="quest.internal_name"
          />
        </div>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useMoonPhase, ALL_PHASES } from '../../composables/useMoonPhase'
import { useGameDataStore } from '../../stores/gameDataStore'
import type { QuestInfo } from '../../types/gameData'
import QuestInline from '../Shared/Quest/QuestInline.vue'
import { computed } from 'vue'

const { phase, daysUntil, loading: moonLoading } = useMoonPhase()
const gameData = useGameDataStore()

const moonQuests = ref<QuestInfo[]>([])
const questsLoading = ref(false)

const nextPhaseText = computed(() => {
  if (daysUntil.value.length === 0) return null
  const next = daysUntil.value[0]
  return `${next.label} in ${next.days} day${next.days === 1 ? '' : 's'}`
})

watch(() => phase.value?.name, async (phaseName) => {
  if (!phaseName) return
  questsLoading.value = true
  try {
    moonQuests.value = await gameData.getQuestsByMoonPhase(phaseName)
  } catch (e) {
    console.error('[MoonPhaseCard] Failed to load moon quests:', e)
    moonQuests.value = []
  } finally {
    questsLoading.value = false
  }
}, { immediate: true })
</script>
