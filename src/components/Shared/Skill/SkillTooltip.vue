<template>
  <div class="flex gap-2 items-start mb-2">
    <img
      v-if="iconSrc"
      :src="iconSrc"
      :alt="skill.name"
      class="w-8 h-8 object-contain bg-black/30 border border-border-light rounded shrink-0" />
    <div class="flex-1">
      <div class="font-bold text-entity-skill text-sm mb-0.5">{{ skill.name }}</div>
      <div v-if="skill.xp_table" class="text-text-muted text-xs">XP Table: {{ skill.xp_table }}</div>
    </div>
  </div>

  <div v-if="skill.description" class="text-text-secondary text-xs leading-relaxed mb-2 italic">
    {{ skill.description }}
  </div>

  <div v-if="skill.keywords?.length" class="flex flex-wrap gap-1">
    <span
      v-for="keyword in skill.keywords"
      :key="keyword"
      class="bg-entity-skill/10 text-entity-skill px-1.5 py-0.5 rounded-sm text-[10px] uppercase tracking-wide"
    >
      {{ keyword }}
    </span>
  </div>
</template>

<script setup lang="ts">
import type { SkillInfo } from "../../../types/gameData";

defineProps<{
  skill: SkillInfo;
  iconSrc: string | null;
}>();
</script>
