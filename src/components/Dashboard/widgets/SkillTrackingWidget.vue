<template>
  <EmptyState
    v-if="store.sessionSkillList.length === 0"
    variant="compact"
    primary="No skill updates yet."
    secondary="Start playing to see XP gains here." />
  <div v-else class="flex flex-wrap gap-4">
    <SkillCard v-for="skill in store.sessionSkillList" :key="skill.skillType" :skill="skill" />
  </div>
</template>

<script setup lang="ts">
import { useGameStateStore } from '../../../stores/gameStateStore'
import EmptyState from '../../Shared/EmptyState.vue'
import SkillCard from '../../Shared/SkillCard.vue'

const store = useGameStateStore()
</script>
