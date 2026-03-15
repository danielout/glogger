<template>
  <div v-if="skillList.length === 0" class="text-text-dim italic py-4">
    No skill updates yet.
  </div>
  <div v-else class="flex flex-wrap gap-4 pt-4">
    <SkillCard v-for="skill in skillList" :key="skill.skillType" :skill="skill" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSkillStore } from '../../stores/skillStore'
import SkillCard from './SkillCard.vue'

const store = useSkillStore()
const skillList = computed(() => Object.values(store.skills))
</script>
