<template>
  <PaneLayout
    screen-key="crafting-brewery"
    :left-pane="{ title: 'Recipes', defaultWidth: 280, minWidth: 200, maxWidth: 420 }">
    <template #left>
      <div class="flex flex-col gap-1 h-full min-h-0">
        <!-- Search -->
        <div class="px-2 pt-1">
          <input
            v-model="store.searchQuery"
            type="text"
            placeholder="Search recipes..."
            class="input text-xs w-full" />
        </div>

        <!-- Category filter pills -->
        <div class="flex flex-wrap gap-1 px-2 pb-1">
          <button
            v-for="option in categoryOptions"
            :key="option.value"
            :class="[
              'text-[0.6rem] px-1.5 py-0.5 rounded border cursor-pointer transition-colors',
              store.categoryFilter === option.value
                ? 'bg-accent-gold/20 border-accent-gold/40 text-accent-gold'
                : 'bg-transparent border-border-light text-text-muted hover:text-text-primary',
            ]"
            @click="store.categoryFilter = option.value">
            {{ option.label }}
            <span class="opacity-60">({{ option.count }})</span>
          </button>
        </div>

        <!-- Recipe list -->
        <div class="flex-1 min-h-0 overflow-y-auto">
          <div v-if="store.loading" class="text-text-dim text-xs italic px-2 py-4 text-center">
            Loading brewing data...
          </div>

          <div v-else-if="store.filteredCount === 0" class="text-text-dim text-xs italic px-2 py-4 text-center">
            No recipes found.
          </div>

          <template v-else>
            <div v-for="group in store.filteredRecipesByCategory" :key="group.category">
              <!-- Category header -->
              <div class="text-[0.6rem] uppercase tracking-widest text-text-dim px-2 pt-2 pb-0.5 border-b border-surface-card sticky top-0 bg-surface-base z-10">
                {{ group.label }}
              </div>

              <!-- Recipe items -->
              <button
                v-for="recipe in group.recipes"
                :key="recipe.recipe_id"
                class="flex items-center justify-between px-2 py-1.5 text-xs text-left cursor-pointer border-none w-full"
                :class="store.selectedRecipeId === recipe.recipe_id
                  ? 'bg-accent-gold/15 text-accent-gold'
                  : 'bg-transparent text-text-secondary hover:bg-surface-base'"
                @click="store.selectRecipe(recipe.recipe_id)">
                <span class="truncate">{{ recipe.name }}</span>
                <span class="text-text-muted font-mono shrink-0 ml-2 text-[0.6rem]">Lv{{ recipe.skill_level_req }}</span>
              </button>
            </div>
          </template>
        </div>

        <!-- Count footer -->
        <div v-if="!store.loading" class="text-[0.6rem] text-text-muted px-2 py-1 border-t border-surface-card">
          {{ store.filteredCount }} recipes
        </div>
      </div>
    </template>

    <!-- Center: detail panel -->
    <div class="h-full overflow-y-auto">
      <EmptyState
        v-if="!store.loading && !store.selectedRecipe"
        variant="panel"
        primary="Select a recipe"
        secondary="Choose a brewing recipe from the list to view its ingredients and variable slots." />

      <EmptyState
        v-else-if="store.loading"
        variant="panel"
        primary="Loading brewing data..."
        secondary="Parsing CDN recipe and item data." />

      <BreweryRecipeDetail
        v-else-if="store.selectedRecipe"
        :recipe="store.selectedRecipe"
        :ingredient-by-id="store.ingredientById" />
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import { onMounted, computed } from "vue";
import PaneLayout from "../Shared/PaneLayout.vue";
import EmptyState from "../Shared/EmptyState.vue";
import BreweryRecipeDetail from "./BreweryRecipeDetail.vue";
import { useBreweryStore } from "../../stores/breweryStore";
import { CATEGORY_LABELS } from "../../types/gameData/brewing";
import type { BrewingCategory } from "../../types/gameData/brewing";

const store = useBreweryStore();

const categoryOptions = computed(() => {
  const options: { value: BrewingCategory | "all"; label: string; count: number }[] = [
    { value: "all", label: "All", count: store.categoryCounts.get("all") ?? 0 },
  ];
  for (const [cat, count] of store.categoryCounts) {
    if (cat === "all" || count === 0) continue;
    options.push({
      value: cat as BrewingCategory,
      label: CATEGORY_LABELS[cat as BrewingCategory],
      count,
    });
  }
  return options;
});

onMounted(() => {
  store.loadBrewingData();
});
</script>
