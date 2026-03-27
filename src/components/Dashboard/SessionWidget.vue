<template>
  <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
    <div class="text-[0.65rem] text-text-muted uppercase tracking-wide mb-2">Session</div>
    <div v-if="topSkill" class="flex flex-col gap-1">
      <div class="text-xs text-text-secondary">Top Skill</div>
      <div class="text-sm text-accent-gold font-bold">{{ topSkill.skillType }}</div>
      <div class="text-xs text-text-secondary">{{ xpPerHour.toLocaleString() }} XP/hr</div>
    </div>
    <EmptyState v-else variant="compact" primary="No session data yet" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import EmptyState from '../Shared/EmptyState.vue'

const store = useGameStateStore()

const topSkill = computed(() => {
  const skills = store.sessionSkillList
  if (skills.length === 0) return null
  return skills.reduce((best, s) => (s.xpGained > best.xpGained ? s : best))
})

const xpPerHour = computed(() => {
  if (!topSkill.value) return 0
  return store.xpPerHour(topSkill.value)
})
</script>
