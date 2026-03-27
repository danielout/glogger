<template>
  <EntityTooltipWrapper
    border-class="border-entity-recipe/50"
    @hover="loadData"
  >
    <component
      :is="plain ? 'span' : 'button'"
      :class="plain
        ? 'hover:underline cursor-pointer text-inherit'
        : 'inline-flex items-center gap-1 cursor-pointer hover:underline'"
      @click="handleClick"
    >
      <GameIcon v-if="!plain && showIcon && recipeData?.icon_id" :icon-id="recipeData.icon_id" :alt="reference" size="xs" />
      <span :class="plain ? '' : 'text-entity-recipe text-xs font-medium'">{{ recipeData?.name ?? reference }}</span>
    </component>
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
  reference: string;
  showIcon?: boolean;
  plain?: boolean;
}>(), {
  showIcon: true,
  plain: false,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const recipeData = ref<RecipeInfo | null>(null);
const iconSrc = ref<string | null>(null);

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

function handleClick() {
  navigateToEntity({ type: "recipe", id: recipeData.value?.name ?? props.reference });
}
</script>
