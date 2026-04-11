<template>
  <div
    class="flex items-start gap-2 px-2 py-1.5 rounded text-sm border transition-all"
    :class="isAssigned
      ? 'bg-surface-elevated border-border-default opacity-40 cursor-default'
      : 'bg-surface-elevated border-border-default hover:bg-surface-hover hover:border-accent-gold/30 cursor-pointer'"
    @click="!isAssigned && emit('add')">
    <GameIcon v-if="recipe.icon_id" :icon-id="recipe.icon_id" size="xs" class="shrink-0 mt-0.5" />
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span class="font-medium text-text-primary truncate text-xs">{{ recipe.recipe_name }}</span>
        <span class="text-[10px] text-amber-400 font-semibold shrink-0">{{ recipe.cp_cost }} CP</span>
      </div>
      <div class="text-[10px] text-text-secondary mt-0.5 line-clamp-2">{{ recipe.effect_description }}</div>
      <div v-if="recipe.skill_level_req" class="text-[10px] text-text-dim mt-0.5">
        Requires {{ recipe.skill }} level {{ recipe.skill_level_req }}
      </div>
    </div>
    <button
      v-if="!isAssigned"
      class="text-accent-gold/70 hover:text-accent-gold text-xs shrink-0 mt-0.5 cursor-pointer"
      title="Add recipe"
      @click.stop="emit('add')">
      +
    </button>
  </div>
</template>

<script setup lang="ts">
import type { CpRecipeOption as CpRecipeOptionType } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'

defineProps<{
  recipe: CpRecipeOptionType
  isAssigned: boolean
}>()

const emit = defineEmits<{
  add: []
}>()
</script>
