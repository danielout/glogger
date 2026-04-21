<template>
  <EntityTooltipWrapper
    border-class="border-entity-recipe/50"
    entity-type="recipe"
    :entity-reference="reference"
    :entity-label="recipeData?.name ?? reference"
    @hover="loadData"
  >
    <span
      class="inline-flex items-center gap-0.5 cursor-pointer hover:underline font-medium"
      :class="[
        inheritColor ? 'text-inherit' : 'text-entity-recipe',
        bordered ? 'bg-entity-recipe/5 border border-entity-recipe/20 rounded px-1 py-0.5' : '',
      ]"
      @click="handleClick"
    >
      <GameIcon v-if="showIcon && recipeData?.icon_id" :icon-id="recipeData.icon_id" :alt="reference" size="inline" />
      <span>{{ recipeData?.name ?? reference }}</span>
    </span>
    <template #tooltip>
      <RecipeTooltip v-if="recipeData" :recipe="recipeData" :icon-src="iconSrc" :is-learned="isLearned" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useGameStateStore } from "../../../stores/gameStateStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { RecipeInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import RecipeTooltip from "./RecipeTooltip.vue";

const props = withDefaults(defineProps<{
  reference: string;
  showIcon?: boolean;
  bordered?: boolean;
  inheritColor?: boolean;
}>(), {
  showIcon: true,
  bordered: false,
  inheritColor: false,
});

const store = useGameDataStore();
const gameState = useGameStateStore();
const { navigateToEntity } = useEntityNavigation();

const recipeData = ref<RecipeInfo | null>(null);
const iconSrc = ref<string | null>(null);

const isLearned = computed(() => {
  if (!recipeData.value) return null;
  const completions = gameState.recipeCompletions;
  if (Object.keys(completions).length === 0) return null; // no recipe data loaded
  return `Recipe_${recipeData.value.id}` in completions;
});

async function loadData() {
  if (recipeData.value) return;
  try {
    const recipe = await store.resolveRecipe(props.reference);
    if (!recipe) return;
    recipeData.value = recipe;
    if (recipe.icon_id) {
      const path = await store.getIconPath(recipe.icon_id);
      iconSrc.value = convertFileSrc(path);
    }
  } catch (e) {
    console.warn(`Failed to resolve recipe: ${props.reference}`, e);
  }
}

onMounted(loadData);

watch(() => props.reference, () => {
  recipeData.value = null;
  iconSrc.value = null;
  loadData();
});

function handleClick() {
  navigateToEntity({ type: "recipe", id: recipeData.value?.name ?? props.reference });
}
</script>
