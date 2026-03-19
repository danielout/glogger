<template>
  <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
    <div class="text-[0.65rem] text-text-muted uppercase tracking-wide mb-2">Active Skills</div>
    <div v-if="skillCount === 0" class="text-text-dim text-sm italic">No active skills</div>
    <template v-else>
      <div class="text-2xl font-bold text-accent-gold mb-1">{{ skillCount }}</div>
      <div class="text-xs text-text-secondary">{{ totalXpGained.toLocaleString() }} total XP gained</div>
      <div class="text-xs text-text-secondary">{{ totalLevelsGained }} level{{ totalLevelsGained !== 1 ? 's' : '' }} gained</div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSkillStore } from '../../stores/skillStore'

const skillStore = useSkillStore()

const skills = computed(() => Object.values(skillStore.skills))
const skillCount = computed(() => skills.value.length)
const totalXpGained = computed(() => skills.value.reduce((sum, s) => sum + s.xpGained, 0))
const totalLevelsGained = computed(() => skills.value.reduce((sum, s) => sum + s.levelsGained, 0))
</script>
