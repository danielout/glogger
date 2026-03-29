<template>
  <span class="skill-level-display" :title="tooltipText">
    <slot>{{ skill.level }}</slot>
    <span v-if="skill.bonus_levels > 0" class="text-accent-gold/60 ml-0.5 text-[0.65rem]">({{ skill.base_level }}+{{ skill.bonus_levels }})</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  skill: { level: number; base_level: number; bonus_levels: number }
}>()

const tooltipText = computed(() => {
  if (props.skill.bonus_levels > 0) {
    return `Base ${props.skill.base_level} + ${props.skill.bonus_levels} bonus = ${props.skill.level}`
  }
  return `Level ${props.skill.level}`
})
</script>
