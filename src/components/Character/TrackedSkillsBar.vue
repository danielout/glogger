<template>
  <div v-if="trackedSkills.length > 0 || showEmpty" class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <div class="text-xs uppercase tracking-widest text-text-dim">Tracked Skills</div>
    </div>

    <div v-if="trackedSkills.length === 0" class="text-xs text-text-dim italic py-2">
      Track skills you want to watch closely. Select a skill below and click Track.
    </div>

    <div v-else class="flex flex-wrap gap-3">
      <TrackedSkillCard
        v-for="gs in trackedSkills"
        :key="gs.skill_name"
        :skill="gs"
        :session="store.sessionSkills[gs.skill_name] ?? null"
        :is-selected="selectedSkill === gs.skill_name"
        @select="$emit('select', $event)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import type { GameStateSkill } from '../../types/gameState'
import TrackedSkillCard from './TrackedSkillCard.vue'

defineProps<{
  selectedSkill: string | null
  showEmpty?: boolean
}>()

defineEmits<{
  select: [skillName: string]
}>()

const store = useGameStateStore()

const trackedSkills = computed<GameStateSkill[]>(() => {
  const names = store.trackedSkillNames
  const byName = store.skillsByName
  return names.map(n => byName[n]).filter((s): s is GameStateSkill => !!s)
})
</script>
