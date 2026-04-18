import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  BrewingRecipe,
  BrewingIngredient,
  BrewingCategory,
  BrewingDiscovery,
  BrewingScanResult,
} from "../types/gameData/brewing";
import { CATEGORY_ORDER, CATEGORY_LABELS } from "../types/gameData/brewing";

export const useBreweryStore = defineStore("brewery", () => {
  // ── State ──────────────────────────────────────────────────────────────────

  const recipes = ref<BrewingRecipe[]>([]);
  const ingredients = ref<BrewingIngredient[]>([]);
  const discoveries = ref<BrewingDiscovery[]>([]);
  const loading = ref(false);
  const loaded = ref(false);
  const scanning = ref(false);
  const error = ref<string | null>(null);

  const selectedRecipeId = ref<number | null>(null);
  const searchQuery = ref("");
  const categoryFilter = ref<BrewingCategory | "all">("all");

  // Right panel state
  const effectSearchQuery = ref("");
  const selectedEffect = ref<string | null>(null);

  // ── Computed ───────────────────────────────────────────────────────────────

  /** Ingredient lookup by item ID */
  const ingredientById = computed(() => {
    const map = new Map<number, BrewingIngredient>();
    for (const ing of ingredients.value) {
      map.set(ing.item_id, ing);
    }
    return map;
  });

  /** Discoveries grouped by recipe ID */
  const discoveriesByRecipe = computed(() => {
    const map = new Map<number, BrewingDiscovery[]>();
    for (const d of discoveries.value) {
      if (!map.has(d.recipe_id)) {
        map.set(d.recipe_id, []);
      }
      map.get(d.recipe_id)!.push(d);
    }
    return map;
  });

  /** Discovery count per recipe (for showing in recipe list) */
  const discoveryCountByRecipe = computed(() => {
    const map = new Map<number, number>();
    for (const [recipeId, discs] of discoveriesByRecipe.value) {
      map.set(recipeId, discs.length);
    }
    return map;
  });

  /** Total discoveries across all recipes */
  const totalDiscoveries = computed(() => discoveries.value.length);

  /** Discoveries for the currently selected recipe */
  const selectedRecipeDiscoveries = computed(() => {
    if (selectedRecipeId.value === null) return [];
    return discoveriesByRecipe.value.get(selectedRecipeId.value) ?? [];
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

  // ── Effect search computed ──────────────────────────────────────────────

  /** All unique effects discovered, with metadata */
  const uniqueEffects = computed(() => {
    const map = new Map<string, {
      power: string;
      effectLabel: string | null;
      raceRestriction: string | null;
      recipeCount: number;
      discoveryCount: number;
      recipeIds: Set<number>;
    }>();

    for (const d of discoveries.value) {
      const existing = map.get(d.power);
      if (existing) {
        existing.discoveryCount++;
        existing.recipeIds.add(d.recipe_id);
        existing.recipeCount = existing.recipeIds.size;
        // Prefer a non-null label
        if (!existing.effectLabel && d.effect_label) {
          existing.effectLabel = d.effect_label;
        }
      } else {
        map.set(d.power, {
          power: d.power,
          effectLabel: d.effect_label,
          raceRestriction: d.race_restriction,
          recipeCount: 1,
          discoveryCount: 1,
          recipeIds: new Set([d.recipe_id]),
        });
      }
    }

    return [...map.values()].sort((a, b) => {
      // Sort: non-race-restricted first, then by label/power name
      if (a.raceRestriction && !b.raceRestriction) return 1;
      if (!a.raceRestriction && b.raceRestriction) return -1;
      const aName = a.effectLabel ?? a.power;
      const bName = b.effectLabel ?? b.power;
      return aName.localeCompare(bName);
    });
  });

  /** Filtered effects based on search query */
  const filteredEffects = computed(() => {
    const q = effectSearchQuery.value.toLowerCase().trim();
    if (!q) return uniqueEffects.value;
    return uniqueEffects.value.filter((e) => {
      const label = e.effectLabel?.toLowerCase() ?? "";
      const power = e.power.toLowerCase();
      return label.includes(q) || power.includes(q);
    });
  });

  /** Discoveries for the selected effect (across all recipes) */
  const selectedEffectDiscoveries = computed(() => {
    if (!selectedEffect.value) return [];
    return discoveries.value.filter((d) => d.power === selectedEffect.value);
  });

  /** Recipe lookup by ID for the right panel */
  const recipeById = computed(() => {
    const map = new Map<number, BrewingRecipe>();
    for (const r of recipes.value) {
      map.set(r.recipe_id, r);
    }
    return map;
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

  async function loadDiscoveries(character: string) {
    try {
      discoveries.value = await invoke<BrewingDiscovery[]>(
        "get_brewing_discoveries",
        { character }
      );
    } catch (e) {
      console.error("Failed to load brewing discoveries:", e);
    }
  }

  async function scanAllSnapshots(character: string): Promise<BrewingScanResult | null> {
    if (scanning.value) return null;
    scanning.value = true;
    try {
      const result = await invoke<BrewingScanResult>(
        "scan_all_snapshots_for_brewing",
        { character }
      );
      // Reload discoveries after scanning
      await loadDiscoveries(character);
      return result;
    } catch (e) {
      console.error("Failed to scan snapshots for brewing:", e);
      error.value = `Scan failed: ${e}`;
      return null;
    } finally {
      scanning.value = false;
    }
  }

  async function importCsv(character: string): Promise<BrewingScanResult | null> {
    if (scanning.value) return null;

    const { open } = await import("@tauri-apps/plugin-dialog");
    const filePath = await open({
      filters: [{ name: "CSV", extensions: ["csv"] }],
      multiple: false,
    });
    if (!filePath) return null;

    scanning.value = true;
    try {
      const result = await invoke<BrewingScanResult>(
        "import_brewing_discoveries_csv",
        { filePath, character }
      );
      await loadDiscoveries(character);
      return result;
    } catch (e) {
      console.error("Failed to import brewing CSV:", e);
      error.value = `CSV import failed: ${e}`;
      return null;
    } finally {
      scanning.value = false;
    }
  }

  function selectRecipe(recipeId: number) {
    selectedRecipeId.value = recipeId;
  }

  function clearSelection() {
    selectedRecipeId.value = null;
  }

  function selectEffect(power: string) {
    selectedEffect.value = power;
  }

  function clearEffectSelection() {
    selectedEffect.value = null;
  }

  return {
    // State
    recipes,
    ingredients,
    discoveries,
    loading,
    loaded,
    scanning,
    error,
    selectedRecipeId,
    searchQuery,
    categoryFilter,
    effectSearchQuery,
    selectedEffect,
    // Computed
    ingredientById,
    discoveriesByRecipe,
    discoveryCountByRecipe,
    totalDiscoveries,
    selectedRecipeDiscoveries,
    recipesByCategory,
    categoryCounts,
    filteredRecipesByCategory,
    selectedRecipe,
    filteredCount,
    uniqueEffects,
    filteredEffects,
    selectedEffectDiscoveries,
    recipeById,
    // Actions
    loadBrewingData,
    loadDiscoveries,
    scanAllSnapshots,
    importCsv,
    selectRecipe,
    clearSelection,
    selectEffect,
    clearEffectSelection,
  };
});
