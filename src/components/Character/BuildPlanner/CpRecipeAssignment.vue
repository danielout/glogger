<template>
  <div
    class="flex items-start gap-2 px-2 py-1.5 rounded text-sm bg-surface-elevated border border-border-default group">
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span class="text-[10px] font-semibold text-amber-400 uppercase">{{ typeLabel }}</span>
        <span class="font-medium text-text-primary truncate text-xs">{{ recipe.recipe_name ?? 'Unknown Recipe' }}</span>
        <span class="text-[10px] text-text-dim shrink-0">{{ recipe.cp_cost }} CP</span>
      </div>
    </div>
    <button
      class="text-red-400/60 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer shrink-0 mt-0.5"
      title="Remove"
      @click="emit('remove')">
      x
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { BuildPresetCpRecipe } from '../../../types/buildPlanner'

const props = defineProps<{
  recipe: BuildPresetCpRecipe
}>()

const emit = defineEmits<{
  remove: []
}>()

const typeLabel = computed(() => {
  switch (props.recipe.effect_type) {
    case 'shamanic_infusion': return 'Infusion'
    case 'crafting_enhancement': return 'Enhance'
    default: return 'CP'
  }
})
</script>
