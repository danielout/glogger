<template>
  <div
    class="bg-[#1a1a2e] border border-border-default rounded-lg p-3 min-w-44 max-w-52 cursor-pointer transition-all hover:border-accent-gold/40"
    :class="{ 'border-accent-gold/60': isSelected }"
    @click="$emit('select', skill.skill_name)">
    <div class="mb-1.5">
      <div class="text-sm font-bold text-accent-gold truncate">{{ skill.skill_name }}</div>
      <div class="text-xs text-text-secondary">
        Lv <span class="text-white font-bold">{{ skill.level + skill.bonus_levels }}</span>
        <span v-if="skill.bonus_levels > 0" class="text-accent-gold/60 ml-0.5">({{ skill.level }} + {{ skill.bonus_levels }})</span>
      </div>
    </div>

    <!-- XP Progress bar -->
    <div class="h-1 bg-border-default rounded-sm overflow-hidden mb-1">
      <div class="h-full bg-accent-gold rounded-sm transition-all duration-300" :style="{ width: xpPercent + '%' }"></div>
    </div>

    <!-- Session stats or idle -->
    <div v-if="session" class="flex items-center justify-between text-[0.65rem]">
      <span class="text-accent-gold">+{{ session.xpGained.toLocaleString() }} XP</span>
      <span class="text-text-muted">{{ xphr }}/hr</span>
    </div>
    <div v-else class="text-[0.65rem] text-text-dim italic">idle</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useGameStateStore, type SkillSessionData } from '../../stores/gameStateStore'
import type { GameStateSkill } from '../../types/gameState'

const props = defineProps<{
  skill: GameStateSkill
  session: SkillSessionData | null
  isSelected: boolean
}>()

defineEmits<{
  select: [skillName: string]
}>()

const store = useGameStateStore()

const xphr = computed(() => {
  if (!props.session) return '0'
  return store.xpPerHour(props.session).toLocaleString()
})

const xpPercent = computed(() => {
  if (props.skill.tnl <= 0) return 100
  const total = props.skill.xp + props.skill.tnl
  if (total <= 0) return 0
  return Math.min(100, Math.round((props.skill.xp / total) * 100))
})
</script>
