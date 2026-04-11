<template>
  <div class="flex flex-col gap-2 overflow-y-auto h-full">
    <div v-if="cpBudget === 0" class="text-xs text-text-dim py-4 text-center">
      No CP budget. Assign a base item to this slot first.
    </div>

    <template v-else>
      <div v-if="store.loadingCpRecipes" class="text-[10px] text-text-muted py-2 text-center">
        Loading recipes...
      </div>
      <div v-else-if="store.availableCpRecipes.length > 0" class="flex flex-col gap-1">
        <span class="text-[10px] text-text-muted">
          {{ store.availableCpRecipes.length }} recipes available
        </span>
        <CpRecipeOption
          v-for="(recipe, idx) in store.availableCpRecipes"
          :key="`${recipe.recipe_id}-${idx}`"
          :recipe="recipe"
          :is-assigned="recipe.cp_cost > cpRemaining"
          @add="store.addCpRecipe(recipe)" />
      </div>
      <div v-else class="text-xs text-text-dim py-4 text-center">
        No CP recipes available for this slot.
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { getSlotCraftingPoints } from '../../../types/buildPlanner'
import CpRecipeOption from './CpRecipeOption.vue'

const store = useBuildPlannerStore()

const cpBudget = computed(() => {
  if (!store.selectedSlot) return 0
  return getSlotCraftingPoints(store.getSlotItem(store.selectedSlot))
})

const cpUsed = computed(() => {
  if (!store.selectedSlot) return 0
  return store.getSlotCpUsed(store.selectedSlot)
})

const cpRemaining = computed(() => cpBudget.value - cpUsed.value)
</script>
