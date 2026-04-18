import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  BrewingRecipe,
  BrewingIngredient,
  BrewingCategory,
} from "../types/gameData/brewing";
import { CATEGORY_ORDER, CATEGORY_LABELS } from "../types/gameData/brewing";

export const useBreweryStore = defineStore("brewery", () => {
  // ── State ──────────────────────────────────────────────────────────────────

  const recipes = ref<BrewingRecipe[]>([]);
  const ingredients = ref<BrewingIngredient[]>([]);
  const loading = ref(false);
  const loaded = ref(false);
  const error = ref<string | null>(null);

  const selectedRecipeId = ref<number | null>(null);
  const searchQuery = ref("");
  const categoryFilter = ref<BrewingCategory | "all">("all");

  // ── Computed ───────────────────────────────────────────────────────────────

  /** Ingredient lookup by item ID */
  const ingredientById = computed(() => {
    const map = new Map<number, BrewingIngredient>();
    for (const ing of ingredients.value) {
      map.set(ing.item_id, ing);
    }
    return map;
  });

  /** Recipes grouped by category */
  const recipesByCategory = computed(() => {
    const groups = new Map<BrewingCategory, BrewingRecipe[]>();
    for (const cat of CATEGORY_ORDER) {
      groups.set(cat, []);
    }
    for (const r of recipes.value) {
      groups.get(r.category)?.push(r);
    }
    return groups;
  });

  /** Category counts for filter pills */
  const categoryCounts = computed(() => {
    const counts = new Map<BrewingCategory | "all", number>();
    counts.set("all", recipes.value.length);
    for (const cat of CATEGORY_ORDER) {
      counts.set(cat, recipesByCategory.value.get(cat)?.length ?? 0);
    }
    return counts;
  });

  /** Filtered + searched recipes, grouped by category */
  const filteredRecipesByCategory = computed(() => {
    const q = searchQuery.value.toLowerCase().trim();
    const catFilter = categoryFilter.value;

    const groups: { category: BrewingCategory; label: string; recipes: BrewingRecipe[] }[] = [];

    for (const cat of CATEGORY_ORDER) {
      if (catFilter !== "all" && catFilter !== cat) continue;

      let catRecipes = recipesByCategory.value.get(cat) ?? [];
      if (q) {
        catRecipes = catRecipes.filter(
          (r) =>
            r.name.toLowerCase().includes(q) ||
            (r.internal_name?.toLowerCase().includes(q) ?? false)
        );
      }

      if (catRecipes.length > 0) {
        groups.push({
          category: cat,
          label: CATEGORY_LABELS[cat],
          recipes: catRecipes,
        });
      }
    }
    return groups;
  });

  /** The currently selected recipe */
  const selectedRecipe = computed(() => {
    if (selectedRecipeId.value === null) return null;
    return recipes.value.find((r) => r.recipe_id === selectedRecipeId.value) ?? null;
  });

  /** Total recipe count after filtering */
  const filteredCount = computed(() => {
    return filteredRecipesByCategory.value.reduce(
      (sum, group) => sum + group.recipes.length,
      0
    );
  });

  // ── Actions ────────────────────────────────────────────────────────────────

  async function loadBrewingData() {
    if (loaded.value || loading.value) return;
    loading.value = true;
    error.value = null;

    try {
      const [recipesData, ingredientsData] = await Promise.all([
        invoke<BrewingRecipe[]>("get_brewing_recipes"),
        invoke<BrewingIngredient[]>("get_brewing_ingredients"),
      ]);
      recipes.value = recipesData;
      ingredients.value = ingredientsData;
      loaded.value = true;
    } catch (e) {
      error.value = `Failed to load brewing data: ${e}`;
      console.error(error.value);
    } finally {
      loading.value = false;
    }
  }

  function selectRecipe(recipeId: number) {
    selectedRecipeId.value = recipeId;
  }

  function clearSelection() {
    selectedRecipeId.value = null;
  }

  return {
    // State
    recipes,
    ingredients,
    loading,
    loaded,
    error,
    selectedRecipeId,
    searchQuery,
    categoryFilter,
    // Computed
    ingredientById,
    recipesByCategory,
    categoryCounts,
    filteredRecipesByCategory,
    selectedRecipe,
    filteredCount,
    // Actions
    loadBrewingData,
    selectRecipe,
    clearSelection,
  };
});
