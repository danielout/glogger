<template>
  <EntityTooltipWrapper
    border-class="border-entity-recipe/50"
    @hover="loadData"
  >
    <button
      class="inline-flex items-center gap-1 cursor-pointer hover:underline"
      @click="handleClick"
    >
      <GameIcon v-if="showIcon && recipeData?.icon_id" :icon-id="recipeData.icon_id" :alt="name" size="xs" />
      <span class="text-entity-recipe text-xs font-medium">{{ recipeData?.name ?? name }}</span>
    </button>
    <template #tooltip>
      <RecipeTooltip v-if="recipeData" :recipe="recipeData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { RecipeInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import RecipeTooltip from "./RecipeTooltip.vue";

const props = withDefaults(defineProps<{
  name: string;
  showIcon?: boolean;
}>(), {
  showIcon: true,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const recipeData = ref<RecipeInfo | null>(null);
const iconSrc = ref<string | null>(null);

async function loadData() {
  if (recipeData.value) return;
  try {
    const recipe = await store.getRecipeByName(props.name);
    if (!recipe) return;
    recipeData.value = recipe;
    if (recipe.icon_id) {
      const path = await store.getIconPath(recipe.icon_id);
      iconSrc.value = convertFileSrc(path);
    }
  } catch (e) {
    console.warn(`Failed to load recipe: ${props.name}`, e);
  }
}

function handleClick() {
  navigateToEntity({ type: "recipe", id: props.name });
}
</script>
