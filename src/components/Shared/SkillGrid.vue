<template>
  <div v-if="store.sessionSkillList.length === 0" class="text-text-dim italic py-4">
    No skill updates yet.
  </div>
  <div v-else class="flex flex-wrap gap-4 pt-4">
    <SkillCard v-for="skill in store.sessionSkillList" :key="skill.skillType" :skill="skill" />
  </div>
</template>

<script setup lang="ts">
import { useGameStateStore } from '../../stores/gameStateStore'
import SkillCard from './SkillCard.vue'

const store = useGameStateStore()
</script>
