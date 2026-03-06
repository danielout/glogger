<script setup lang="ts">
import { computed } from 'vue'
import { useSkillStore } from '../stores/skillStore'
import SkillCard from './SkillCard.vue'

const store = useSkillStore()
const skillList = computed(() => Object.values(store.skills))
</script>

<template>
  <div v-if="skillList.length === 0" class="empty">
    No skill updates yet.
  </div>
  <div v-else class="grid">
    <SkillCard v-for="skill in skillList" :key="skill.skillType" :skill="skill" />
  </div>
</template>

<style scoped>
.empty { color: #555; font-style: italic; padding: 1rem 0; }
.grid {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  padding-top: 1rem;
}
</style>