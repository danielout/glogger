<template>
  <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
    <div class="text-[0.65rem] text-text-muted uppercase tracking-wide mb-2">Session</div>
    <div v-if="topSkill" class="flex flex-col gap-1">
      <div class="text-xs text-text-secondary">Top Skill</div>
      <div class="text-sm text-accent-gold font-bold">{{ topSkill.skillType }}</div>
      <div class="text-xs text-text-secondary">{{ xpPerHour.toLocaleString() }} XP/hr</div>
    </div>
    <div v-else class="text-text-dim text-sm italic">No session data yet</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSkillStore } from '../../stores/skillStore'

const skillStore = useSkillStore()

const topSkill = computed(() => {
  const skills = Object.values(skillStore.skills)
  if (skills.length === 0) return null
  return skills.reduce((best, s) => (s.xpGained > best.xpGained ? s : best))
})

const xpPerHour = computed(() => {
  if (!topSkill.value) return 0
  return skillStore.xpPerHour(topSkill.value)
})
</script>
