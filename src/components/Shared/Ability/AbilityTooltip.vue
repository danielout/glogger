<template>
  <div class="flex gap-2 items-start mb-2">
    <img
      v-if="iconSrc"
      :src="iconSrc"
      :alt="ability.name"
      class="w-8 h-8 object-contain bg-black/30 border border-border-light rounded shrink-0" />
    <div class="flex-1">
      <div class="font-bold text-entity-ability text-sm mb-0.5">{{ ability.name }}</div>
      <div class="flex gap-2 text-xs">
        <span v-if="ability.skill" class="text-entity-skill">{{ ability.skill }}</span>
        <span v-if="ability.level" class="text-text-muted">Lv {{ ability.level }}</span>
      </div>
    </div>
  </div>

  <div v-if="ability.description" class="text-text-secondary text-xs leading-relaxed mb-2 italic">
    {{ ability.description }}
  </div>

  <div v-if="ability.keywords?.length" class="flex flex-wrap gap-1">
    <span
      v-for="keyword in ability.keywords"
      :key="keyword"
      class="bg-entity-ability/10 text-entity-ability px-1.5 py-0.5 rounded-sm text-[0.65rem] uppercase tracking-wide"
    >
      {{ keyword }}
    </span>
  </div>
</template>

<script setup lang="ts">
import type { AbilityInfo } from "../../../types/gameData";

defineProps<{
  ability: AbilityInfo;
  iconSrc: string | null;
}>();
</script>
