<template>
  <div class="bg-surface-base border border-surface-elevated rounded text-xs">
    <!-- Header row (always visible) -->
    <div
      class="flex items-center gap-3 px-3 py-2 cursor-pointer select-none hover:bg-surface-elevated/30"
      @click="expanded = !expanded">
      <span class="text-text-muted w-3 text-center text-[0.65rem]">
        {{ expanded ? '▾' : '▸' }}
      </span>
      <RecipeInline :name="entry.recipe_name" />
      <span class="text-text-primary font-mono">×{{ entry.quantity }}</span>
      <div class="ml-auto flex items-center gap-2" @click.stop>
        <input
          :value="entry.quantity"
          type="number"
          min="1"
          class="input w-14 text-xs text-center"
          @change="(e: Event) => $emit('update-qty', entry.id, (e.target as HTMLInputElement).valueAsNumber)" />
        <button
          class="text-accent-red/60 text-xs cursor-pointer bg-transparent border-none hover:text-accent-red"
          @click="$emit('remove', entry.id)">
          ✕
        </button>
      </div>
    </div>

    <!-- Expanded ingredient list -->
    <div v-if="expanded && recipe" class="border-t border-surface-dark/50 bg-surface-dark/20 px-2 py-1">
      <div v-if="loading" class="text-text-dim italic py-2 px-2">Loading ingredients...</div>
      <template v-else>
        <IngredientRow
          v-for="(ing, i) in resolvedIngredients"
          :key="ing.item_id ?? `dyn-${i}`"
          :ingredient="ing"
          :is-marked-for-crafting="isMarkedForCrafting(ing.item_id)"
          @toggle-intermediate="(itemId) => $emit('toggle-intermediate', entry.id, itemId)" />
        <div v-if="resolvedIngredients.length === 0" class="text-text-dim italic py-2 px-2">
          No ingredients
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useCraftingStore } from "../../stores/craftingStore";
import type { CraftingProjectEntry, ResolvedIngredient } from "../../types/crafting";
import type { RecipeInfo } from "../../types/gameData/recipes";
import RecipeInline from "../Shared/Recipe/RecipeInline.vue";
import IngredientRow from "./IngredientRow.vue";

const props = defineProps<{
  entry: CraftingProjectEntry
  intermediateExpansions: Map<string, boolean>
}>();

defineEmits<{
  'update-qty': [entryId: number, qty: number]
  'remove': [entryId: number]
  'toggle-intermediate': [entryId: number, itemId: number | null]
}>();

const gameData = useGameDataStore();
const craftingStore = useCraftingStore();

const expanded = ref(false);
const loading = ref(false);

defineExpose({ expanded });
const recipe = ref<RecipeInfo | null>(null);
const resolvedIngredients = ref<ResolvedIngredient[]>([]);

function isMarkedForCrafting(itemId: number | null): boolean {
  if (itemId === null) return false;
  return props.intermediateExpansions.get(`${props.entry.id}:${itemId}`) ?? false;
}

watch(expanded, async (isExpanded) => {
  if (isExpanded && !recipe.value) {
    loading.value = true;
    try {
      recipe.value = await gameData.getRecipeByName(props.entry.recipe_name);
      if (recipe.value) {
        const resolved = await craftingStore.resolveRecipeIngredients(
          recipe.value,
          props.entry.quantity,
          false,
        );
        resolvedIngredients.value = resolved.ingredients;
      }
    } catch (e) {
      console.error("[crafting] Failed to resolve entry ingredients:", e);
    } finally {
      loading.value = false;
    }
  }
});

// Re-resolve if quantity changes while expanded
watch(() => props.entry.quantity, async () => {
  if (expanded.value && recipe.value) {
    const resolved = await craftingStore.resolveRecipeIngredients(
      recipe.value,
      props.entry.quantity,
      false,
    );
    resolvedIngredients.value = resolved.ingredients;
  }
});
</script>
