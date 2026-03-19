<template>
  <div class="pt-4 flex flex-col gap-4">
    <!-- Status Row -->
    <div class="grid grid-cols-3 gap-4">
      <StatusWidget />
      <ActiveSkillsWidget />
      <SessionWidget />
    </div>

    <!-- Skill Cards (live XP tracking) -->
    <div>
      <h2 class="text-sm font-bold text-text-secondary uppercase tracking-wide mb-3">Live Skill Tracking</h2>
      <div v-if="skillList.length === 0" class="text-text-dim italic text-sm">
        No skill updates yet. Start playing to see XP gains here.
      </div>
      <div v-else class="flex flex-wrap gap-4">
        <SkillCard v-for="skill in skillList" :key="skill.skillType" :skill="skill" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSkillStore } from '../../stores/skillStore'
import SkillCard from '../Shared/SkillCard.vue'
import StatusWidget from './StatusWidget.vue'
import ActiveSkillsWidget from './ActiveSkillsWidget.vue'
import SessionWidget from './SessionWidget.vue'

const skillStore = useSkillStore()
const skillList = computed(() => Object.values(skillStore.skills))
</script>
